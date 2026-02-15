use std::{fmt::Debug, hash::Hash, marker::PhantomData, mem::MaybeUninit, ops::Range};

use bitvec::{slice::BitSlice, vec::BitVec};

use crate::{
    Directed, DirectednessTrait, Undirected,
    adjacency_matrix::{
        AdjacencyMatrix, BitvecStorage,
        bitvec::indexing::{DataIndex, LivenessIndex, MatrixIndexing},
    },
};

pub type AsymmetricBitvecAdjacencyMatrix<I, V> = BitvecAdjacencyMatrix<I, V, Directed>;
pub type SymmetricBitvecAdjacencyMatrix<I, V> = BitvecAdjacencyMatrix<I, V, Undirected>;

/// Bitvec-based adjacency matrix.
///
/// Uses a bitvec to track which entries exist.  For directed graphs, the data
/// stored in a square matrix of size N is indexed by `row * N + col`.  For
/// undirected graphs, only the upper triangle of the matrix is stored, and
/// entries are indexed by `row * (row + 1) / 2 + col` for `col <= row`.
///
/// Requires indices that can be converted to/from usize.
pub struct BitvecAdjacencyMatrix<I, V, D>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord + Debug,
    D: DirectednessTrait + Default,
{
    /// Linear storage of adjacency data.
    data: Vec<MaybeUninit<V>>,
    /// Bitvec tracking which entries are live (true) or dead (false).  For
    /// directed graphs, this contains two halves:
    /// - The first half corresponds to the regular liveness of entries at `row
    ///   * size + col`.
    /// - The second half corresponds to the reflected liveness of entries at
    ///   `col * size + row`.
    ///
    /// For undirected graphs, the regular liveness is symmetric and the
    /// reflected liveness is implicitly the same as the regular liveness.
    ///
    /// This allows efficient iteration over both rows and columns without
    /// needing to transpose the matrix.
    liveness: BitVec,
    /// Indexing helper to convert between (row, col) and linear indices into the data and liveness vectors.
    indexing: MatrixIndexing<D>,
    /// The number of live entries currently in the matrix.
    entry_count: usize,
    phantom: PhantomData<I>,
}

impl<I, V, D> BitvecAdjacencyMatrix<I, V, D>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord + Debug,
    D: DirectednessTrait + Default,
{
    fn with_size(size: usize) -> Self {
        let indexing = MatrixIndexing::new(size, D::default());
        let liveness_storage_size = indexing.liveness_storage_size();
        let bitvec_size = if D::default().is_directed() {
            2 * liveness_storage_size
        } else {
            liveness_storage_size
        };
        let mut liveness = BitVec::with_capacity(bitvec_size);
        liveness.resize(bitvec_size, false);
        let data_storage_size = indexing.data_storage_size();
        let mut data = Vec::with_capacity(data_storage_size);
        data.resize_with(data_storage_size, MaybeUninit::uninit);
        BitvecAdjacencyMatrix {
            data,
            liveness,
            indexing,
            entry_count: 0,
            phantom: PhantomData,
        }
    }

    fn liveness_range(&self) -> Range<usize> {
        0..self.indexing.liveness_storage_size()
    }

    /// Returns a bit slice containing the regular liveness matrix.
    fn liveness_bits(&self) -> &BitSlice {
        &self.liveness[self.liveness_range()]
    }

    /// Returns a mutable bit slice containing the regular liveness matrix.
    fn liveness_bits_mut(&mut self) -> &mut BitSlice {
        let range = self.liveness_range();
        &mut self.liveness[range]
    }

    fn reflected_liveness_range(&self) -> Range<usize> {
        if D::default().is_directed() {
            self.indexing.liveness_storage_size()..self.liveness.len()
        } else {
            self.liveness_range()
        }
    }

    /// Returns a bit slice containing the reflected liveness matrix.
    fn reflected_liveness_bits(&self) -> &BitSlice {
        &self.liveness[self.reflected_liveness_range()]
    }

    /// Returns a mutable bit slice containing the reflected liveness matrix.
    fn reflected_liveness_bits_mut(&mut self) -> &mut BitSlice {
        let range = self.reflected_liveness_range();
        &mut self.liveness[range]
    }

    fn get_data_ref(&self, index: LivenessIndex) -> Option<&V> {
        self.liveness_bits()[index]
            .then(|| self.unchecked_get_data_ref(self.indexing.liveness_index_to_data_index(index)))
    }

    fn get_data_mut(&mut self, index: LivenessIndex) -> Option<&mut V> {
        if self.liveness_bits()[index] {
            Some(unsafe {
                self.data[self.indexing.liveness_index_to_data_index(index)].assume_init_mut()
            })
        } else {
            None
        }
    }

    fn unchecked_get_data_read(&self, index: DataIndex) -> V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_read() }
    }

    fn unchecked_get_data_ref(&self, index: DataIndex) -> &V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_ref() }
    }

    fn size(&self) -> usize {
        self.indexing.size()
    }
}

impl<I, V, D> Drop for BitvecAdjacencyMatrix<I, V, D>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord + Debug,
    D: DirectednessTrait + Default,
{
    fn drop(&mut self) {
        // Drop all initialized values (only iterate over the liveness bits)
        let size = self.indexing.liveness_storage_size();
        for index in self.liveness[0..size].iter_ones().map(LivenessIndex) {
            let (row, col) = self.indexing.liveness_coordinates(index);
            let data_index = self.indexing.unchecked_data_index(row, col);
            if D::default().is_directed() || row <= col {
                unsafe {
                    self.data[data_index].assume_init_drop();
                }
            }
        }
    }
}

impl<I, V, D> AdjacencyMatrix for BitvecAdjacencyMatrix<I, V, D>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord + Debug,
    D: DirectednessTrait + Default,
{
    type Index = I;
    type Value = V;
    type Directedness = D;
    type Storage = BitvecStorage;

    fn new() -> Self {
        let directedness = D::default();
        Self {
            liveness: BitVec::new(),
            indexing: MatrixIndexing::new(0, directedness),
            data: Vec::new(),
            entry_count: 0,
            phantom: PhantomData,
        }
    }

    fn insert(&mut self, row: I, col: I, data: V) -> Option<V> {
        let row = row.into();
        let col = col.into();
        self.reserve((row.max(col) + 1).saturating_sub(self.size()));

        let liveness_index = self.indexing.unchecked_liveness_index(row, col);
        let data_index = self.indexing.liveness_index_to_data_index(liveness_index);
        let is_live = self.liveness_bits()[liveness_index];

        let old_data = if is_live {
            Some(self.unchecked_get_data_read(data_index))
        } else {
            self.liveness_bits_mut().set(liveness_index.0, true);

            let reflected_index = self.indexing.unchecked_liveness_index(col, row);
            self.reflected_liveness_bits_mut()
                .set(reflected_index.0, true);

            self.entry_count += 1;

            None
        };

        self.data[data_index] = MaybeUninit::new(data);
        old_data
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        let row = row.into();
        let col = col.into();
        self.get_data_ref(self.indexing.liveness_index(row, col)?)
    }

    fn get_mut(&mut self, row: I, col: I) -> Option<&mut V> {
        let row = row.into();
        let col = col.into();
        self.get_data_mut(self.indexing.liveness_index(row, col)?)
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        let row = row.into();
        let col = col.into();
        let data_index = self.indexing.data_index(row, col)?;
        let liveness_index = self.indexing.unchecked_liveness_index(row, col);
        let was_live = self.liveness_bits()[liveness_index];
        self.liveness_bits_mut().set(liveness_index.0, false);

        let reflected_index = self.indexing.unchecked_liveness_index(col, row);
        self.reflected_liveness_bits_mut()
            .set(reflected_index.0, false);

        if was_live {
            self.entry_count -= 1;
            Some(self.unchecked_get_data_read(data_index))
        } else {
            None
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (I, I, &'a V)>
    where
        V: 'a,
    {
        self.liveness_bits()
            .iter_ones()
            .map(LivenessIndex)
            .filter_map(|index| {
                let (row, col) = self.indexing.liveness_coordinates(index);
                if D::default().is_directed() || row <= col {
                    Some((
                        row.into(),
                        col.into(),
                        self.unchecked_get_data_ref(
                            self.indexing.liveness_index_to_data_index(index),
                        ),
                    ))
                } else {
                    None
                }
            })
    }

    fn into_iter(mut self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        let mut result = Vec::new();

        // Collect all live entries
        for index in self.liveness_bits().iter_ones().map(LivenessIndex) {
            let (row, col) = self.indexing.liveness_coordinates(index);
            if D::default().is_directed() || row <= col {
                // SAFETY: index is live (from iter_ones)
                let value =
                    self.unchecked_get_data_read(self.indexing.liveness_index_to_data_index(index));
                result.push((row.into(), col.into(), value));
            }
        }

        // Mark all as dead to prevent double-drop in Drop impl
        self.liveness.fill(false);

        result.into_iter()
    }

    fn entries_in_row(&self, row: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let row = row.into();
        let row_start = self
            .indexing
            .liveness_index(row, 0)
            .expect("Invalid row index");
        let row_end = self.indexing.unchecked_liveness_index(row + 1, 0);
        self.liveness_bits()[row_start.0..row_end.0]
            .iter_ones()
            .map(move |col| {
                (
                    col.into(),
                    self.unchecked_get_data_ref(
                        self.indexing.liveness_index_to_data_index(row_start + col),
                    ),
                )
            })
    }

    fn entries_in_col(&self, col: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let col = col.into();
        let col_start = self
            .indexing
            .liveness_index(col, 0)
            .expect("Invalid column index");
        let col_end = col_start + self.size();
        self.reflected_liveness_bits()[col_start.0..col_end.0]
            .iter_ones()
            .map(move |col| {
                (
                    col.into(),
                    self.unchecked_get_data_ref(
                        self.indexing.liveness_index_to_data_index(col_start + col),
                    ),
                )
            })
    }

    fn clear(&mut self) {
        // Drop all initialized values before clearing
        for index in self.liveness[self.liveness_range()]
            .iter_ones()
            .map(LivenessIndex)
        {
            let (row, col) = self.indexing.liveness_coordinates(index);
            if D::default().is_directed() || row <= col {
                unsafe {
                    self.data[self.indexing.unchecked_data_index(row, col)].assume_init_drop();
                }
            }
        }
        self.liveness.fill(false);
        self.entry_count = 0;
    }

    fn clear_row_and_column(&mut self, row: Self::Index, col: Self::Index) {
        let row = row.into();
        let col = col.into();
        let size = self.size();

        if row >= size || col >= size {
            return;
        }

        // Clear all entries in the given row
        let row_start = self.indexing.unchecked_liveness_index(row, 0);
        let row_end = self.indexing.unchecked_liveness_index(row + 1, 0);
        debug_assert!(row_end.0 - row_start.0 == size);
        let col_offsets: Vec<_> = self.liveness[self.liveness_range()][row_start.0..row_end.0]
            .iter_ones()
            .collect();
        for col_offset in col_offsets {
            unsafe {
                self.data[self.indexing.unchecked_data_index(row, col_offset)].assume_init_drop();
            }
            let reflected_index = col_offset * size + row;
            self.reflected_liveness_bits_mut()
                .set(reflected_index, false);
            self.entry_count -= 1;
        }
        self.liveness_bits_mut()[row_start.0..row_end.0].fill(false);

        // Clear all entries in the given column
        let col_start = self.indexing.unchecked_liveness_index(col, 0);
        let col_end = self.indexing.unchecked_liveness_index(col + 1, 0);
        let row_offsets: Vec<_> = self.reflected_liveness_bits()[col_start.0..col_end.0]
            .iter_ones()
            .collect();
        for row_offset in row_offsets {
            let liveness_index = self.indexing.unchecked_liveness_index(row_offset, col);
            unsafe {
                self.data[self.indexing.liveness_index_to_data_index(liveness_index)]
                    .assume_init_drop();
            }
            self.liveness_bits_mut().set(liveness_index.0, false);
            self.entry_count -= 1;
        }
        self.reflected_liveness_bits_mut()[col_start.0..col_end.0].fill(false);
    }

    fn len(&self) -> usize {
        self.entry_count
    }

    fn reserve(&mut self, additional_capacity: usize) {
        if additional_capacity == 0 {
            return;
        }

        let new_size = (self.size() + additional_capacity).next_power_of_two();
        self.reserve_exact(new_size - self.size());
    }

    fn reserve_exact(&mut self, additional_capacity: usize) {
        if additional_capacity == 0 {
            return;
        }

        let current_capacity = self.size();
        let new_capacity = current_capacity + additional_capacity;
        let mut new_self = Self::with_size(new_capacity);

        if D::default().is_directed() {
            for row in 0..current_capacity {
                let old_start = self.indexing.unchecked_liveness_index(row, 0);
                let old_end = self.indexing.unchecked_liveness_index(row + 1, 0);
                let new_start = new_self.indexing.unchecked_liveness_index(row, 0);
                let new_end = old_end + (new_start - old_start);
                new_self.liveness_bits_mut()[new_start.0..new_start.0 + current_capacity]
                    .copy_from_bitslice(&self.liveness_bits()[old_start.0..old_end.0]);
                new_self.reflected_liveness_bits_mut()[new_start.0..new_end.0]
                    .copy_from_bitslice(&self.reflected_liveness_bits()[old_start.0..old_end.0]);
                for (old_col, old_datum) in
                    self.data[self.indexing.liveness_index_to_data_index(old_start).0
                        ..self.indexing.liveness_index_to_data_index(old_end).0]
                        .iter_mut()
                        .enumerate()
                {
                    std::mem::swap(
                        old_datum,
                        &mut new_self.data[self
                            .indexing
                            .liveness_index_to_data_index(new_start + old_col)
                            .0],
                    );
                }
            }
        } else {
            // Copy existing data to the new storage
            for row in 0..current_capacity {
                for col in 0..=row {
                    if let Some(old_index) = self.indexing.data_index(row, col)
                        && self.liveness[self.indexing.unchecked_liveness_index(row, col)]
                        && let Some(new_index) = new_self.indexing.data_index(row, col)
                    {
                        let idx1 = new_self.indexing.unchecked_liveness_index(row, col);
                        new_self.liveness.set(idx1.0, true);
                        if row != col {
                            let idx2 = new_self.indexing.unchecked_liveness_index(col, row);
                            new_self.liveness.set(idx2.0, true);
                        }
                        new_self.data[new_index] =
                            MaybeUninit::new(self.unchecked_get_data_read(old_index));
                    }
                }
            }
        }

        // Clear old matrix bits to prevent double-drop
        // Data has been swapped to new_self, so old entries are uninitialized
        self.liveness.fill(false);

        new_self.entry_count = self.entry_count;
        *self = new_self;
    }
}

#[cfg(test)]
mod tests {
    mod asymmetric {
        use std::collections::HashSet;

        use crate::test_util::DropCounter;

        use super::super::*;

        #[test]
        fn test_insert_and_get() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            assert_eq!(matrix.insert(0, 1, 42), None);
            assert_eq!(matrix.get(0, 1), Some(&42));
            assert_eq!(matrix.get(1, 0), None);
        }

        #[test]
        fn test_insert_duplicate() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            assert_eq!(matrix.insert(0, 1, ()), None);
            assert_eq!(matrix.insert(0, 1, ()), Some(()));
        }

        #[test]
        fn test_remove() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, ());
            assert_eq!(matrix.remove(0, 1), Some(()));
            assert_eq!(matrix.get(0, 1), None);
        }

        #[test]
        fn test_edges() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, ());
            matrix.insert(2, 3, ());
            let edges: Vec<_> = matrix.iter().collect();
            assert_eq!(edges.len(), 2);
        }

        #[test]
        fn test_edges_from() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, ());
            matrix.insert(0, 3, ());
            let edges: Vec<_> = matrix.entries_in_row(0).collect();
            assert_eq!(edges.len(), 2);
            assert!(edges.iter().any(|(to, _)| *to == 1));
            assert!(edges.iter().any(|(to, _)| *to == 3));
        }

        #[test]
        fn test_edges_into() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, "A");
            assert_eq!(matrix.get(0, 1), Some(&"A"));
            matrix.insert(1, 1, "B");
            assert_eq!(matrix.get(1, 1), Some(&"B"));
            matrix.insert(3, 1, "C");
            assert_eq!(matrix.get(3, 1), Some(&"C"));
            let edges: Vec<_> = matrix.entries_in_col(1).collect();
            assert_eq!(edges.len(), 3);
            assert!(edges.iter().any(|(from, _)| *from == 0));
            assert!(edges.iter().any(|(from, _)| *from == 1));
            assert!(edges.iter().any(|(from, _)| *from == 3));
        }

        #[test]
        fn test_iter() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, "A");
            matrix.insert(2, 3, "B");
            matrix.insert(1, 0, "C");
            let entries: Vec<_> = matrix.iter().collect();
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 0 && col == 1 && val == &"A")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 2 && col == 3 && val == &"B")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 1 && col == 0 && val == &"C")
            );
        }

        #[test]
        fn test_into_iter() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, "A");
            matrix.insert(2, 3, "B");
            matrix.insert(1, 0, "C");
            let entries: Vec<_> = matrix.into_iter().collect();
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 0 && col == 1 && val == "A")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 2 && col == 3 && val == "B")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 1 && col == 0 && val == "C")
            );
        }

        #[test]
        fn test_len() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            assert_eq!(matrix.len(), 0);
            matrix.insert(0, 1, ());
            assert_eq!(matrix.len(), 1);
            matrix.insert(0, 1, ());
            assert_eq!(matrix.len(), 1);
            matrix.insert(2, 3, ());
            assert_eq!(matrix.len(), 2);
            matrix.remove(0, 1);
            assert_eq!(matrix.len(), 1);
            matrix.clear();
            assert_eq!(matrix.len(), 0);
        }

        #[test]
        fn test_large_stress_asymmetric() {
            // Insert many entries across a 100x100 matrix, remove in pseudo-random
            // order, and call reserve occasionally to exercise resizing logic.
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
            let nodes: usize = 120;
            let mut entries = Vec::new();

            for i in 0..nodes {
                for j in 0..nodes {
                    eprintln!("Inserting ({}, {})", i, j);
                    // deterministic sparse pattern
                    if (i * 31 + j * 17) % 23 == 0 {
                        matrix.insert(i, j, i * nodes + j);
                        entries.push((i, j));
                    }
                }
            }

            let mut set: HashSet<_> = entries.iter().cloned().collect();
            assert_eq!(matrix.iter().count(), set.len());

            // Remove entries one by one
            let total = set.len();
            for k in 0..total {
                assert!(!set.is_empty());
                // pick an arbitrary entry
                let &(r, c) = set.iter().next().unwrap();
                eprintln!("Removing ({}, {})", r, c);
                set.remove(&(r, c));

                let removed = matrix.remove(r, c).expect("expected present");
                assert_eq!(removed, r * nodes + c);

                if k % 50 == 0 {
                    // bump reserve to force reallocation/copy behavior
                    matrix.reserve_exact(16);
                    // verify remaining entries are still accessible
                    for &(rr, cc) in set.iter() {
                        assert!(matrix.get(rr, cc).is_some());
                    }
                }
            }

            assert_eq!(matrix.iter().count(), 0);
        }

        #[test]
        fn test_drop_initialized_values() {
            let counter = DropCounter::new();

            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();

            // Insert some values
            matrix.insert(0, 1, counter.new_value());
            matrix.insert(2, 3, counter.new_value());
            matrix.insert(5, 7, counter.new_value());

            // Replace one value (should drop the old one)
            matrix.insert(0, 1, counter.new_value());
            assert_eq!(counter.drop_count(), 1);

            // Remove one value (should drop it)
            matrix.remove(2, 3);
            assert_eq!(counter.drop_count(), 2);

            // Matrix still holds 2 values: (0,1) and (5,7)
            drop(matrix);

            // Total drops: 2 (from operations) + 2 (from matrix drop) = 4
            assert_eq!(counter.drop_count(), 4);
        }

        #[test]
        fn test_no_double_drop_after_into_iter() {
            let counter = DropCounter::new();

            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();

            // Insert some values
            matrix.insert(0, 1, counter.new_value());
            matrix.insert(2, 3, counter.new_value());
            matrix.insert(5, 7, counter.new_value());
            assert_eq!(counter.drop_count(), 0);

            // Consume matrix with into_iter
            let collected: Vec<_> = matrix.into_iter().collect();
            assert_eq!(collected.len(), 3);

            // Values should still be alive in collected
            assert_eq!(counter.drop_count(), 0);

            // Drop the collected values
            drop(collected);

            // Now all 3 values should be dropped exactly once
            assert_eq!(counter.drop_count(), 3);

            // Matrix was consumed by into_iter, so no additional drops
            assert_eq!(counter.drop_count(), 3);
        }

        #[test]
        fn test_no_double_drop_after_clear() {
            let counter = DropCounter::new();

            {
                let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();

                // Insert some values
                matrix.insert(0, 1, counter.new_value());
                matrix.insert(2, 3, counter.new_value());
                matrix.insert(5, 7, counter.new_value());

                assert_eq!(counter.drop_count(), 0);

                // Clear should drop all values
                matrix.clear();

                // All 3 values should be dropped by clear()
                assert_eq!(counter.drop_count(), 3);

                // Add new values after clear
                matrix.insert(1, 2, counter.new_value());
                matrix.insert(3, 4, counter.new_value());

                // Still 3 drops (new values not dropped yet)
                assert_eq!(counter.drop_count(), 3);
            } // Matrix dropped here - should drop the 2 new values

            // Total: 3 (from clear) + 2 (from matrix drop) = 5
            assert_eq!(counter.drop_count(), 5);
        }

        #[test]
        fn test_clear_row_and_column() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();

            // Build a 5x5 matrix with various entries
            // Row 2: (2,0), (2,1), (2,3), (2,4)
            // Col 3: (0,3), (1,3), (2,3), (4,3)
            for i in 0..5 {
                for j in 0..5 {
                    if i == 2 || j == 3 {
                        matrix.insert(i, j, format!("({},{})", i, j));
                    }
                }
            }

            // Should have entries in row 2 and column 3
            assert_eq!(matrix.len(), 9); // 5 in row 2 + 5 in col 3 - 1 overlap at (2,3)

            // Clear row 2 and column 3
            matrix.clear_row_and_column(2, 3);

            // All entries in row 2 and column 3 should be gone
            assert_eq!(matrix.len(), 0);

            // Verify specific entries are removed
            assert_eq!(matrix.get(2, 0), None);
            assert_eq!(matrix.get(2, 1), None);
            assert_eq!(matrix.get(2, 3), None);
            assert_eq!(matrix.get(0, 3), None);
            assert_eq!(matrix.get(1, 3), None);
            assert_eq!(matrix.get(4, 3), None);
        }

        #[test]
        fn test_clear_row_and_column_preserves_other_entries() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();

            // Add entries in a cross pattern centered at (2,2)
            // Row 2: (2,0), (2,1), (2,2), (2,3), (2,4)
            // Col 2: (0,2), (1,2), (2,2), (3,2), (4,2)
            // Plus some other entries that shouldn't be affected
            for i in 0..5 {
                matrix.insert(2, i, format!("R2-{}", i));
                matrix.insert(i, 2, format!("C2-{}", i));
            }
            matrix.insert(0, 0, "corner".to_string());
            matrix.insert(4, 4, "corner2".to_string());
            matrix.insert(1, 3, "other".to_string());

            let initial_len = matrix.len();
            assert_eq!(initial_len, 12); // 5 + 5 - 1 (overlap) + 3 others

            // Clear row 2 and column 2
            matrix.clear_row_and_column(2, 2);

            // Should have removed 9 entries (5 in row + 5 in col - 1 overlap)
            assert_eq!(matrix.len(), 3);

            // Verify other entries still exist
            assert_eq!(matrix.get(0, 0), Some(&"corner".to_string()));
            assert_eq!(matrix.get(4, 4), Some(&"corner2".to_string()));
            assert_eq!(matrix.get(1, 3), Some(&"other".to_string()));

            // Verify row 2 and col 2 are cleared
            for i in 0..5 {
                assert_eq!(matrix.get(2, i), None);
                assert_eq!(matrix.get(i, 2), None);
            }
        }

        #[test]
        fn test_clear_row_and_column_drops_values() {
            let counter = DropCounter::new();

            {
                let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();

                // Add entries in row 1 and column 1
                matrix.insert(1, 0, counter.new_value());
                matrix.insert(1, 2, counter.new_value());
                matrix.insert(0, 1, counter.new_value());
                matrix.insert(2, 1, counter.new_value());
                matrix.insert(1, 1, counter.new_value()); // overlap

                // Add other entries that should not be dropped
                matrix.insert(0, 0, counter.new_value());
                matrix.insert(2, 2, counter.new_value());

                assert_eq!(counter.drop_count(), 0);
                assert_eq!(matrix.len(), 7);

                // Clear row 1 and column 1
                matrix.clear_row_and_column(1, 1);

                // Should have dropped 5 values (from row 1 and col 1)
                assert_eq!(counter.drop_count(), 5);
                assert_eq!(matrix.len(), 2);

                // Other entries still alive
            } // Drop remaining 2 values

            assert_eq!(counter.drop_count(), 7);
        }

        #[test]
        fn test_clear_row_and_column_out_of_bounds() {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();

            matrix.insert(0, 0, "test");
            matrix.insert(1, 1, "test2");

            // Should not panic or affect existing entries
            matrix.clear_row_and_column(100, 200);

            assert_eq!(matrix.len(), 2);
            assert_eq!(matrix.get(0, 0), Some(&"test"));
            assert_eq!(matrix.get(1, 1), Some(&"test2"));
        }
    }

    mod symmetric {
        use std::collections::HashSet;

        use super::super::*;
        use crate::{test_util::DropCounter, triangular::triangular};

        #[test]
        fn test_insert_and_get() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, ());
            assert_eq!(matrix.get(0, 1), Some(&()));
            assert_eq!(matrix.get(1, 0), Some(&()));
        }

        #[test]
        fn test_entries_in_row() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, ());
            matrix.insert(0, 2, ());
            let entries: Vec<_> = matrix.entries_in_row(0).collect();
            assert_eq!(entries.len(), 2);
        }

        #[test]
        fn test_entries_in_col() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 2, ());
            matrix.insert(1, 2, ());
            matrix.insert(3, 3, ());
            assert_eq!(
                matrix.entries_in_col(2).collect::<Vec<_>>(),
                vec![(0, &()), (1, &())]
            );
            assert_eq!(matrix.entries_in_col(3).collect::<Vec<_>>(), vec![(3, &())]);
        }

        #[test]
        fn test_entries_in_col2() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, ());
            assert_eq!(matrix.entries_in_col(1).collect::<Vec<_>>(), vec![(0, &())]);
            assert_eq!(matrix.entries_in_col(0).collect::<Vec<_>>(), vec![(1, &())]);
        }

        #[test]
        fn test_insert_duplicate() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            assert_eq!(matrix.insert(0, 1, ()), None);
            assert_eq!(matrix.insert(0, 1, ()), Some(()));
        }

        #[test]
        fn test_remove() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, ());
            assert_eq!(matrix.remove(0, 1), Some(()));
            assert_eq!(matrix.get(0, 1), None);
        }

        #[test]
        fn test_remove_both_directions() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, ());
            assert_eq!(matrix.remove(1, 0), Some(()));
            assert_eq!(matrix.get(0, 1), None);
            assert_eq!(matrix.get(1, 0), None);
        }

        #[test]
        fn test_remove_nonexistent() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::<_, ()>::new();
            assert_eq!(matrix.remove(0, 1), None);
        }

        #[test]
        fn test_len() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            assert_eq!(matrix.len(), 0);
            matrix.insert(0, 1, "edge");
            assert_eq!(matrix.get(0, 1), Some(&"edge"));
            assert_eq!(matrix.get(1, 0), Some(&"edge"));
            assert_eq!(matrix.len(), 1);
            matrix.insert(1, 0, "edge");
            assert_eq!(matrix.get(0, 1), Some(&"edge"));
            assert_eq!(matrix.get(1, 0), Some(&"edge"));
            assert_eq!(matrix.len(), 1);
            matrix.insert(2, 2, "loop");
            assert_eq!(matrix.len(), 2);
            matrix.remove(1, 0);
            assert_eq!(matrix.get(0, 1), None);
            assert_eq!(matrix.get(1, 0), None);
            assert_eq!(matrix.len(), 1);
            matrix.clear();
            assert_eq!(matrix.len(), 0);
        }

        #[test]
        fn test_entries() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(0, 1, "a");
            matrix.insert(3, 2, "b");
            let mut entries: Vec<_> = matrix.iter().collect();
            entries.sort();
            assert_eq!(entries, vec![(0, 1, &"a"), (2, 3, &"b")]);
        }

        #[test]
        fn test_large_indices() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(100, 200, ());
            assert_eq!(matrix.get(100, 200), Some(&()));
            assert_eq!(matrix.get(200, 100), Some(&()));
        }

        #[test]
        fn test_self_loop() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
            matrix.insert(5, 5, ());
            assert_eq!(matrix.get(5, 5), Some(&()));
        }

        //         #[test]
        //         fn test_debug_empty() {
        //             let matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
        //             assert_eq!(
        //                 format!("{:?}", matrix),
        //                 "SymmetricBitvecAdjacencyMatrix { }"
        //             );
        //         }

        //         #[test]
        //         fn test_debug_with_edges() {
        //             let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        //             matrix.insert(0, 1, ());
        //             matrix.insert(1, 2, ());
        //             matrix.insert(0, 3, ());
        //             assert_eq!(
        //                 format!("{:?}", matrix),
        //                 "SymmetricBitvecAdjacencyMatrix { 0 10 010 1000 }"
        //             );
        //         }

        //         #[test]
        //         fn test_debug_alternate() {
        //             let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        //             matrix.insert(0, 1, ());
        //             matrix.insert(2, 2, ());
        //             matrix.insert(0, 3, ());
        //             matrix.insert(0, 25, ());
        //             assert_eq!(
        //                 format!("{:#?}", matrix),
        //                 r#"SymmetricBitvecAdjacencyMatrix {
        //     0
        //     10
        //     001
        //     1000
        //     00000
        //     00000 0
        //     00000 00
        //     00000 000
        //     00000 0000
        //     00000 00000
        //     00000 00000 0
        //     00000 00000 00
        //     00000 00000 000
        //     00000 00000 0000
        //     00000 00000 00000
        //     00000 00000 00000 0
        //     00000 00000 00000 00
        //     00000 00000 00000 000
        //     00000 00000 00000 0000
        //     00000 00000 00000 00000
        //     ...
        // }
        // "#
        //             );
        //         }

        #[test]
        fn test_large_stress_symmetric() {
            // Insert many edges into the symmetric matrix (undirected), remove
            // them in pseudo-random order, and call reserve to exercise resizing.
            let mut matrix = SymmetricBitvecAdjacencyMatrix::<usize, usize>::new();
            let nodes: usize = 120;
            let mut entries = Vec::new();

            for i in 0..nodes {
                for j in 0..=i {
                    // deterministic sparse pattern
                    if (i * 29 + j * 13) % 19 == 0 {
                        matrix.insert(i, j, i * nodes + j);
                        if ((i + j) % 7) == 0 {
                            // insert both directions to test symmetry
                            matrix.insert(j, i, i * nodes + j);
                        }
                        entries.push((i, j));
                    }
                }
            }

            let mut set: HashSet<_> = entries.iter().cloned().collect();
            assert_eq!(matrix.iter().count(), set.len());

            // Remove entries and occasionally reserve a larger size
            let total = set.len();
            for k in 0..total {
                assert!(!set.is_empty());
                let &(a, b) = set.iter().next().unwrap();
                set.remove(&(a, b));

                let removed = matrix.remove(a, b).expect("expected present");
                assert_eq!(removed, a * nodes + b);

                if k % 60 == 0 {
                    matrix.reserve_exact(32);
                    for &(x, y) in set.iter() {
                        // undirected: both directions should be accessible
                        assert!(matrix.get(x, y).is_some());
                        assert!(matrix.get(y, x).is_some());
                    }
                }
            }

            assert_eq!(matrix.iter().count(), 0);
        }

        #[test]
        fn test_reserve_exact() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
            let capacity = 7;

            matrix.reserve_exact(capacity);
            let expected_data_storage = triangular(capacity);
            let expected_liveness_storage = capacity * capacity;

            assert_eq!(matrix.indexing.size(), capacity);
            assert_eq!(matrix.data.len(), expected_data_storage);
            assert_eq!(matrix.liveness.len(), expected_liveness_storage);
        }

        #[test]
        fn test_drop_initialized_values() {
            let counter = DropCounter::new();

            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

            // Insert some values
            matrix.insert(0, 1, counter.new_value());
            matrix.insert(2, 3, counter.new_value());
            matrix.insert(5, 7, counter.new_value());

            // Replace one value (should drop the old one)
            matrix.insert(0, 1, counter.new_value());
            assert_eq!(counter.drop_count(), 1);

            // Remove one value (should drop it)
            matrix.remove(2, 3);
            assert_eq!(counter.drop_count(), 2);

            // Total drops: 2 (from operations) + 2 (from matrix drop) = 4
            drop(matrix);
            assert_eq!(counter.drop_count(), 4);
        }

        #[test]
        fn test_no_double_drop_after_into_iter() {
            let counter = DropCounter::new();

            {
                let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

                // Insert some values
                matrix.insert(0, 1, counter.new_value());
                matrix.insert(2, 3, counter.new_value());
                matrix.insert(5, 7, counter.new_value());

                assert_eq!(counter.drop_count(), 0);
                // Consume matrix with into_iter
                let collected: Vec<_> = matrix.into_iter().collect();
                assert_eq!(collected.len(), 3);

                // Values should still be alive in collected
                assert_eq!(counter.drop_count(), 0);

                // Drop the collected values
                drop(collected);

                // Now all 3 values should be dropped exactly once
                assert_eq!(counter.drop_count(), 3);
            }

            // Matrix was consumed by into_iter, so no additional drops
            assert_eq!(counter.drop_count(), 3);
        }

        #[test]
        fn test_no_double_drop_after_clear() {
            let counter = DropCounter::new();

            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

            // Insert some values
            matrix.insert(0, 1, counter.new_value());
            matrix.insert(2, 3, counter.new_value());
            matrix.insert(5, 7, counter.new_value());
            assert_eq!(counter.drop_count(), 0);

            // Clear should drop all values
            matrix.clear();
            assert_eq!(counter.drop_count(), 3);

            // Add new values after clear
            matrix.insert(1, 2, counter.new_value());
            matrix.insert(3, 4, counter.new_value());

            // Still 3 drops (new values not dropped yet)
            assert_eq!(counter.drop_count(), 3);

            drop(matrix);

            // Total: 3 (from clear) + 2 (from matrix drop) = 5
            assert_eq!(counter.drop_count(), 5);
        }

        #[test]
        fn test_clear_row_and_column() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

            // Build a symmetric matrix
            // Edges involving node 2: (0,2), (1,2), (2,3), (2,4)
            // These should all be removed when clearing row/col 2
            matrix.insert(0, 2, "edge_0_2");
            matrix.insert(1, 2, "edge_1_2");
            matrix.insert(2, 3, "edge_2_3");
            matrix.insert(2, 4, "edge_2_4");

            // Add some other edges that should remain
            matrix.insert(0, 1, "edge_0_1");
            matrix.insert(3, 4, "edge_3_4");

            assert_eq!(matrix.len(), 6);

            // Clear row 2 and column 2 (which are the same in symmetric matrix)
            matrix.clear_row_and_column(2, 2);

            // Should have removed all edges involving node 2
            assert_eq!(matrix.len(), 2);

            // Verify edges involving node 2 are gone
            assert_eq!(matrix.get(0, 2), None);
            assert_eq!(matrix.get(2, 0), None);
            assert_eq!(matrix.get(1, 2), None);
            assert_eq!(matrix.get(2, 1), None);
            assert_eq!(matrix.get(2, 3), None);
            assert_eq!(matrix.get(3, 2), None);
            assert_eq!(matrix.get(2, 4), None);
            assert_eq!(matrix.get(4, 2), None);

            // Verify other edges remain
            assert_eq!(matrix.get(0, 1), Some(&"edge_0_1"));
            assert_eq!(matrix.get(1, 0), Some(&"edge_0_1"));
            assert_eq!(matrix.get(3, 4), Some(&"edge_3_4"));
            assert_eq!(matrix.get(4, 3), Some(&"edge_3_4"));
        }

        #[test]
        fn test_clear_row_and_column_with_different_indices() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

            // Build a graph with edges
            matrix.insert(0, 1, "a");
            matrix.insert(0, 2, "b");
            matrix.insert(1, 2, "c");
            matrix.insert(1, 3, "d");
            matrix.insert(2, 3, "e");
            matrix.insert(3, 4, "f");

            assert_eq!(matrix.len(), 6);

            // Clear row 1 and column 2 (should remove all edges involving nodes 1 or 2)
            matrix.clear_row_and_column(1, 2);

            // Edges involving node 1: (0,1), (1,2), (1,3)
            // Edges involving node 2: (0,2), (1,2), (2,3)
            // Union: (0,1), (0,2), (1,2), (1,3), (2,3) = 5 edges removed
            // Remaining: (3,4)
            assert_eq!(matrix.len(), 1);
            assert_eq!(matrix.get(3, 4), Some(&"f"));

            // Verify all edges involving 1 or 2 are gone
            assert_eq!(matrix.get(0, 1), None);
            assert_eq!(matrix.get(0, 2), None);
            assert_eq!(matrix.get(1, 2), None);
            assert_eq!(matrix.get(1, 3), None);
            assert_eq!(matrix.get(2, 3), None);
        }

        #[test]
        fn test_clear_row_and_column_drops_values() {
            let counter = DropCounter::new();

            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

            // Add edges involving node 1
            matrix.insert(0, 1, counter.new_value());
            matrix.insert(1, 2, counter.new_value());
            matrix.insert(1, 3, counter.new_value());

            // Add other edges
            matrix.insert(0, 2, counter.new_value());
            matrix.insert(2, 3, counter.new_value());

            assert_eq!(counter.drop_count(), 0);
            assert_eq!(matrix.len(), 5);

            // Clear row 1 and column 1
            matrix.clear_row_and_column(1, 1);

            // Should have dropped 3 values (edges involving node 1)
            assert_eq!(counter.drop_count(), 3);
            assert_eq!(matrix.len(), 2);

            // Drop remaining 2 values
            drop(matrix);

            assert_eq!(counter.drop_count(), 5);
        }

        #[test]
        fn test_clear_row_and_column_self_loop() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

            // Add a self-loop and other edges
            matrix.insert(1, 1, "self_loop");
            matrix.insert(0, 1, "edge_0_1");
            matrix.insert(1, 2, "edge_1_2");
            matrix.insert(0, 2, "edge_0_2");

            assert_eq!(matrix.len(), 4);

            // Clear row 1 and column 1
            matrix.clear_row_and_column(1, 1);

            // Should remove self-loop and edges to node 1
            assert_eq!(matrix.len(), 1);
            assert_eq!(matrix.get(0, 2), Some(&"edge_0_2"));
            assert_eq!(matrix.get(1, 1), None);
            assert_eq!(matrix.get(0, 1), None);
            assert_eq!(matrix.get(1, 2), None);
        }

        #[test]
        fn test_clear_row_and_column_out_of_bounds() {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

            matrix.insert(0, 1, "test");
            matrix.insert(1, 2, "test2");

            // Should not panic or affect existing entries
            matrix.clear_row_and_column(100, 200);

            assert_eq!(matrix.len(), 2);
            assert_eq!(matrix.get(0, 1), Some(&"test"));
            assert_eq!(matrix.get(1, 2), Some(&"test2"));
        }
    }
}
