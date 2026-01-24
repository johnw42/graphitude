use std::hash::Hash;

pub mod bitvec;
pub mod hash;

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

pub trait MaxtrixImpl<K,V> {
    type Symmetric: AdjacencyMatrix<Key=K, Value=V>;
    type Asymmetric: AdjacencyMatrix<Key=K, Value=V>;
}

pub struct BitvecImpl;
pub struct HashImpl;

impl<V> MaxtrixImpl<usize, V> for BitvecImpl {
    type Symmetric = bitvec::symmetric::SymmetricBitvecAdjacencyMatrix<V>;
    type Asymmetric = bitvec::asymmetric::AsymmetricBitvecAdjacencyMatrix<V>;
}

impl<K, V> MaxtrixImpl<K, V> for HashImpl where K: Hash + Eq + Clone + Ord {
    type Symmetric = hash::symmetric::SymmetricHashAdjacencyMatrix<K, V>;
    type Asymmetric = hash::asymmetric::AsymmetricHashAdjacencyMatrix<K, V>;
}