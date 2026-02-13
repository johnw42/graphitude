#![cfg(feature = "bitvec")]

use std::ops::{Add, Index, IndexMut, Range, Sub};

use bitvec::slice::BitSlice;

use crate::{
    DirectednessTrait,
    triangular::{triangular, triangular_inv_floor},
    util::sort_pair,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataIndex(pub usize);

impl<T> Index<DataIndex> for Vec<T> {
    type Output = T;

    fn index(&self, index: DataIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl<T> IndexMut<DataIndex> for Vec<T> {
    fn index_mut(&mut self, index: DataIndex) -> &mut Self::Output {
        &mut self[index.0]
    }
}

impl Add<usize> for DataIndex {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        DataIndex(self.0 + rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LivenessIndex(pub usize);

impl Index<LivenessIndex> for BitSlice {
    type Output = bool;

    fn index(&self, index: LivenessIndex) -> &Self::Output {
        &self[index.0]
    }
}

impl Add<usize> for LivenessIndex {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        LivenessIndex(self.0 + rhs)
    }
}

impl Sub for LivenessIndex {
    type Output = usize;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

/// Utilities for indexing into symmetric matrices stored in a flat array.  Only
/// entries where `column <= row` are stored, so for, for example, for a 4×4
/// symmetric matrix stored in a flat array, the entries are stored at the
/// following indices in an array of length 10:
///
/// ```text
/// ⎛ 0 1 3 6 ⎞
/// ⎟ 1 2 4 8 ⎟
/// ⎟ 3 4 5 8 ⎟
/// ⎝ 6 8 8 9 ⎠
/// ```
///
/// Note that the indices corresponding to row/column 0 for an n×n matrix are
/// the triangular numbers: 0, 1, 3, 6, 10, 15, etc., and the indices of the
/// diagonal are one less than a trigangular number.
pub(crate) struct MatrixIndexing<D> {
    /// The size of one dimension of the symmetric matrix.
    size: usize,
    directedness: D,
}

impl<D> MatrixIndexing<D>
where
    D: DirectednessTrait,
{
    /// Creates a new `MatrixIndexing` for a symmetric matrix of the given size.
    pub fn new(size: usize, directedness: D) -> Self {
        Self { size, directedness }
    }

    /// Returns the size of one dimension of the symmetric matrix.
    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the storage size required for the symmetric matrix.
    pub fn data_storage_size(&self) -> usize {
        if self.directedness.is_directed() {
            self.size * self.size
        } else {
            triangular(self.size)
        }
    }

    /// Returns the storage size required for the symmetric matrix.
    pub fn liveness_storage_size(&self) -> usize {
        self.size * self.size
    }

    /// Returns the linear index row `i` and column `j`, if within bounds.
    pub fn data_index(&self, i: usize, j: usize) -> Option<DataIndex> {
        (i < self.size && j < self.size).then(|| self.unchecked_data_index(i, j))
    }

    /// Returns the linear index row `i` and column `j` without bounds checking.
    pub fn unchecked_data_index(&self, i: usize, j: usize) -> DataIndex {
        if self.directedness.is_directed() {
            DataIndex(i * self.size + j)
        } else {
            let (k1, k2) = sort_pair(i, j);
            DataIndex(triangular(k2) + k1)
        }
    }

    pub fn liveness_index(&self, i: usize, j: usize) -> Option<LivenessIndex> {
        (i < self.size && j < self.size).then(|| self.unchecked_liveness_index(i, j))
    }

    pub fn unchecked_liveness_index(&self, i: usize, j: usize) -> LivenessIndex {
        LivenessIndex(i * self.size + j)
    }

    /// Returns the `(column, row)` coordinates corresponding to the given
    /// index, where `column <= row`.
    #[allow(unused)]
    pub fn data_coordinates(&self, index: DataIndex) -> (usize, usize) {
        if self.directedness.is_directed() {
            self.liveness_coordinates(LivenessIndex(index.0))
        } else {
            let row = triangular_inv_floor(index.0);
            let col = index.0 - triangular(row);
            sort_pair(col, row)
        }
    }

    pub fn liveness_coordinates(&self, index: LivenessIndex) -> (usize, usize) {
        let size = self.size;
        let row = index.0 / size;
        let col = index.0 % size;
        (row, col)
    }

    pub fn liveness_index_to_data_index(&self, liveness_index: LivenessIndex) -> DataIndex {
        if self.directedness.is_directed() {
            DataIndex(liveness_index.0)
        } else {
            let (row, col) = self.liveness_coordinates(liveness_index);
            self.unchecked_data_index(row, col)
        }
    }

    #[allow(dead_code)]
    pub fn data_row_range(&self, i: usize) -> Range<DataIndex> {
        let start = self.unchecked_data_index(i, 0);
        let end = self.unchecked_data_index(i + 1, 0);
        start..end
    }

    #[allow(dead_code)]
    pub fn liveness_row_range(&self, i: usize) -> Range<LivenessIndex> {
        let start = self.unchecked_liveness_index(i, 0);
        let end = self.unchecked_liveness_index(i + 1, 0);
        start..end
    }

    /// Returns an iterator over the indices in row `i` of the symmetric matrix.
    #[allow(dead_code)]
    pub fn data_row(&self, i: usize) -> impl Iterator<Item = DataIndex> + '_ {
        let (_range, iter) = self.data_row_with_range(i);
        iter
    }

    /// Returns a tuple containing the range of indices for row `i` and an
    /// iterator over the remaining indices in that row.
    #[allow(dead_code)]
    pub fn data_row_with_range(
        &self,
        i: usize,
    ) -> (Range<DataIndex>, impl Iterator<Item = DataIndex> + '_) {
        let start = self.unchecked_data_index(i, 0);
        let end = self.unchecked_data_index(i + 1, 0);
        (
            start..end,
            (i..self.size).map(move |j| self.unchecked_data_index(i, j)),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Undirected;

    use super::*;

    #[test]
    fn test_new() {
        let smi = MatrixIndexing::new(5, Undirected);
        assert_eq!(smi.size, 5);
    }
    #[test]
    fn test_size() {
        let smi = MatrixIndexing::new(7, Undirected);
        assert_eq!(smi.size(), 7);
    }

    #[test]
    fn test_index_valid() {
        let smi = MatrixIndexing::new(4, Undirected);
        assert!(smi.data_index(0, 0).is_some());
        assert!(smi.data_index(2, 3).is_some());
        assert!(smi.data_index(3, 2).is_some());
    }

    #[test]
    fn test_index_out_of_bounds() {
        let smi = MatrixIndexing::new(4, Undirected);
        assert!(smi.data_index(4, 0).is_none());
        assert!(smi.data_index(0, 4).is_none());
        assert!(smi.data_index(5, 5).is_none());
    }

    #[test]
    fn test_symmetry() {
        let smi = MatrixIndexing::new(5, Undirected);
        for i in 0..5 {
            for j in 0..5 {
                assert_eq!(smi.data_index(i, j), smi.data_index(j, i));
            }
        }
    }

    #[test]
    fn test_unchecked_index_diagonal() {
        let smi = MatrixIndexing::new(5, Undirected);
        assert_eq!(smi.unchecked_data_index(0, 0), DataIndex(0));
        assert_eq!(smi.unchecked_data_index(1, 1), DataIndex(2));
        assert_eq!(smi.unchecked_data_index(2, 2), DataIndex(5));
    }

    #[test]
    fn test_coordinates_roundtrip() {
        let smi = MatrixIndexing::new(6, Undirected);
        for i in 0..6 {
            for j in 0..6 {
                let idx = smi.unchecked_data_index(i, j);
                let (col, row) = smi.data_coordinates(idx);
                debug_assert!(col <= row);
                assert_eq!(smi.unchecked_data_index(col, row), idx);
            }
        }
    }

    #[test]
    fn test_row_iterator() {
        let smi = MatrixIndexing::new(4, Undirected);
        let row_0: Vec<_> = smi
            .data_row(0)
            .map(|index| smi.data_coordinates(index))
            .collect();
        assert_eq!(
            row_0,
            [(0, 0), (0, 1), (0, 2), (0, 3)]
                .into_iter()
                .collect::<Vec<_>>()
        );

        let row_2: Vec<_> = smi
            .data_row(2)
            .map(|index| smi.data_coordinates(index))
            .collect();
        assert_eq!(
            row_2,
            [(0, 2), (1, 2), (2, 2), (2, 3)]
                .into_iter()
                .collect::<Vec<_>>()
        );

        // Verify that the row iterator produces correct indices
        // according to a more straightforward method.
        for i in 0..smi.size() {
            assert_eq!(
                smi.data_row(i).collect::<Vec<_>>(),
                (0..smi.size())
                    .map(|j| smi.data_index(i, j).unwrap())
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_row_symmetry() {
        let smi = MatrixIndexing::new(5, Undirected);
        for i in 0..5 {
            let row: Vec<_> = smi.data_row(i).collect();
            for (j, &idx) in row.iter().enumerate() {
                assert_eq!(idx, smi.data_index(i, j).unwrap());
            }
        }
    }
}
