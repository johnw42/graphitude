#![cfg(feature = "bitvec")]

use std::ops::Range;

use crate::{
    pairs::SortedPair,
    triangular::{triangular, triangular_inv_floor},
    util::sort_pair,
};

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
pub(crate) struct SymmetricMatrixIndexing {
    /// The size of one dimension of the symmetric matrix.
    size: usize,
}

impl SymmetricMatrixIndexing {
    /// Creates a new `SymmetricMatrixIndexing` for a symmetric matrix of the given size.
    pub fn new(size: usize) -> Self {
        Self { size }
    }

    /// Returns the size of one dimension of the symmetric matrix.
    #[allow(unused)]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the storage size required for the symmetric matrix.
    pub fn storage_size(&self) -> usize {
        triangular(self.size)
    }

    /// Returns the linear index row `i` and column `j`, if within bounds.
    pub fn index(&self, i: usize, j: usize) -> Option<usize> {
        (i < self.size && j < self.size).then(|| self.unchecked_index(i, j))
    }

    /// Returns the linear index row `i` and column `j` without bounds checking.
    pub fn unchecked_index(&self, i: usize, j: usize) -> usize {
        let (k1, k2) = sort_pair(i, j);
        triangular(k2) + k1
    }

    /// Returns the `(column, row)` coordinates corresponding to the given
    /// index, where `column <= row`.
    #[allow(dead_code)]
    pub fn coordinates(&self, index: usize) -> SortedPair<usize> {
        let row = triangular_inv_floor(index);
        let col = index - triangular(row);
        SortedPair::from_sorted(col, row)
    }

    /// Returns an iterator over the indices in row `i` of the symmetric matrix.
    #[allow(dead_code)]
    pub fn row(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        let (range, iter) = self.row_with_range(i);
        range.chain(iter)
    }

    /// Returns a tuple containing the range of indices for row `i` and an
    /// iterator over the remaining indices in that row.
    #[allow(dead_code)]
    pub fn row_with_range(&self, i: usize) -> (Range<usize>, impl Iterator<Item = usize> + '_) {
        let start = self.index(i, 0).unwrap();
        let end = self.index(i, i).unwrap();
        (
            start..end,
            (i..self.size).map(move |j| self.index(i, j).unwrap()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::pairs::PairTrait;

    use super::*;

    #[test]
    fn test_new() {
        let smi = SymmetricMatrixIndexing::new(5);
        assert_eq!(smi.size, 5);
    }
    #[test]
    fn test_size() {
        let smi = SymmetricMatrixIndexing::new(7);
        assert_eq!(smi.size(), 7);
    }

    #[test]
    fn test_index_valid() {
        let smi = SymmetricMatrixIndexing::new(4);
        assert!(smi.index(0, 0).is_some());
        assert!(smi.index(2, 3).is_some());
        assert!(smi.index(3, 2).is_some());
    }

    #[test]
    fn test_index_out_of_bounds() {
        let smi = SymmetricMatrixIndexing::new(4);
        assert!(smi.index(4, 0).is_none());
        assert!(smi.index(0, 4).is_none());
        assert!(smi.index(5, 5).is_none());
    }

    #[test]
    fn test_symmetry() {
        let smi = SymmetricMatrixIndexing::new(5);
        for i in 0..5 {
            for j in 0..5 {
                assert_eq!(smi.index(i, j), smi.index(j, i));
            }
        }
    }

    #[test]
    fn test_unchecked_index_diagonal() {
        let smi = SymmetricMatrixIndexing::new(5);
        assert_eq!(smi.unchecked_index(0, 0), 0);
        assert_eq!(smi.unchecked_index(1, 1), 2);
        assert_eq!(smi.unchecked_index(2, 2), 5);
    }

    #[test]
    fn test_coordinates_roundtrip() {
        let smi = SymmetricMatrixIndexing::new(6);
        for i in 0..6 {
            for j in 0..6 {
                let idx = smi.unchecked_index(i, j);
                let (col, row) = smi.coordinates(idx).into_values();
                debug_assert!(col <= row);
                assert_eq!(smi.unchecked_index(col, row), idx);
            }
        }
    }

    #[test]
    fn test_row_iterator() {
        let smi = SymmetricMatrixIndexing::new(4);
        let row_0: Vec<_> = smi.row(0).map(|index| smi.coordinates(index)).collect();
        assert_eq!(
            row_0,
            [(0, 0), (0, 1), (0, 2), (0, 3)]
                .into_iter()
                .map(SortedPair::from)
                .collect::<Vec<_>>()
        );

        let row_2: Vec<_> = smi.row(2).map(|index| smi.coordinates(index)).collect();
        assert_eq!(
            row_2,
            [(0, 2), (1, 2), (2, 2), (2, 3)]
                .into_iter()
                .map(SortedPair::from)
                .collect::<Vec<_>>()
        );

        // Verify that the row iterator produces correct indices
        // according to a more straightforward method.
        for i in 0..smi.size() {
            assert_eq!(
                smi.row(i).collect::<Vec<_>>(),
                (0..smi.size())
                    .map(|j| smi.index(i, j).unwrap())
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn test_row_symmetry() {
        let smi = SymmetricMatrixIndexing::new(5);
        for i in 0..5 {
            let row: Vec<_> = smi.row(i).collect();
            for (j, &idx) in row.iter().enumerate() {
                assert_eq!(idx, smi.index(i, j).unwrap());
            }
        }
    }
}
