use std::{fmt::Debug, mem::MaybeUninit, ops::Range};

use bitvec::{slice::BitSlice, vec::BitVec};

use crate::{
    DirectednessTrait,
    adjacency_matrix::{
        AdjacencyMatrix, BitvecStorage,
        bitvec::indexing::{DataIndex, LivenessIndex, MatrixIndexing},
        trait_def::format_debug,
    },
};

/// Bitvec-based adjacency matrix.
///
/// Uses a bitvec to track which entries exist.  For directed graphs, the data
/// stored in a square matrix of size N is indexed by `row * N + col`.  For
/// undirected graphs, only the upper triangle of the matrix is stored, and
/// entries are indexed by `row * (row + 1) / 2 + col` for `col <= row`.
///
/// Requires indices that can be converted to/from usize.
pub struct BitvecAdjacencyMatrix<V, D>
where
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
}

impl<V, D> BitvecAdjacencyMatrix<V, D>
where
    D: DirectednessTrait + Default,
{
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
}

impl<V, D> Drop for BitvecAdjacencyMatrix<V, D>
where
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

impl<V, D> AdjacencyMatrix for BitvecAdjacencyMatrix<V, D>
where
    D: DirectednessTrait + Default,
{
    type Value = V;
    type Directedness = D;
    type Storage = BitvecStorage;

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
        }
    }

    fn size_bound(&self) -> usize {
        self.indexing.size()
    }

    fn insert(&mut self, row: usize, col: usize, data: V) -> Option<V> {
        self.reserve((row.max(col) + 1).saturating_sub(self.size_bound()));

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

    fn get(&self, row: usize, col: usize) -> Option<&V> {
        self.get_data_ref(self.indexing.liveness_index(row, col)?)
    }

    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut V> {
        self.get_data_mut(self.indexing.liveness_index(row, col)?)
    }

    fn remove(&mut self, row: usize, col: usize) -> Option<V> {
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

    fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, usize, &'a V)>
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
                        row,
                        col,
                        self.unchecked_get_data_ref(
                            self.indexing.liveness_index_to_data_index(index),
                        ),
                    ))
                } else {
                    None
                }
            })
    }

    fn into_iter(mut self) -> impl Iterator<Item = (usize, usize, Self::Value)> {
        let mut result = Vec::new();

        // Collect all live entries
        for index in self.liveness_bits().iter_ones().map(LivenessIndex) {
            let (row, col) = self.indexing.liveness_coordinates(index);
            if D::default().is_directed() || row <= col {
                // SAFETY: index is live (from iter_ones)
                let value =
                    self.unchecked_get_data_read(self.indexing.liveness_index_to_data_index(index));
                result.push((row, col, value));
            }
        }

        // Mark all as dead to prevent double-drop in Drop impl
        self.liveness.fill(false);

        result.into_iter()
    }

    fn entries_in_row(&self, row: usize) -> impl Iterator<Item = (usize, &'_ V)> + '_ {
        let size = self.size_bound();
        let row_start = (row < size).then(|| self.indexing.unchecked_liveness_index(row, 0));
        row_start.into_iter().flat_map(move |row_start| {
            let row_end = self.indexing.unchecked_liveness_index(row + 1, 0);
            self.liveness_bits()[row_start.0..row_end.0]
                .iter_ones()
                .map(move |col_offset| {
                    let data_index = self.indexing.unchecked_data_index(row, col_offset);
                    (col_offset, self.unchecked_get_data_ref(data_index))
                })
        })
    }

    fn entries_in_col(&self, col: usize) -> impl Iterator<Item = (usize, &'_ V)> + '_ {
        let size = self.size_bound();
        let col_start = (col < size).then(|| self.indexing.unchecked_liveness_index(col, 0));
        col_start.into_iter().flat_map(move |col_start| {
            let col_end = col_start + size;
            self.reflected_liveness_bits()[col_start.0..col_end.0]
                .iter_ones()
                .map(move |row_offset| {
                    let data_index = self.indexing.unchecked_data_index(row_offset, col);
                    (row_offset, self.unchecked_get_data_ref(data_index))
                })
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

    fn clear_row_and_column(&mut self, row: usize, col: usize) {
        let size = self.size_bound();

        if row >= size || col >= size {
            return;
        }

        // Clear all entries in the given row
        let row_start = self.indexing.unchecked_liveness_index(row, 0);
        let row_end = self.indexing.unchecked_liveness_index(row + 1, 0);
        debug_assert_eq!(row_end.0 - row_start.0, size);
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

        let new_size = (self.size_bound() + additional_capacity).next_power_of_two();
        self.reserve_exact(new_size - self.size_bound());
    }

    fn reserve_exact(&mut self, additional_capacity: usize) {
        if additional_capacity == 0 {
            return;
        }

        let current_capacity = self.size_bound();
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

impl<V, D> Default for BitvecAdjacencyMatrix<V, D>
where
    D: DirectednessTrait + Default,
{
    fn default() -> Self {
        Self::with_size(0)
    }
}

impl<V, D> Clone for BitvecAdjacencyMatrix<V, D>
where
    V: Clone,
    D: DirectednessTrait + Default,
{
    fn clone(&self) -> Self {
        let mut new_self = Self::with_size(self.size_bound());
        new_self.liveness.clone_from(&self.liveness);
        for index in self.liveness_bits().iter_ones().map(LivenessIndex) {
            let (row, col) = self.indexing.liveness_coordinates(index);
            if D::default().is_directed() || row <= col {
                let data_index = self.indexing.liveness_index_to_data_index(index);
                new_self.data[data_index] =
                    MaybeUninit::new(self.unchecked_get_data_ref(data_index).clone());
            }
        }
        new_self.entry_count = self.entry_count;
        new_self
    }
}

impl<V, D> Debug for BitvecAdjacencyMatrix<V, D>
where
    V: Debug,
    D: DirectednessTrait + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "BitvecAdjacencyMatrix")
    }
}

#[cfg(test)]
mod tests {
    use crate::adjacency_matrix_tests;

    adjacency_matrix_tests!(
        directed,
        BitvecAdjacencyMatrix<T, Directed>);
    adjacency_matrix_tests!(
        undirected,
        BitvecAdjacencyMatrix<T, Undirected>);

    // #[test]
    // fn test_debug_empty() {
    //     let matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
    //     assert_eq!(
    //         format!("{:?}", matrix),
    //         "SymmetricBitvecAdjacencyMatrix { }"
    //     );
    // }

    // #[test]
    // fn test_debug_with_edges() {
    //     let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
    //     matrix.insert(0, 1, ());
    //     matrix.insert(1, 2, ());
    //     matrix.insert(0, 3, ());
    //     assert_eq!(
    //         format!("{:?}", matrix),
    //         "SymmetricBitvecAdjacencyMatrix { 0 10 010 1000 }"
    //     );
    // }

    // #[test]
    // fn test_debug_alternate() {
    //     let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
    //     matrix.insert(0, 1, ());
    //     matrix.insert(2, 2, ());
    //     matrix.insert(0, 3, ());
    //     matrix.insert(0, 25, ());
    //     assert_eq!(
    //         format!("{:#?}", matrix),
    //         r#"SymmetricBitvecAdjacencyMatrix {
    //         0
    //         10
    //         001
    //         1000
    //         00000
    //         00000 0
    //         00000 00
    //         00000 000
    //         00000 0000
    //         00000 00000
    //         00000 00000 0
    //         00000 00000 00
    //         00000 00000 000
    //         00000 00000 0000
    //         00000 00000 00000
    //         00000 00000 00000 0
    //         00000 00000 00000 00
    //         00000 00000 00000 000
    //         00000 00000 00000 0000
    //         00000 00000 00000 00000
    //         ...
    //     }
    //     "#
    //     );
    // }
}
