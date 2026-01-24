use std::hash::Hash;

use crate::{
    AsymmetricBitvecAdjacencyMatrix, AsymmetricHashAdjacencyMatrix, SymmetricBitvecAdjacencyMatrix,
    SymmetricHashAdjacencyMatrix,
};

pub mod bitvec;
pub mod hash;

pub struct BitvecStorage;
pub struct HashStorage;
pub trait Storage {}
impl Storage for BitvecStorage {}
impl Storage for HashStorage {}

pub struct Symmetric;
pub struct Asymmetric;

pub trait Symmetry {}
impl Symmetry for Symmetric {}
impl Symmetry for Asymmetric {}

pub trait AdjacencyMatrix
where
    Self::Key: Hash + Eq + Clone,
{
    type Key;
    type Value;
    type Symmetry: Symmetry;
    type Storage: Storage;

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

    /// Iterates over all edges between the given vertices `from` and `into`.
    fn edge_between(
        &self,
        from: &Self::Key,
        into: &Self::Key,
    ) -> Option<(Self::Key, Self::Key, &'_ Self::Value)> {
        self.get(from, into)
            .map(|data| (from.clone(), into.clone(), data))
    }

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

/// Trait for selecting an adjacency matrix implementation based on symmetry and storage.
pub trait AdjacencyMatrixSelector<K, V>
where
    K: Hash + Eq + Clone,
{
    type Matrix: AdjacencyMatrix<Key = K, Value = V>;
}

impl<K, V> AdjacencyMatrixSelector<K, V> for (Asymmetric, BitvecStorage)
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq + Hash,
{
    type Matrix = AsymmetricBitvecAdjacencyMatrix<K, V>;
}

impl<K, V> AdjacencyMatrixSelector<K, V> for (Symmetric, BitvecStorage)
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord,
{
    type Matrix = SymmetricBitvecAdjacencyMatrix<K, V>;
}

impl<K, V> AdjacencyMatrixSelector<K, V> for (Symmetric, HashStorage)
where
    K: Hash + Eq + Clone + Ord,
{
    type Matrix = SymmetricHashAdjacencyMatrix<K, V>;
}

impl<K, V> AdjacencyMatrixSelector<K, V> for (Asymmetric, HashStorage)
where
    K: Hash + Eq + Clone,
{
    type Matrix = AsymmetricHashAdjacencyMatrix<K, V>;
}

// Helper type alias for convenient usage
pub type SelectMatrix<Sym, Stor, K, V> = <(Sym, Stor) as AdjacencyMatrixSelector<K, V>>::Matrix;
