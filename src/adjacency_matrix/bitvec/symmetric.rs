use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use bitvec::vec::BitVec;

use crate::symmetric_maxtrix_indexing::SymmetricMatrixIndexing;
use crate::util::{euler_sum, euler_sum_inv_floor, sort_pair};

use crate::adjacency_matrix::{AdjacencyMatrix, BitvecStorage, Symmetric};

pub struct SymmetricBitvecAdjacencyMatrix<K, V> {
    data: Vec<MaybeUninit<V>>,
    matrix: BitVec,
    indexing: SymmetricMatrixIndexing,
    key: PhantomData<K>,
}

impl<K, V> SymmetricBitvecAdjacencyMatrix<K, V>
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq,
{
    pub fn with_size(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let indexing = SymmetricMatrixIndexing::new(capacity);
        let storage_size = indexing.storage_size();
        let mut matrix = BitVec::with_capacity(storage_size);
        matrix.resize(storage_size, false);
        let mut data = Vec::with_capacity(storage_size);
        data.resize_with(storage_size, MaybeUninit::uninit);
        Self {
            data,
            matrix,
            indexing,
            key: PhantomData,
        }
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
}

impl<V, K> AdjacencyMatrix for SymmetricBitvecAdjacencyMatrix<K, V>
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord,
{
    type Key = K;
    type Value = V;
    type Symmetry = Symmetric;
    type Storage = BitvecStorage;

    fn new() -> Self {
        Self {
            matrix: BitVec::new(),
            data: Vec::new(),
            indexing: SymmetricMatrixIndexing::new(0),
            key: PhantomData,
        }
    }

    fn insert(&mut self, from: K, into: K, data: V) -> Option<V> {
        let (k1, k2) = sort_pair(from.into(), into.into());
        if self.indexing.index(k1.into(), k2.into()).is_none() {
            let required_size = (k2 + 1).next_power_of_two();
            if self.indexing.storage_size() < required_size {
                let repr_size = euler_sum(required_size);
                self.matrix.resize(repr_size, false);
                self.data.resize_with(repr_size, MaybeUninit::uninit);
                self.indexing.resize(required_size);
            }
        }
        let index = self.indexing.unchecked_index(k1.into(), k2.into());
        let old_data = self.get_data_read(index);
        self.matrix.set(index, true);
        self.data[index] = MaybeUninit::new(data);
        old_data
    }

    fn get(&self, from: &K, into: &K) -> Option<&V> {
        self.get_data_ref(self.indexing.index((*from).into(), (*into).into())?)
    }

    fn remove(&mut self, from: &K, into: &K) -> Option<V> {
        let index = self.indexing.index((*from).into(), (*into).into())?;
        let was_live = self.is_live(index);
        self.matrix.set(index, false);
        was_live.then(|| self.unchecked_get_data_read(index))
    }

    fn edges<'a>(&'a self) -> impl Iterator<Item = (K, K, &'a V)>
    where
        V: 'a,
    {
        self.matrix.iter_ones().map(|index| {
            let (k1, k2) = self.indexing.coordinates(index);
            (k1.into(), k2.into(), self.unchecked_get_data_ref(index))
        })
    }

    fn edge_ends(k1: &Self::Key, k2: &Self::Key) -> (Self::Key, Self::Key) {
        sort_pair(k1.clone(), k2.clone())
    }

    fn edges_from<'a>(&'a self, from: &K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        let from = (*from).into();
        self.indexing.row(from).filter_map(move |index| {
            if self.is_live(index) {
                let (i, j) = self.indexing.coordinates(index);
                debug_assert!(i == from || j == from);
                Some((
                    if i == from { j.into() } else { i.into() },
                    self.unchecked_get_data_ref(index),
                ))
            } else {
                None
            }
        })
    }

    fn edges_into<'a>(&'a self, into: &K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        self.edges_from(into)
    }
}

impl<K, V> Debug for SymmetricBitvecAdjacencyMatrix<K, V>
where
    K: Into<usize>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Adjacency Matrix: {{")?;
        for i in 0..euler_sum_inv_floor(self.matrix.len()) {
            write!(f, "  ")?;
            if i > 20 {
                writeln!(f, "...")?;
                break;
            }
            for j in 0..=i {
                let index = euler_sum(i) + j;
                if self.matrix[index] {
                    write!(f, "1")?;
                } else {
                    write!(f, "0")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "}}")?;
        Ok(())
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
        matrix.insert(3,3, ());
        assert_eq!(matrix.edges_into(&2).collect::<Vec<_>>(), vec![(0, &()), (1, &())]);
        assert_eq!(matrix.edges_into(&3).collect::<Vec<_>>(), vec![(3, &())]);
    }

    #[test]
    fn test_edges_into2() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.edges_into(&1).collect::<Vec<_>>(), vec![(0, &())]);
        assert_eq!(matrix.edges_into(&0).collect::<Vec<_>>(), vec![(1, &())]);
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
        let mut matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
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
}
