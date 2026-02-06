use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit};

use bitvec::vec::BitVec;

use crate::adjacency_matrix::{AdjacencyMatrix, Asymmetric, BitvecStorage};

/// Bitvec-based asymmetric adjacency matrix for directed graphs.
///
/// Uses a dense square matrix representation with a bitvec to track which entries exist.
/// Automatically resizes to the next power of two for efficient indexing.
/// Requires indices that can be converted to/from usize.
pub struct AsymmetricBitvecAdjacencyMatrix<I, V> {
    data: Vec<MaybeUninit<V>>,
    liveness: BitVec,
    reflected_liveness: BitVec,
    size: usize,
    entry_count: usize,
    index_type: PhantomData<I>,
}

impl<I, V> AsymmetricBitvecAdjacencyMatrix<I, V>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq + Hash,
{
    fn with_size(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let mut liveness = BitVec::with_capacity(capacity * capacity);
        liveness.resize(capacity * capacity, false);
        let mut reflected_liveness = BitVec::with_capacity(capacity * capacity);
        reflected_liveness.resize(capacity * capacity, false);
        let mut data = Vec::with_capacity(capacity * capacity);
        data.resize_with(capacity * capacity, MaybeUninit::uninit);
        AsymmetricBitvecAdjacencyMatrix {
            data,
            liveness,
            reflected_liveness,
            size: capacity,
            entry_count: 0,
            index_type: PhantomData,
        }
    }

    /// Gets the linear storage index for the entry at `row` and `col`, if within bounds.
    fn index(&self, row: I, col: I) -> Option<usize> {
        (row.into() < self.size && col.into() < self.size).then(|| self.unchecked_index(row, col))
    }

    fn is_live(&self, index: usize) -> bool {
        self.liveness[index]
    }

    fn get_data_read(&self, index: usize) -> Option<V> {
        self.is_live(index)
            .then(|| self.unchecked_get_data_read(index))
    }

    fn get_data_ref(&self, index: usize) -> Option<&V> {
        self.is_live(index)
            .then(|| self.unchecked_get_data_ref(index))
    }

    fn unchecked_get_data_read(&self, index: usize) -> V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_read() }
    }

    fn unchecked_get_data_ref(&self, index: usize) -> &V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_ref() }
    }

    /// Gets the linear index for the entry at `row` and `col` without bounds checking.
    fn unchecked_index(&self, row: I, col: I) -> usize {
        (row.into() * self.size) + col.into()
    }

    fn coordinates(&self, index: usize) -> (I, I) {
        Self::coordinates_with(self.size, index)
    }

    fn coordinates_with(size: usize, index: usize) -> (I, I) {
        debug_assert_eq!(size, 1 << size.trailing_zeros());
        debug_assert_eq!(index % size, index & (size - 1));
        let row = index / size;
        let col = index & (size - 1);
        (row.into(), col.into())
    }
}

impl<I, V> Drop for AsymmetricBitvecAdjacencyMatrix<I, V> {
    fn drop(&mut self) {
        // Drop all initialized values
        for index in self.liveness.iter_ones() {
            unsafe {
                self.data[index].assume_init_drop();
            }
        }
    }
}

impl<I, V> AdjacencyMatrix for AsymmetricBitvecAdjacencyMatrix<I, V>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord,
{
    type Index = I;
    type Value = V;
    type Symmetry = Asymmetric;
    type Storage = BitvecStorage;

    fn new() -> Self {
        Self {
            liveness: BitVec::new(),
            reflected_liveness: BitVec::new(),
            size: 0,
            data: Vec::new(),
            entry_count: 0,
            index_type: PhantomData,
        }
    }

    fn insert(&mut self, row: I, col: I, data: V) -> Option<V> {
        if self.index(row, col).is_none() {
            let required_size = usize::max(row.into(), col.into()) + 1;
            if self.size < required_size {
                let mut new_self = Self::with_size(required_size);
                for old_row in 0..self.size {
                    let old_start = self.unchecked_index(old_row.into(), 0.into());
                    let new_start = new_self.unchecked_index(old_row.into(), 0.into());
                    new_self.liveness[new_start..new_start + self.size]
                        .copy_from_bitslice(&self.liveness[old_start..old_start + self.size]);
                    for (old_col, old_datum) in self.data[old_start..old_start + self.size]
                        .iter_mut()
                        .enumerate()
                    {
                        std::mem::swap(old_datum, &mut new_self.data[new_start + old_col]);
                    }
                }
                for old_col in 0..self.size {
                    let old_start = self.unchecked_index(old_col.into(), 0.into());
                    let new_start = new_self.unchecked_index(old_col.into(), 0.into());
                    new_self.reflected_liveness[new_start..new_start + self.size]
                        .copy_from_bitslice(
                            &self.reflected_liveness[old_start..old_start + self.size],
                        );
                }
                new_self.entry_count = self.entry_count;
                // Clear old matrix bits to prevent double-drop
                self.liveness.fill(false);
                self.reflected_liveness.fill(false);
                *self = new_self;
            }
        }
        let index = self.unchecked_index(row, col);
        let old_data = self.get_data_read(index);
        self.liveness.set(index, true);
        let reflected_index = self.unchecked_index(col, row);
        self.reflected_liveness.set(reflected_index, true);
        self.data[index] = MaybeUninit::new(data);
        if old_data.is_none() {
            self.entry_count += 1;
        }
        old_data
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        self.get_data_ref(self.index(row, col)?)
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        let index = self.index(row, col)?;
        let was_live = self.is_live(index);
        self.liveness.set(index, false);
        let reflected_index = self.unchecked_index(col, row);
        debug_assert_eq!(self.reflected_liveness[reflected_index], was_live);
        self.reflected_liveness.set(reflected_index, false);
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
        self.liveness.iter_ones().map(|index| {
            let (row, col) = self.coordinates(index);
            (row, col, self.unchecked_get_data_ref(index))
        })
    }

    fn into_iter(mut self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        let size = self.size;
        let mut result = Vec::new();

        // Collect all live entries
        for index in self.liveness.iter_ones() {
            let (row, col) = Self::coordinates_with(size, index);
            // SAFETY: index is live (from iter_ones)
            let value = unsafe { self.data[index].assume_init_read() };
            result.push((row, col, value));
        }

        // Mark all as dead to prevent double-drop in Drop impl
        self.liveness.fill(false);
        self.reflected_liveness.fill(false);

        result.into_iter()
    }

    fn entries_in_row(&self, row: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let row_start = self.index(row, 0.into()).expect("Invalid row index");
        let row_end = row_start + self.size;
        self.liveness[row_start..row_end]
            .iter_ones()
            .map(move |index| (index.into(), self.unchecked_get_data_ref(row_start + index)))
    }

    fn entries_in_col(&self, col: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let col_start = self.index(col, 0.into()).expect("Invalid column index");
        let col_end = col_start + self.size;
        self.reflected_liveness[col_start..col_end]
            .iter_ones()
            .map(move |index| {
                (
                    index.into(),
                    self.unchecked_get_data_ref(index * self.size + col.into()),
                )
            })
    }

    fn clear(&mut self) {
        // Drop all initialized values before clearing
        for index in self.liveness.iter_ones() {
            unsafe {
                self.data[index].assume_init_drop();
            }
        }
        self.liveness.fill(false);
        self.reflected_liveness.fill(false);
        self.entry_count = 0;
    }

    fn clear_row_and_column(&mut self, row: Self::Index, col: Self::Index) {
        let row_idx = row.into();
        let col_idx = col.into();

        if row_idx >= self.size || col_idx >= self.size {
            return;
        }

        // Clear all entries in the given row
        let row_start = self.unchecked_index(row, 0.into());
        let row_end = row_start + self.size;
        for col_offset in self.liveness[row_start..row_end].iter_ones() {
            let index = row_start + col_offset;
            unsafe {
                self.data[index].assume_init_drop();
            }
            // Update reflected_liveness: reflected_liveness[col*size + row] corresponds to liveness[row*size + col]
            let reflected_index = col_offset * self.size + row_idx;
            self.reflected_liveness.set(reflected_index, false);
            self.entry_count -= 1;
        }
        self.liveness[row_start..row_end].fill(false);

        // Clear all entries in the given column
        let col_start = self.unchecked_index(col, 0.into());
        let col_end = col_start + self.size;
        for row_offset in self.reflected_liveness[col_start..col_end].iter_ones() {
            // reflected_liveness[col*size + row] corresponds to liveness[row*size + col]
            let data_index = row_offset * self.size + col_idx;
            unsafe {
                self.data[data_index].assume_init_drop();
            }
            self.liveness.set(data_index, false);
            self.entry_count -= 1;
        }
        self.reflected_liveness[col_start..col_end].fill(false);
    }

    fn len(&self) -> usize {
        self.entry_count
    }

    fn reserve(&mut self, capacity: usize) {
        if self.size < capacity {
            let mut new_self = Self::with_size(capacity);
            for old_row in 0..self.size {
                let old_row_start = self.unchecked_index(old_row.into(), 0.into());
                let new_row_start = new_self.unchecked_index(old_row.into(), 0.into());
                new_self.liveness[new_row_start..new_row_start + self.size]
                    .copy_from_bitslice(&self.liveness[old_row_start..old_row_start + self.size]);
                for (old_col, old_datum) in self.data[old_row_start..old_row_start + self.size]
                    .iter_mut()
                    .enumerate()
                {
                    std::mem::swap(old_datum, &mut new_self.data[new_row_start + old_col]);
                }
            }
            for old_col in 0..self.size {
                let old_col_start = self.unchecked_index(old_col.into(), 0.into());
                let new_col_start = new_self.unchecked_index(old_col.into(), 0.into());
                new_self.reflected_liveness[new_col_start..new_col_start + self.size]
                    .copy_from_bitslice(
                        &self.reflected_liveness[old_col_start..old_col_start + self.size],
                    );
            }
            new_self.entry_count = self.entry_count;
            // Clear old matrix bits to prevent double-drop
            // Data has been swapped to new_self, so old entries are uninitialized
            self.liveness.fill(false);
            self.reflected_liveness.fill(false);
            *self = new_self;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_util::DropCounter;

    use super::*;

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
    fn test_into_iter() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, "A");
        matrix.insert(2, 3, "B");
        matrix.insert(1, 0, "C");
        let entries: Vec<_> = matrix.into_iter().collect();
        assert_eq!(entries.len(), 3);
        assert!(
            entries
                .iter()
                .any(|(row, col, val)| *row == 0 && *col == 1 && *val == "A")
        );
        assert!(
            entries
                .iter()
                .any(|(row, col, val)| *row == 2 && *col == 3 && *val == "B")
        );
        assert!(
            entries
                .iter()
                .any(|(row, col, val)| *row == 1 && *col == 0 && *val == "C")
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
                // deterministic sparse pattern
                if (i * 31 + j * 17) % 23 == 0 {
                    matrix.insert(i, j, i * nodes + j);
                    entries.push((i, j));
                }
            }
        }

        let mut set: std::collections::HashSet<_> = entries.iter().cloned().collect();
        assert_eq!(matrix.iter().count(), set.len());

        // Remove entries one by one
        let total = set.len();
        for k in 0..total {
            assert!(!set.is_empty());
            // pick an arbitrary entry
            let &(r, c) = set.iter().next().unwrap();
            set.remove(&(r, c));

            let removed = matrix.remove(r, c).expect("expected present");
            assert_eq!(removed, r * nodes + c);

            if k % 50 == 0 {
                // bump reserve to force reallocation/copy behavior
                let desired = matrix.index(r, c).map(|_| nodes + 16).unwrap_or(nodes + 16);
                matrix.reserve(desired);
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

        {
            let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();

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
        }

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
