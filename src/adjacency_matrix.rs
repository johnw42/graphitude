use std::{collections::HashMap, hash::Hash};

use bitvec::vec::BitVec;

use crate::util::sort_pair;

pub trait AdjacencyMatrix
where
    Self::Key: Hash + Eq + Clone,
{
    type Key;
    type Value;

    /// Creates a new, empty adjacency matrix.
    fn new() -> Self;

    /// Inserts an edge from `from` to `into` with associated data `data`.
    /// Returns the previous data associated with the edge, if any.
    fn insert(
        &mut self,
        from: Self::Key,
        into: Self::Key,
        data: Self::Value,
    ) -> Option<Self::Value>;

    /// Gets a reference to the data associated with the edge from `from` to `into`, if it exists.
    fn get(&self, from: &Self::Key, into: &Self::Key) -> Option<&Self::Value>;

    /// Removes the edge from `from` to `into`, returning the associated data if it existed.
    fn remove(&mut self, from: &Self::Key, into: &Self::Key) -> Option<Self::Value>;

    /// Iterates over all edges in the adjacency matrix.
    fn edges<'a>(&'a self) -> impl Iterator<Item = (Self::Key, Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a;

    /// Iterates over all edges originating from the given vertex `from`.
    fn edges_from<'a>(
        &'a self,
        from: &Self::Key,
    ) -> impl Iterator<Item = (Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a;

    /// Iterates over all edges terminating at the given vertex `into`.
    fn edges_into<'a>(
        &'a self,
        into: &Self::Key,
    ) -> impl Iterator<Item = (Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a;
}

/// An adjacency matrix that treats edges as undirected by storing each edge
/// in a symmetric manner.
#[derive(Clone, Debug)]
pub struct Symmetric<M>(M);

impl<M> AdjacencyMatrix for Symmetric<M>
where
    M: AdjacencyMatrix,
    M::Key: Ord,
{
    type Key = M::Key;
    type Value = M::Value;

    fn new() -> Self {
        Symmetric(M::new())
    }

    fn insert(
        &mut self,
        from: Self::Key,
        into: Self::Key,
        data: Self::Value,
    ) -> Option<Self::Value> {
        let (k1, k2) = sort_pair(from, into);
        self.0.insert(k1, k2, data)
    }

    fn get(&self, from: &Self::Key, into: &Self::Key) -> Option<&Self::Value> {
        let (k1, k2) = sort_pair(from, into);
        self.0.get(k1, k2)
    }

    fn remove(&mut self, from: &Self::Key, into: &Self::Key) -> Option<Self::Value> {
        let (k1, k2) = sort_pair(from.clone(), into.clone());
        self.0.remove(&k1, &k2)
    }

    fn edges<'a>(&'a self) -> impl Iterator<Item = (Self::Key, Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a,
    {
        self.0.edges()
    }

    fn edges_from<'a>(
        &'a self,
        from: &Self::Key,
    ) -> impl Iterator<Item = (Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a,
    {
        self.0.edges_from(from).chain(self.0.edges_into(from))
    }

    fn edges_into<'a>(
        &'a self,
        into: &Self::Key,
    ) -> impl Iterator<Item = (Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a,
    {
        self.edges_from(into)
    }
}

#[derive(Clone, Debug)]
pub struct AsymmetricAdjacencyMatrix<K, V> {
    edges: HashMap<K, HashMap<K, V>>,
}

impl<K, V> AdjacencyMatrix for AsymmetricAdjacencyMatrix<K, V>
where
    K: Hash + Eq + Clone,
{
    type Key = K;
    type Value = V;

    fn new() -> Self {
        AsymmetricAdjacencyMatrix {
            edges: HashMap::new(),
        }
    }

    fn insert(&mut self, from: K, into: K, data: V) -> Option<V> {
        self.edges.entry(from).or_default().insert(into, data)
    }

    fn get(&self, from: &K, into: &K) -> Option<&V> {
        self.edges.get(from).and_then(|m| m.get(into))
    }

    fn remove(&mut self, from: &K, into: &K) -> Option<V> {
        self.edges.get_mut(from).and_then(|m| m.remove(into))
    }

    fn edges<'a>(&'a self) -> impl Iterator<Item = (K, K, &'a V)>
    where
        V: 'a,
    {
        self.edges.iter().flat_map(|(from, targets)| {
            targets
                .iter()
                .map(move |(into, data)| (from.clone(), into.clone(), data))
        })
    }

    fn edges_from<'a>(&'a self, from: &K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        self.edges
            .get(from)
            .into_iter()
            .flat_map(|targets| targets.iter().map(move |(into, data)| (into.clone(), data)))
    }

    fn edges_into<'a>(&'a self, into: &K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        self.edges
            .iter()
            .filter_map(move |(from, targets)| targets.get(into).map(|data| (from.clone(), data)))
    }
}

pub type SymmetricAdjacencyMatrix<K, V> = Symmetric<AsymmetricAdjacencyMatrix<K, V>>;

pub struct AsymmetricBitvecAdjacencyMatrix {
    matrix: BitVec<usize>,
    size: usize,
    log2_size: u32,
}

impl AsymmetricBitvecAdjacencyMatrix {
    fn with_size(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let mut matrix = BitVec::with_capacity(capacity * capacity);
        matrix.resize(capacity * capacity, false);
        AsymmetricBitvecAdjacencyMatrix {
            matrix,
            size: capacity,
            log2_size: capacity.trailing_zeros(),
        }
    }

    fn index(&self, from: usize, into: usize) -> Option<usize> {
        (from < self.size && into < self.size).then(|| self.unchecked_index(from, into))
    }

    fn unchecked_index(&self, from: usize, into: usize) -> usize {
        (from << self.log2_size) + into
    }

    fn coordinates(&self, index: usize) -> (usize, usize) {
        let from = index >> self.log2_size;
        let into = index & (self.size - 1);
        (from, into)
    }
}

impl AdjacencyMatrix for AsymmetricBitvecAdjacencyMatrix {
    type Key = usize;
    type Value = ();

    fn new() -> Self {
        AsymmetricBitvecAdjacencyMatrix {
            matrix: BitVec::new(),
            size: 0,
            log2_size: 0,
        }
    }

    fn insert(&mut self, from: usize, into: usize, _data: ()) -> Option<()> {
        let result = self.get(&from, &into).map(|_| ());
        let required_size = usize::max(from, into) + 1;
        if self.size < required_size {
            let mut new_self = Self::with_size(required_size);
            for row in 0..self.size {
                let old_start = self.unchecked_index(row, 0);
                let new_start = new_self.unchecked_index(row, 0);
                new_self.matrix[new_start..new_start + self.size]
                    .copy_from_bitslice(&self.matrix[old_start..old_start + self.size]);
            }
            *self = new_self;
        }
        let index = self.unchecked_index(from, into);
        self.matrix.set(index, true);
        result
    }

    fn get(&self, from: &usize, into: &usize) -> Option<&()> {
        self.index(*from, *into)
            .and_then(|idx| self.matrix[idx].then_some(&()))
    }

    fn remove(&mut self, from: &usize, into: &usize) -> Option<()> {
        self.index(*from, *into).and_then(|idx| {
            let result = self.matrix[idx].then_some(());
            self.matrix.set(idx, false);
            result
        })
    }

    fn edges<'a>(&'a self) -> impl Iterator<Item = (usize, usize, &'a ())>
    where
        (): 'a,
    {
        self.matrix.iter_ones().map(|idx| {
            let (from, into) = self.coordinates(idx);
            (from, into, &())
        })
    }

    fn edges_from<'a>(&'a self, from: &usize) -> impl Iterator<Item = (usize, &'a ())>
    where
        (): 'a,
    {
        let row_start = self.index(*from, 0).expect("Invalid 'from' index");
        let row_end = row_start + self.size;
        self.matrix[row_start..row_end]
            .iter_ones()
            .map(|idx| (idx, &()))
    }

    fn edges_into<'a>(&'a self, into: &usize) -> impl Iterator<Item = (usize, &'a ())>
    where
        (): 'a,
    {
        (0..self.size).filter_map(move |from| self.get(&from, into).map(|data| (from, data)))
    }
}

pub type SymmetricBitvecAdjacencyMatrix = Symmetric<AsymmetricBitvecAdjacencyMatrix>;

// TODO: Review!
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asymmetric_matrix_insert_and_get() {
        let mut matrix = AsymmetricAdjacencyMatrix::new();
        assert_eq!(matrix.insert(0, 1, "edge"), None);
        assert_eq!(matrix.get(&0, &1), Some(&"edge"));
        assert_eq!(matrix.get(&1, &0), None);
    }

    #[test]
    fn test_asymmetric_matrix_insert_overwrites() {
        let mut matrix = AsymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "first");
        assert_eq!(matrix.insert(0, 1, "second"), Some("first"));
        assert_eq!(matrix.get(&0, &1), Some(&"second"));
    }

    #[test]
    fn test_asymmetric_matrix_remove() {
        let mut matrix = AsymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.remove(&0, &1), Some("edge"));
        assert_eq!(matrix.get(&0, &1), None);
    }

    #[test]
    fn test_asymmetric_matrix_remove_nonexistent() {
        let mut matrix: AsymmetricAdjacencyMatrix<usize, &str> = AsymmetricAdjacencyMatrix::new();
        assert_eq!(matrix.remove(&0, &1), None);
    }

    #[test]
    fn test_symmetric_matrix_insert_and_get() {
        let mut matrix = SymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.get(&0, &1), Some(&"edge"));
        assert_eq!(matrix.get(&1, &0), Some(&"edge"));
    }

    #[test]
    fn test_symmetric_matrix_remove() {
        let mut matrix = SymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.remove(&1, &0), Some("edge"));
        assert_eq!(matrix.get(&0, &1), None);
    }

    #[test]
    fn test_bitvec_insert_and_get() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        assert_eq!(matrix.insert(0, 1, ()), None);
        assert_eq!(matrix.get(&0, &1), Some(&()));
        assert_eq!(matrix.get(&1, &0), None);
    }

    #[test]
    fn test_bitvec_insert_duplicate() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        assert_eq!(matrix.insert(0, 1, ()), None);
        assert_eq!(matrix.insert(0, 1, ()), Some(()));
    }

    #[test]
    fn test_bitvec_remove() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.remove(&0, &1), Some(()));
        assert_eq!(matrix.get(&0, &1), None);
    }

    #[test]
    fn test_symmetric_bitvec_insert_and_get() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.get(&0, &1), Some(&()));
        assert_eq!(matrix.get(&1, &0), Some(&()));
    }

    #[test]
    fn test_asymmetric_matrix_edges() {
        let mut matrix = AsymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(1, 0, "b");
        let edges: Vec<_> = matrix.edges().collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_bitvec_edges() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(2, 3, ());
        let edges: Vec<_> = matrix.edges().collect();
        assert_eq!(edges.len(), 2);
    }
    #[test]
    fn test_asymmetric_matrix_edges_from() {
        let mut matrix = AsymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(0, 2, "b");
        matrix.insert(1, 2, "c");
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(to, _)| *to == 1));
        assert!(edges.iter().any(|(to, _)| *to == 2));
    }

    #[test]
    fn test_asymmetric_matrix_edges_into() {
        let mut matrix = AsymmetricAdjacencyMatrix::new();
        matrix.insert(0, 2, "a");
        matrix.insert(1, 2, "b");
        matrix.insert(3, 2, "c");
        let edges: Vec<_> = matrix.edges_into(&2).collect();
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().any(|(from, _)| *from == 0));
        assert!(edges.iter().any(|(from, _)| *from == 1));
        assert!(edges.iter().any(|(from, _)| *from == 3));
    }

    #[test]
    fn test_symmetric_matrix_edges_from() {
        let mut matrix = SymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(0, 2, "b");
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_symmetric_matrix_edges_into() {
        let mut matrix = SymmetricAdjacencyMatrix::new();
        matrix.insert(0, 2, "a");
        matrix.insert(1, 2, "b");
        let edges: Vec<_> = matrix.edges_into(&2).collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_bitvec_edges_from() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(0, 3, ());
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(to, _)| *to == 1));
        assert!(edges.iter().any(|(to, _)| *to == 3));
    }

    #[test]
    fn test_bitvec_edges_into() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 2, ());
        matrix.insert(1, 2, ());
        matrix.insert(3, 2, ());
        let edges: Vec<_> = matrix.edges_into(&2).collect();
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().any(|(from, _)| *from == 0));
        assert!(edges.iter().any(|(from, _)| *from == 1));
        assert!(edges.iter().any(|(from, _)| *from == 3));
    }

    #[test]
    fn test_symmetric_bitvec_edges_from() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(0, 2, ());
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_symmetric_bitvec_edges_into() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 2, ());
        matrix.insert(1, 2, ());
        let edges: Vec<_> = matrix.edges_into(&2).collect();
        assert_eq!(edges.len(), 2);
    }
}
