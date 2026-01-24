use std::mem::MaybeUninit;

use bitvec::vec::BitVec;

use crate::util::{euler_sum, euler_sum_inv_floor, sort_pair};

use crate::adjacency_matrix::AdjacencyMatrix;

pub struct SymmetricBitvecAdjacencyMatrix<V> {
    data: Vec<MaybeUninit<V>>,
    matrix: BitVec,
    size: usize,
}

impl<V> SymmetricBitvecAdjacencyMatrix<V> {
    pub fn with_size(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let repr_size = euler_sum(capacity);
        let mut matrix = BitVec::with_capacity(repr_size);
        matrix.resize(repr_size, false);
        let mut data = Vec::with_capacity(repr_size);
        data.resize_with(repr_size, MaybeUninit::uninit);
        Self {
            data,
            matrix,
            size: capacity,
        }
    }

    /// Gets the linear index for the edge from `from` to `into`, if within bounds.
    fn index(&self, from: usize, into: usize) -> Option<usize> {
        (from < self.size && into < self.size).then(|| self.unchecked_index(from, into))
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

    /// Gets the linear index for the edge from `from` to `into` without bounds checking.
    fn unchecked_index(&self, from: usize, into: usize) -> usize {
        let (k1, k2) = sort_pair(from, into);
        euler_sum(k2) + k1
    }

    fn coordinates(&self, index: usize) -> (usize, usize) {
        let x = euler_sum_inv_floor(index);
        let y = index - euler_sum(x);
        debug_assert!(x >= y);
        (y, x)
    }
}

impl<V> AdjacencyMatrix for SymmetricBitvecAdjacencyMatrix<V> {
    type Key = usize;
    type Value = V;

    fn new() -> Self {
        Self {
            matrix: BitVec::new(),
            size: 0,
            data: Vec::new(),
        }
    }

    fn insert(&mut self, from: usize, into: usize, data: V) -> Option<V> {
        let (k1, k2) = sort_pair(from, into);
        if self.index(k1, k2).is_none() {
            let required_size = (k2 + 1).next_power_of_two();
            if self.size < required_size {
                let repr_size = euler_sum(required_size);
                self.matrix.resize(repr_size, false);
                self.data.resize_with(repr_size, MaybeUninit::uninit);
                self.size = required_size;
            }
        }
        let index = self.unchecked_index(k1, k2);
        let old_data = self.get_data_read(index);
        self.matrix.set(index, true);
        self.data[index] = MaybeUninit::new(data);
        old_data
    }

    fn get(&self, from: &usize, into: &usize) -> Option<&V> {
        self.get_data_ref(self.index(*from, *into)?)
    }

    fn remove(&mut self, from: &usize, into: &usize) -> Option<V> {
        let index = self.index(*from, *into)?;
        let was_live = self.is_live(index);
        self.matrix.set(index, false);
        was_live.then(|| self.unchecked_get_data_read(index))
    }

    fn edges<'a>(&'a self) -> impl Iterator<Item = (usize, usize, &'a V)>
    where
        V: 'a,
    {
        self.matrix.iter_ones().map(|index| {
            let (k1, k2) = self.coordinates(index);
            (k1, k2, self.unchecked_get_data_ref(index))
        })
    }

    fn edges_from<'a>(&'a self, from: &usize) -> impl Iterator<Item = (usize, &'a V)>
    where
        V: 'a,
    {
        let row_start = self.index(*from, 0).expect("Invalid 'from' index");
        let row_end = row_start + self.size;
        self.matrix[row_start..row_end]
            .iter_ones()
            .map(|index| (index, self.unchecked_get_data_ref(index)))
    }

    fn edges_into<'a>(&'a self, into: &usize) -> impl Iterator<Item = (usize, &'a V)>
    where
        V: 'a,
    {
        self.edges_from(into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.get(&0, &1), Some(&()));
        assert_eq!(matrix.get(&1, &0), Some(&()));
    }

    #[test]
    fn test_edges_from() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(0, 2, ());
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_edges_into() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 2, ());
        matrix.insert(1, 2, ());
        let edges: Vec<_> = matrix.edges_into(&2).collect();
        assert_eq!(edges.len(), 2);
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
        assert_eq!(matrix.remove(&0, &1), Some(()));
        assert_eq!(matrix.get(&0, &1), None);
    }

    #[test]
    fn test_remove_both_directions() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.remove(&1, &0), Some(()));
        assert_eq!(matrix.get(&0, &1), None);
        assert_eq!(matrix.get(&1, &0), None);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::<()>::new();
        assert_eq!(matrix.remove(&0, &1), None);
    }

    #[test]
    fn test_edges() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(3, 2, "b");
        let mut edges: Vec<_> = matrix.edges().collect();
        edges.sort();
        assert_eq!(edges, vec![(0, 1, &"a"), (2, 3, &"b")]);
    }

    #[test]
    fn test_large_indices() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(100, 200, ());
        assert_eq!(matrix.get(&100, &200), Some(&()));
        assert_eq!(matrix.get(&200, &100), Some(&()));
    }

    #[test]
    fn test_self_loop() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(5, 5, ());
        assert_eq!(matrix.get(&5, &5), Some(&()));
    }

    #[test]
    fn test_coordinates() {
        let matrix = SymmetricBitvecAdjacencyMatrix::<()>::with_size(8);
        for index in 0..matrix.matrix.len() {
            let (k1, k2) = matrix.coordinates(index);
            let computed_index = matrix.unchecked_index(k1, k2);
            assert_eq!(index, computed_index, "Failed at index {}", index);
        }
    }
}
