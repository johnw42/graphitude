use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit};

use bitvec::vec::BitVec;
use tracing_subscriber::field::debug;

use crate::adjacency_matrix::{AdjacencyMatrix, Asymmetric, BitvecStorage};

/// Bitvec-based asymmetric adjacency matrix for directed graphs.
///
/// Uses a dense square matrix representation with a bitvec to track which entries exist.
/// Automatically resizes to the next power of two for efficient indexing.
/// Requires indices that can be converted to/from usize.
pub struct AsymmetricBitvecAdjacencyMatrix<I, V> {
    data: Vec<MaybeUninit<V>>,
    matrix: BitVec,
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
        let mut matrix = BitVec::with_capacity(capacity * capacity);
        matrix.resize(capacity * capacity, false);
        let mut data = Vec::with_capacity(capacity * capacity);
        data.resize_with(capacity * capacity, MaybeUninit::uninit);
        AsymmetricBitvecAdjacencyMatrix {
            data,
            matrix,
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
        self.matrix[index]
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
            matrix: BitVec::new(),
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
                    new_self.matrix[new_start..new_start + self.size]
                        .copy_from_bitslice(&self.matrix[old_start..old_start + self.size]);
                    for (old_col, old_datum) in self.data[old_start..old_start + self.size]
                        .iter_mut()
                        .enumerate()
                    {
                        std::mem::swap(old_datum, &mut new_self.data[new_start + old_col]);
                    }
                }
                new_self.entry_count = self.entry_count;
                *self = new_self;
            }
        }
        let index = self.unchecked_index(row, col);
        let old_data = self.get_data_read(index);
        self.matrix.set(index, true);
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
        self.matrix.set(index, false);
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
        self.matrix.iter_ones().map(|index| {
            let (row, col) = self.coordinates(index);
            (row, col, self.unchecked_get_data_ref(index))
        })
    }

    fn into_iter(self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        let Self {
            matrix, data, size, ..
        } = self;
        matrix
            .into_iter()
            .enumerate()
            .filter_map(move |(index, bit)| {
                if bit {
                    let (row, col) = Self::coordinates_with(size, index);
                    Some((row, col, unsafe { data[index].assume_init_read() }))
                } else {
                    None
                }
            })
    }

    fn entries_in_row(&self, row: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let row_start = self.index(row, 0.into()).expect("Invalid row index");
        let row_end = row_start + self.size;
        self.matrix[row_start..row_end]
            .iter_ones()
            .map(move |index| (index.into(), self.unchecked_get_data_ref(row_start + index)))
    }

    fn entries_in_col(&self, col: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        (0..self.size).filter_map(move |target_row| {
            self.get(target_row.into(), col)
                .map(|data| (target_row.into(), data))
        })
    }

    fn clear(&mut self) {
        self.matrix.fill(false);
        self.entry_count = 0;
    }

    fn len(&self) -> usize {
        self.entry_count
    }

    fn reserve(&mut self, capacity: usize) {
        if self.size < capacity {
            let mut new_self = Self::with_size(capacity);
            for old_row in 0..self.size {
                let old_start = self.unchecked_index(old_row.into(), 0.into());
                let new_start = new_self.unchecked_index(old_row.into(), 0.into());
                new_self.matrix[new_start..new_start + self.size]
                    .copy_from_bitslice(&self.matrix[old_start..old_start + self.size]);
                for (old_col, old_datum) in self.data[old_start..old_start + self.size]
                    .iter_mut()
                    .enumerate()
                {
                    std::mem::swap(old_datum, &mut new_self.data[new_start + old_col]);
                }
            }
            new_self.entry_count = self.entry_count;
            *self = new_self;
        }
    }
}

#[cfg(test)]
mod tests {
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
}
