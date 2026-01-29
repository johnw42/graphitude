#![cfg(feature = "bitvec")]
use std::{fmt::Debug, hash::Hash};

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
    Self: Sized,
    Self::Key: Hash + Eq + Clone,
{
    type Key;
    type Value;
    type Symmetry: Symmetry;
    type Storage: Storage;

    /// Creates a new, empty adjacency matrix.
    fn new() -> Self;

    /// Inserts an entry at `row` and `col` with associated data `data`.
    /// Returns the previous data associated with the entry, if any.
    fn insert(&mut self, row: Self::Key, col: Self::Key, data: Self::Value) -> Option<Self::Value>;

    /// Clears all entries from the adjacency matrix.
    fn clear(&mut self);

    /// Gets a reference to the data associated with the entry at `row` and `col`, if it exists.
    fn get(&self, row: Self::Key, col: Self::Key) -> Option<&Self::Value>;

    /// Removes the entry at `row` and `col`, returning the associated data if it existed.
    fn remove(&mut self, row: Self::Key, col: Self::Key) -> Option<Self::Value>;

    /// Iterates over all entries in the adjacency matrix.
    fn entries<'a>(&'a self) -> impl Iterator<Item = (Self::Key, Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a;

    /// For internal use.  Gets the canonical indices for the given keys.  This will return a pair
    /// `(k1, k2)` such that for symmetric matrices, `k1 <= k2`.
    #[doc(hidden)]
    fn entry_indices(k1: Self::Key, k2: Self::Key) -> (Self::Key, Self::Key) {
        (k1, k2)
    }

    /// Gets the entry at the given row and col.
    fn entry_at(
        &self,
        row: Self::Key,
        col: Self::Key,
    ) -> Option<(Self::Key, Self::Key, &'_ Self::Value)> {
        self.get(row.clone(), col.clone()).map(|data| {
            let (k1, k2) = Self::entry_indices(row.clone(), col.clone());
            (k1, k2, data)
        })
    }

    /// Iterates over all entries in the given row.
    fn entries_in_row<'a>(
        &'a self,
        row: Self::Key,
    ) -> impl Iterator<Item = (Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a;

    /// Iterates over all entries in the given col.
    fn entries_in_col<'a>(
        &'a self,
        col: Self::Key,
    ) -> impl Iterator<Item = (Self::Key, &'a Self::Value)>
    where
        Self::Value: 'a;

    fn reserve(&mut self, _additional: usize) {
        todo!()
    }

    fn reserve_exact(&mut self, _additional: usize) {
        todo!()
    }

    fn compact(&mut self) {
        todo!()
    }

    fn shrink_to_fit(&mut self) {
        todo!()
    }
}

/// Trait for selecting an adjacency matrix implementation based on symmetry and storage.
pub trait AdjacencyMatrixSelector<K, V>
where
    K: Hash + Eq + Clone,
{
    type Matrix: AdjacencyMatrix<Key = K, Value = V>;
}

#[cfg(feature = "bitvec")]
impl<K, V> AdjacencyMatrixSelector<K, V> for (Asymmetric, BitvecStorage)
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq + Hash,
{
    type Matrix = AsymmetricBitvecAdjacencyMatrix<K, V>;
}

#[cfg(feature = "bitvec")]
impl<K, V> AdjacencyMatrixSelector<K, V> for (Symmetric, BitvecStorage)
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord,
{
    type Matrix = SymmetricBitvecAdjacencyMatrix<K, V>;
}

impl<K, V> AdjacencyMatrixSelector<K, V> for (Symmetric, HashStorage)
where
    K: Hash + Eq + Clone + Ord + Debug,
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
