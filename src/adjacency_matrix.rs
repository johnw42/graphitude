use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    mem::MaybeUninit,
};

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

#[derive(Clone, Debug)]
pub struct SymmetricAdjacencyMatrix<K, V> {
    edges: HashMap<K, HashMap<K, V>>,
}

impl<K, V> AdjacencyMatrix for SymmetricAdjacencyMatrix<K, V>
where
    K: Hash + Eq + Clone + Ord,
{
    type Key = K;
    type Value = V;

    fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    fn insert(&mut self, from: K, into: K, data: V) -> Option<V> {
        let (k1, k2) = sort_pair(from, into);
        self.edges.entry(k1).or_default().insert(k2, data)
    }

    fn get(&self, from: &K, into: &K) -> Option<&V> {
        let (k1, k2) = sort_pair(from, into);
        self.edges.get(&k1).and_then(|m| m.get(&k2))
    }

    fn remove(&mut self, from: &K, into: &K) -> Option<V> {
        let (k1, k2) = sort_pair(from, into);
        self.edges.get_mut(&k1).and_then(|m| m.remove(&k2))
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
        self.edges_from(into)
    }
}

#[derive(Clone, Debug)]
pub struct AsymmetricAdjacencyMatrix<K, V> {
    edges: HashMap<K, HashMap<K, V>>,
    back_edges: HashMap<K, HashSet<K>>,
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
            back_edges: HashMap::new(),
        }
    }

    fn insert(&mut self, from: K, into: K, data: V) -> Option<V> {
        self.back_edges
            .entry(into.clone())
            .or_default()
            .insert(from.clone());
        self.edges.entry(from).or_default().insert(into, data)
    }

    fn get(&self, from: &K, into: &K) -> Option<&V> {
        self.edges.get(from).and_then(|m| m.get(into))
    }

    fn remove(&mut self, from: &K, into: &K) -> Option<V> {
        if let Some(value) = self.edges.get_mut(from).and_then(|m| m.remove(into)) {
            if let Some(back_edges) = self.back_edges.get_mut(into) {
                if back_edges.remove(from) && back_edges.is_empty() {
                    self.back_edges.remove(into);
                }
            }
            Some(value)
        } else {
            None
        }
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
        let sources = self
            .back_edges
            .get(into)
            .cloned()
            .unwrap_or_else(|| HashSet::new());
        sources.into_iter().filter_map(move |from| {
            self.edges
                .get(&from)
                .and_then(|targets| targets.get(into))
                .map(|data| (from.clone(), data))
        })
    }
}

pub struct AsymmetricBitvecAdjacencyMatrix<V> {
    data: Vec<MaybeUninit<V>>,
    matrix: BitVec,
    size: usize,
    log2_size: u32,
}

impl<V> AsymmetricBitvecAdjacencyMatrix<V> {
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
        (from << self.log2_size) + into
    }

    fn coordinates(&self, index: usize) -> (usize, usize) {
        let from = index >> self.log2_size;
        let into = index & (self.size - 1);
        (from, into)
    }
}

pub struct SymmetricBitvecAdjacencyMatrix<V> {
    data: Vec<MaybeUninit<V>>,
    matrix: BitVec,
    size: usize,
}

// Calculates the Euler sum of n, the sum of 1..n, also the number of edges in a
// complete graph of n vertices.
fn euler_sum(n: usize) -> usize {
    (n * (n + 1)) / 2
}

// Inverse of the Euler sum function, returning the largest n such that
// euler_sum(n) <= k.
fn euler_sum_inv_floor(k: usize) -> usize {
    // (n * (n + 1)) / 2 = k
    // n * (n + 1) = 2k
    // n^2 + n - 2k = 0
    // By the quadratic formula:
    (1 + 8 * k).isqrt().wrapping_sub(1) / 2
}

#[cfg(test)]
#[test]
fn test_euler_sum() {
    // Test some hand-picked values.
    assert_eq!(euler_sum(0), 0);
    assert_eq!(euler_sum(1), 1);
    assert_eq!(euler_sum(2), 3);
    assert_eq!(euler_sum(3), 6);
    assert_eq!(euler_sum(4), 10);
    assert_eq!(euler_sum_inv_floor(10), 4);
    assert_eq!(euler_sum_inv_floor(9), 3);
    assert_eq!(euler_sum_inv_floor(8), 3);
    assert_eq!(euler_sum_inv_floor(7), 3);
    assert_eq!(euler_sum_inv_floor(6), 3);
    assert_eq!(euler_sum_inv_floor(5), 2);
    assert_eq!(euler_sum_inv_floor(4), 2);
    assert_eq!(euler_sum_inv_floor(3), 2);
    assert_eq!(euler_sum_inv_floor(2), 1);
    assert_eq!(euler_sum_inv_floor(1), 1);
    assert_eq!(euler_sum_inv_floor(0), 0);

    // Test that euler_sum_inv_floor is the inverse of euler_sum and rounds down.
    for n in 0..1000 {
        let k1 = euler_sum(n);
        let k2 = euler_sum(n + 1);
        for k in k1..k2 {
            let n2 = euler_sum_inv_floor(k);
            assert_eq!(n, n2, "Failed at k={}", k);
        }
    }
}

impl<V> SymmetricBitvecAdjacencyMatrix<V> {
    fn with_size(size: usize) -> Self {
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

impl<V> AdjacencyMatrix for AsymmetricBitvecAdjacencyMatrix<V> {
    type Key = usize;
    type Value = V;

    fn new() -> Self {
        Self {
            matrix: BitVec::new(),
            size: 0,
            log2_size: 0,
            data: Vec::new(),
        }
    }

    fn insert(&mut self, from: usize, into: usize, data: V) -> Option<V> {
        if self.index(from, into).is_none() {
            let required_size = usize::max(from, into) + 1;
            if self.size < required_size {
                let mut new_self = Self::with_size(required_size);
                for row in 0..self.size {
                    let old_start = self.unchecked_index(row, 0);
                    let new_start = new_self.unchecked_index(row, 0);
                    new_self.matrix[new_start..new_start + self.size]
                        .copy_from_bitslice(&self.matrix[old_start..old_start + self.size]);
                    for (col, old_datum) in self.data[old_start..old_start + self.size]
                        .iter_mut()
                        .enumerate()
                    {
                        std::mem::swap(old_datum, &mut new_self.data[new_start + col]);
                    }
                }
                *self = new_self;
            }
        }
        let index = self.unchecked_index(from, into);
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
            let (from, into) = self.coordinates(index);
            (from, into, self.unchecked_get_data_ref(index))
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
        let (_, into_col) = self.coordinates(*into);
        (0..self.size)
            .filter_map(move |from_row| self.get(&from_row, &into_col).map(|data| (from_row, data)))
    }
}

// TODO: Review!
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asymmetric_matrix_insert_and_get() {
        let mut matrix = AsymmetricAdjacencyMatrix::new();
        matrix.insert(0, 0, "a");
        assert_eq!(matrix.get(&0, &0), Some(&"a"));
        matrix.insert(1, 0, "b");
        assert_eq!(matrix.get(&1, &0), Some(&"b"));
        matrix.insert(2, 0, "c");
        assert_eq!(matrix.get(&2, &0), Some(&"c"));
        matrix.insert(6, 7, "d");
        assert_eq!(matrix.get(&6, &7), Some(&"d"));
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
        assert_eq!(matrix.insert(0, 1, 42), None);
        assert_eq!(matrix.get(&0, &1), Some(&42));
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
        matrix.insert(0, 0, "a");
        assert_eq!(matrix.get(&0, &0), Some(&"a"));
        matrix.insert(1, 0, "b");
        assert_eq!(matrix.get(&1, &0), Some(&"b"));
        matrix.insert(2, 0, "c");
        assert_eq!(matrix.get(&2, &0), Some(&"c"));
        let edges: Vec<_> = matrix.edges_into(&0).collect();
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().any(|(from, _)| *from == 0));
        assert!(edges.iter().any(|(from, _)| *from == 1));
        assert!(edges.iter().any(|(from, _)| *from == 2));
    }

    #[test]
    fn test_symmetric_matrix_edges_from() {
        let mut matrix = SymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(2, 0, "b");
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_symmetric_matrix_edges_into() {
        let mut matrix = SymmetricAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(2, 0, "b");
        let edges: Vec<_> = matrix.edges_into(&0).collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_asymmetric_bitvec_edges_from() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(0, 3, ());
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(to, _)| *to == 1));
        assert!(edges.iter().any(|(to, _)| *to == 3));
    }

    #[test]
    fn test_asymmetric_bitvec_edges_into() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, "A");
        assert_eq!(matrix.get(&0, &1), Some(&"A"));
        matrix.insert(1, 1, "B");
        assert_eq!(matrix.get(&1, &1), Some(&"B"));
        matrix.insert(3, 1, "C");
        assert_eq!(matrix.get(&3, &1), Some(&"C"));
        let edges: Vec<_> = matrix.edges_into(&1).collect();
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().any(|(from, _)| *from == 0));
        assert!(edges.iter().any(|(from, _)| *from == 1));
        assert!(edges.iter().any(|(from, _)| *from == 3));
    }

    #[test]
    fn test_symmetric_bitvec_insert_and_get() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.get(&0, &1), Some(&()));
        assert_eq!(matrix.get(&1, &0), Some(&()));
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

    #[test]
    fn test_symmetric_bitvec_insert_duplicate() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        assert_eq!(matrix.insert(0, 1, ()), None);
        assert_eq!(matrix.insert(0, 1, ()), Some(()));
    }

    #[test]
    fn test_symmetric_bitvec_remove() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.remove(&0, &1), Some(()));
        assert_eq!(matrix.get(&0, &1), None);
    }

    #[test]
    fn test_symmetric_bitvec_remove_symmetric() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.remove(&1, &0), Some(()));
        assert_eq!(matrix.get(&0, &1), None);
        assert_eq!(matrix.get(&1, &0), None);
    }

    #[test]
    fn test_symmetric_bitvec_remove_nonexistent() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::<()>::new();
        assert_eq!(matrix.remove(&0, &1), None);
    }

    #[test]
    fn test_symmetric_bitvec_edges() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(3, 2, "b");
        let mut edges: Vec<_> = matrix.edges().collect();
        edges.sort();
        assert_eq!(edges, vec![(0, 1, &"a"), (2, 3, &"b")]);
    }

    #[test]
    fn test_symmetric_bitvec_large_indices() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(100, 200, ());
        assert_eq!(matrix.get(&100, &200), Some(&()));
        assert_eq!(matrix.get(&200, &100), Some(&()));
    }

    #[test]
    fn test_symmetric_bitvec_self_loop() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(5, 5, ());
        assert_eq!(matrix.get(&5, &5), Some(&()));
    }

    #[test]
    fn test_symmetric_bitvec_coordinates() {
        let matrix = SymmetricBitvecAdjacencyMatrix::<()>::with_size(8);
        for index in 0..matrix.matrix.len() {
            let (k1, k2) = matrix.coordinates(index);
            let computed_index = matrix.unchecked_index(k1, k2);
            assert_eq!(index, computed_index, "Failed at index {}", index);
        }
    }
}
