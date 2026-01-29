use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit};

use bitvec::vec::BitVec;

use crate::adjacency_matrix::{AdjacencyMatrix, Asymmetric, BitvecStorage};

pub struct AsymmetricBitvecAdjacencyMatrix<K, V> {
    data: Vec<MaybeUninit<V>>,
    matrix: BitVec,
    size: usize,
    log2_size: u32,
    key: PhantomData<K>,
}

impl<K, V> AsymmetricBitvecAdjacencyMatrix<K, V>
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq + Hash,
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
            log2_size: capacity.trailing_zeros(),
            key: PhantomData,
        }
    }

    /// Gets the linear index for the entry at `row` and `col`, if within bounds.
    fn index(&self, row: K, col: K) -> Option<usize> {
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
    fn unchecked_index(&self, row: K, col: K) -> usize {
        (row.into() << self.log2_size) + col.into()
    }

    fn coordinates(&self, index: usize) -> (K, K) {
        let row = index >> self.log2_size;
        let col = index & (self.size - 1);
        (row.into(), col.into())
    }
}

impl<K, V> AdjacencyMatrix for AsymmetricBitvecAdjacencyMatrix<K, V>
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq + Hash,
{
    type Key = K;
    type Value = V;
    type Symmetry = Asymmetric;
    type Storage = BitvecStorage;

    fn new() -> Self {
        Self {
            matrix: BitVec::new(),
            size: 0,
            log2_size: 0,
            data: Vec::new(),
            key: PhantomData,
        }
    }

    fn insert(&mut self, row: K, col: K, data: V) -> Option<V> {
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
                *self = new_self;
            }
        }
        let index = self.unchecked_index(row, col);
        let old_data = self.get_data_read(index);
        self.matrix.set(index, true);
        self.data[index] = MaybeUninit::new(data);
        old_data
    }

    fn get(&self, row: K, col: K) -> Option<&V> {
        self.get_data_ref(self.index(row, col)?)
    }

    fn remove(&mut self, row: K, col: K) -> Option<V> {
        let index = self.index(row, col)?;
        let was_live = self.is_live(index);
        self.matrix.set(index, false);
        was_live.then(|| self.unchecked_get_data_read(index))
    }

    fn entries<'a>(&'a self) -> impl Iterator<Item = (K, K, &'a V)>
    where
        V: 'a,
    {
        self.matrix.iter_ones().map(|index| {
            let (row, col) = self.coordinates(index);
            (row, col, self.unchecked_get_data_ref(index))
        })
    }

    fn entries_in_row<'a>(&'a self, row: K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        let row_start = self.index(row, 0.into()).expect("Invalid row index");
        let row_end = row_start + self.size;
        self.matrix[row_start..row_end]
            .iter_ones()
            .map(|index| (index.into(), self.unchecked_get_data_ref(index.into())))
    }

    fn entries_in_col<'a>(&'a self, col: K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        let (_, target_col) = self.coordinates(col.into());
        (0..self.size).filter_map(move |target_row| {
            self.get(target_row.into(), target_col)
                .map(|data| (target_row.into(), data))
        })
    }

    fn clear(&mut self) {
        self.matrix.fill(false);
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
        let edges: Vec<_> = matrix.entries().collect();
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
}
