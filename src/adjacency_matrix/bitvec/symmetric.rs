use std::{fmt::Debug, hash::Hash, marker::PhantomData, mem::MaybeUninit};

use bitvec::vec::BitVec;

use super::symmetric_maxtrix_indexing::SymmetricMatrixIndexing;
use crate::{
    SortedPair,
    adjacency_matrix::{AdjacencyMatrix, BitvecStorage, Symmetric},
};

/// Bitvec-based symmetric adjacency matrix for undirected graphs.
///
/// Uses a packed triangular matrix representation for data storage, while the
/// liveness bitvec is stored as a full matrix. This preserves memory savings
/// for values while allowing O(1) liveness checks by row/col.
/// Requires indices that can be converted to/from usize.
pub struct SymmetricBitvecAdjacencyMatrix<I, V> {
    data: Vec<MaybeUninit<V>>,
    liveness: BitVec,
    indexing: SymmetricMatrixIndexing,
    entry_count: usize,
    index_type: PhantomData<I>,
}

impl<I, V> SymmetricBitvecAdjacencyMatrix<I, V>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq,
{
    pub fn with_size(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let indexing = SymmetricMatrixIndexing::new(capacity);
        let data_storage_size = indexing.storage_size();
        let full_storage_size = capacity
            .checked_mul(capacity)
            .expect("liveness matrix size overflow");
        let mut liveness = BitVec::with_capacity(full_storage_size);
        liveness.resize(full_storage_size, false);
        let mut data = Vec::with_capacity(data_storage_size);
        data.resize_with(data_storage_size, MaybeUninit::uninit);
        Self {
            data,
            liveness,
            indexing,
            entry_count: 0,
            index_type: PhantomData,
        }
    }

    fn liveness_index(&self, row: usize, col: usize) -> usize {
        Self::liveness_index_for(self.indexing.size(), row, col)
    }

    fn is_live(&self, row: usize, col: usize) -> bool {
        let index = self.liveness_index(row, col);
        self.liveness[index]
    }

    fn set_live(&mut self, row: usize, col: usize, live: bool) {
        let index = self.liveness_index(row, col);
        self.liveness.set(index, live);
    }

    fn get_data_read(&self, row: usize, col: usize, data_index: usize) -> Option<V> {
        self.is_live(row, col)
            .then(|| self.unchecked_get_data_read(data_index))
    }

    fn get_data_ref(&self, row: usize, col: usize, data_index: usize) -> Option<&V> {
        self.is_live(row, col)
            .then(|| self.unchecked_get_data_ref(data_index))
    }

    fn unchecked_get_data_read(&self, index: usize) -> V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_read() }
    }

    fn unchecked_get_data_ref(&self, index: usize) -> &V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_ref() }
    }
}

impl<I, V> SymmetricBitvecAdjacencyMatrix<I, V> {
    fn liveness_index_for(size: usize, row: usize, col: usize) -> usize {
        debug_assert!(row < size, "liveness row out of bounds");
        debug_assert!(col < size, "liveness col out of bounds");
        row * size + col
    }

    fn row_col_for_index(size: usize, live_index: usize) -> (usize, usize) {
        (live_index / size, live_index % size)
    }
}

impl<I, V> Drop for SymmetricBitvecAdjacencyMatrix<I, V> {
    fn drop(&mut self) {
        // Drop all initialized values
        // Only drop each unique entry once (row <= col)
        let size = self.indexing.size();
        for live_index in self.liveness.iter_ones() {
            let (row, col) = Self::row_col_for_index(size, live_index);
            if row <= col {
                if let Some(index) = self.indexing.index(row, col) {
                    unsafe {
                        self.data[index].assume_init_drop();
                    }
                }
            }
        }
    }
}

impl<I, V> AdjacencyMatrix for SymmetricBitvecAdjacencyMatrix<I, V>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord,
{
    type Index = I;
    type Value = V;
    type Symmetry = Symmetric;
    type Storage = BitvecStorage;

    fn new() -> Self {
        Self {
            liveness: BitVec::new(),
            data: Vec::new(),
            indexing: SymmetricMatrixIndexing::new(0),
            entry_count: 0,
            index_type: PhantomData,
        }
    }

    fn insert(&mut self, row: I, col: I, data: V) -> Option<V> {
        let (i1, i2) = SortedPair::from((row.into(), col.into())).into();
        if self.indexing.index(i1, i2).is_none() {
            let required_size = (i2 + 1).next_power_of_two();
            if self.indexing.size() < required_size {
                let old_size = self.indexing.size();
                let old_liveness = std::mem::take(&mut self.liveness);

                self.indexing = SymmetricMatrixIndexing::new(required_size);
                let data_storage_size = self.indexing.storage_size();
                let full_storage_size = required_size
                    .checked_mul(required_size)
                    .expect("liveness matrix size overflow");

                let mut new_liveness = BitVec::with_capacity(full_storage_size);
                new_liveness.resize(full_storage_size, false);
                for row in 0..old_size {
                    for col in 0..old_size {
                        let old_index = Self::liveness_index_for(old_size, row, col);
                        if old_liveness[old_index] {
                            let new_index = Self::liveness_index_for(required_size, row, col);
                            new_liveness.set(new_index, true);
                        }
                    }
                }
                self.liveness = new_liveness;
                self.data
                    .resize_with(data_storage_size, MaybeUninit::uninit);
            }
        }
        let index = self.indexing.unchecked_index(i1, i2);
        let old_data = self.get_data_read(i1, i2, index);
        self.set_live(i1, i2, true);
        if i1 != i2 {
            self.set_live(i2, i1, true);
        }
        self.data[index] = MaybeUninit::new(data);
        if old_data.is_none() {
            self.entry_count += 1;
        }
        old_data
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        let row = row.into();
        let col = col.into();
        let index = self.indexing.index(row, col)?;
        self.get_data_ref(row, col, index)
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        let row = row.into();
        let col = col.into();
        let index = self.indexing.index(row, col)?;
        let was_live = self.is_live(row, col);
        self.set_live(row, col, false);
        if row != col {
            self.set_live(col, row, false);
        }
        if was_live {
            self.entry_count -= 1;
            Some(self.unchecked_get_data_read(index))
        } else {
            None
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (I, I, &'a V)>
    where
        V: 'a,
    {
        let size = self.indexing.size();
        self.liveness.iter_ones().filter_map(move |live_index| {
            let (row, col) = Self::row_col_for_index(size, live_index);
            if row > col {
                return None;
            }
            let index = self.indexing.unchecked_index(row, col);
            self.get_data_ref(row, col, index)
                .map(|data| (row.into(), col.into(), data))
        })
    }

    fn into_iter(mut self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        let size = self.indexing.size();
        let mut result = Vec::new();

        // Collect all live entries
        for live_index in self.liveness.iter_ones() {
            let (row, col) = Self::row_col_for_index(size, live_index);
            if row <= col {
                let index = self.indexing.unchecked_index(row, col);
                // SAFETY: live_index is from iter_ones, so this entry is initialized
                let value = unsafe { self.data[index].assume_init_read() };
                result.push((row.into(), col.into(), value));
            }
        }

        // Clear liveness to prevent double-drop in Drop impl
        self.liveness.fill(false);

        result.into_iter()
    }

    fn entries_in_row(&self, row: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let row_idx = row.into();
        let size = self.indexing.size();
        let row_start = Self::liveness_index_for(size, row_idx, 0);
        let row_end = row_start + size;
        self.liveness[row_start..row_end]
            .iter_ones()
            .filter_map(move |col| {
                let index = self.indexing.index(row_idx, col)?;
                self.get_data_ref(row_idx, col, index)
                    .map(|data| (col.into(), data))
            })
    }

    fn entries_in_col(&self, col: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        self.entries_in_row(col)
    }

    fn clear(&mut self) {
        // Drop all initialized values
        let size = self.indexing.size();
        for live_index in self.liveness.iter_ones() {
            let (row, col) = Self::row_col_for_index(size, live_index);
            if row <= col {
                if let Some(index) = self.indexing.index(row, col) {
                    unsafe {
                        self.data[index].assume_init_drop();
                    }
                }
            }
        }
        self.liveness.fill(false);
        self.entry_count = 0;
    }

    fn len(&self) -> usize {
        self.entry_count
    }

    fn reserve(&mut self, capacity: usize) {
        let current_capacity = self.indexing.size();
        if current_capacity < capacity {
            let new_indexing = SymmetricMatrixIndexing::new(capacity);
            let new_data_storage_size = new_indexing.storage_size();
            let new_full_storage_size = capacity
                .checked_mul(capacity)
                .expect("liveness matrix size overflow");

            let mut new_liveness = BitVec::with_capacity(new_full_storage_size);
            new_liveness.resize(new_full_storage_size, false);

            let mut new_data = Vec::with_capacity(new_data_storage_size);
            new_data.resize_with(new_data_storage_size, MaybeUninit::uninit);

            // Copy existing data to the new storage
            for row in 0..current_capacity {
                for col in 0..=row {
                    if let Some(old_index) = self.indexing.index(row, col) {
                        if self.is_live(row, col) {
                            if let Some(new_index) = new_indexing.index(row, col) {
                                let idx1 = Self::liveness_index_for(capacity, row, col);
                                new_liveness.set(idx1, true);
                                if row != col {
                                    let idx2 = Self::liveness_index_for(capacity, col, row);
                                    new_liveness.set(idx2, true);
                                }
                                // SAFETY: old_index is live, so data at that index is initialized
                                new_data[new_index] = MaybeUninit::new(unsafe {
                                    self.data[old_index].assume_init_read()
                                });
                            }
                        }
                    }
                }
            }

            // Clear old liveness to prevent double-drop
            // Data has been read into new_data, so old entries should not be dropped
            self.liveness.fill(false);

            self.indexing = new_indexing;
            self.liveness = new_liveness;
            self.data = new_data;
        }
    }

    fn clear_row_and_column(&mut self, row: Self::Index, col: Self::Index) {
        let row_idx = row.into();
        let col_idx = col.into();

        if row_idx >= self.indexing.size() || col_idx >= self.indexing.size() {
            return;
        }

        let size = self.indexing.size();

        // Collect indices to clear from row row_idx
        let row_start = Self::liveness_index_for(size, row_idx, 0);
        let row_end = row_start + size;
        let row_entries: Vec<_> = self.liveness[row_start..row_end].iter_ones().collect();

        // Clear all entries in row row_idx
        for col_offset in row_entries {
            // Drop the data (only stored once in canonical form)
            let (i1, i2) = if row_idx <= col_offset {
                (row_idx, col_offset)
            } else {
                (col_offset, row_idx)
            };
            if let Some(data_index) = self.indexing.index(i1, i2) {
                unsafe {
                    self.data[data_index].assume_init_drop();
                }
            }
            // Clear both directions in liveness
            self.liveness.set(row_start + col_offset, false);
            let reflected_index = Self::liveness_index_for(size, col_offset, row_idx);
            self.liveness.set(reflected_index, false);
            self.entry_count -= 1;
        }

        // Clear all entries in row col_idx (skip if same as row_idx to avoid double-clearing)
        if col_idx != row_idx {
            let col_row_start = Self::liveness_index_for(size, col_idx, 0);
            let col_row_end = col_row_start + size;
            let col_entries: Vec<_> = self.liveness[col_row_start..col_row_end]
                .iter_ones()
                .collect();

            for col_offset in col_entries {
                // Drop the data (only stored once in canonical form)
                let (i1, i2) = if col_idx <= col_offset {
                    (col_idx, col_offset)
                } else {
                    (col_offset, col_idx)
                };
                if let Some(data_index) = self.indexing.index(i1, i2) {
                    unsafe {
                        self.data[data_index].assume_init_drop();
                    }
                }
                // Clear both directions in liveness
                self.liveness.set(col_row_start + col_offset, false);
                let reflected_index = Self::liveness_index_for(size, col_offset, col_idx);
                self.liveness.set(reflected_index, false);
                self.entry_count -= 1;
            }
        }
    }
}

impl<I, V> Debug for SymmetricBitvecAdjacencyMatrix<I, V>
where
    I: Into<usize>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SymmetricBitvecAdjacencyMatrix {{")?;
        if f.alternate() {
            writeln!(f)?;
            for i in 0..self.indexing.size() {
                write!(f, "    ")?;
                if i >= 20 {
                    writeln!(f, "...")?;
                    break;
                }
                for j in 0..=i {
                    if j > 0 && j % 5 == 0 {
                        write!(f, " ")?;
                    }
                    let live_index = Self::liveness_index_for(self.indexing.size(), i, j);
                    if self.liveness[live_index] {
                        write!(f, "1")?;
                    } else {
                        write!(f, "0")?;
                    }
                }
                writeln!(f)?;
            }
            writeln!(f, "}}")?;
        } else {
            for i in 0..self.indexing.size() {
                write!(f, " ")?;
                for j in 0..=i {
                    if i >= 10 {
                        write!(f, "...")?;
                    }
                    let live_index = Self::liveness_index_for(self.indexing.size(), i, j);
                    if self.liveness[live_index] {
                        write!(f, "1")?;
                    } else {
                        write!(f, "0")?;
                    }
                }
            }
            write!(f, " }}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_debug_empty() {
        let matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
        assert_eq!(
            format!("{:?}", matrix),
            "SymmetricBitvecAdjacencyMatrix { }"
        );
    }

    #[test]
    fn test_debug_with_edges() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(1, 2, ());
        matrix.insert(0, 3, ());
        assert_eq!(
            format!("{:?}", matrix),
            "SymmetricBitvecAdjacencyMatrix { 0 10 010 1000 }"
        );
    }

    #[test]
    fn test_debug_alternate() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(2, 2, ());
        matrix.insert(0, 3, ());
        matrix.insert(0, 25, ());
        assert_eq!(
            format!("{:#?}", matrix),
            r#"SymmetricBitvecAdjacencyMatrix {
    0
    10
    001
    1000
    00000
    00000 0
    00000 00
    00000 000
    00000 0000
    00000 00000
    00000 00000 0
    00000 00000 00
    00000 00000 000
    00000 00000 0000
    00000 00000 00000
    00000 00000 00000 0
    00000 00000 00000 00
    00000 00000 00000 000
    00000 00000 00000 0000
    00000 00000 00000 00000
    ...
}
"#
        );
    }

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

        let mut set: std::collections::HashSet<_> = entries.iter().cloned().collect();
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
                matrix.reserve(nodes + 32);
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
    fn test_reserve_storage_size() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
        let capacity = 7;

        matrix.reserve(capacity);

        let expected_data_storage = triangular(capacity);
        let expected_liveness_storage = capacity * capacity;

        assert_eq!(matrix.indexing.size(), capacity);
        assert_eq!(matrix.data.len(), expected_data_storage);
        assert_eq!(matrix.liveness.len(), expected_liveness_storage);
    }

    #[test]
    fn test_drop_initialized_values() {
        let counter = DropCounter::new();

        {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

            // Insert some values
            matrix.insert(0, 1, counter.new_value());
            matrix.insert(2, 3, counter.new_value());
            matrix.insert(5, 7, counter.new_value());

            // Replace one value (should drop the old one)
            matrix.insert(0, 1, counter.new_value());

            // Remove one value (should drop it)
            matrix.remove(2, 3);

            // At this point:
            // - 1 drop from the replaced value at (0,1)
            // - 1 drop from the removed value at (2,3)
            // Total so far: 2 drops
            assert_eq!(counter.drop_count(), 2);

            // Matrix still holds 2 values: (0,1) and (5,7)
        } // Matrix dropped here - should drop remaining 2 values

        // Total drops: 2 (from operations) + 2 (from matrix drop) = 4
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

        {
            let mut matrix = SymmetricBitvecAdjacencyMatrix::new();

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

        {
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
        } // Drop remaining 2 values

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
