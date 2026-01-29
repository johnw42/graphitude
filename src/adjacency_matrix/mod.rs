#![cfg(feature = "bitvec")]
use std::{fmt::Debug, hash::Hash};

use crate::{
    AsymmetricBitvecAdjacencyMatrix, AsymmetricHashAdjacencyMatrix, SymmetricBitvecAdjacencyMatrix,
    SymmetricHashAdjacencyMatrix,
};

pub mod bitvec;
pub mod hash;

pub(crate) trait CompactionCount: Eq + Clone + Copy + Default {
    fn increment(self) -> Self;
}

impl CompactionCount for () {
    fn increment(self) -> Self {
        ()
    }
}

impl CompactionCount for usize {
    fn increment(self) -> Self {
        self.wrapping_add(1)
    }
}

pub struct BitvecStorage;
pub struct HashStorage;

pub trait Storage {
    #[allow(private_bounds)]
    type CompactionCount: CompactionCount;
}

impl Storage for BitvecStorage {
    #[cfg(not(feature = "unchecked"))]
    type CompactionCount = usize;
    #[cfg(feature = "unchecked")]
    type CompactionCount = ();
}

impl Storage for HashStorage {
    type CompactionCount = ();
}

pub struct Symmetric;
pub struct Asymmetric;

pub trait Symmetry {}
impl Symmetry for Symmetric {}
impl Symmetry for Asymmetric {}

pub trait AdjacencyMatrix
where
    Self: Sized,
{
    type Index: Hash + Eq + Clone;
    type Value;
    type Symmetry: Symmetry;
    type Storage: Storage;

    /// Creates a new, empty adjacency matrix.
    fn new() -> Self;

    /// Creates an empty adjacency matrix of the same type.
    fn clone_empty(&self) -> Self {
        Self::new()
    }

    /// Inserts an entry at `row` and `col` with associated data `data`.
    /// Returns the previous data associated with the entry, if any.
    fn insert(
        &mut self,
        row: Self::Index,
        col: Self::Index,
        data: Self::Value,
    ) -> Option<Self::Value>;

    /// Clears all entries from the adjacency matrix.
    fn clear(&mut self);

    /// Gets a reference to the data associated with the entry at `row` and `col`, if it exists.
    fn get(&self, row: Self::Index, col: Self::Index) -> Option<&Self::Value>;

    /// Removes the entry at `row` and `col`, returning the associated data if it existed.
    fn remove(&mut self, row: Self::Index, col: Self::Index) -> Option<Self::Value>;

    /// Iterates over all entries in the adjacency matrix. Returns an iterator yielding
    /// `(row, col, data)` tuples.
    fn iter<'a>(&'a self) -> impl Iterator<Item = (Self::Index, Self::Index, &'a Self::Value)>
    where
        Self::Value: 'a;

    /// Iterates over all entries in the adjacency matrix, consuming the matrix.
    /// Returns an iterator yielding `(row, col, data)` tuples.
    ///
    /// This trait does not extend `IntoIterator` directly to allow for more
    /// flexible implementations.
    fn into_iter(self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)>;

    /// For internal use.  Gets the canonical indices for the given keys.  This will return a pair
    /// `(k1, k2)` such that for symmetric matrices, `k1 <= k2`.
    #[doc(hidden)]
    fn entry_indices(k1: Self::Index, k2: Self::Index) -> (Self::Index, Self::Index) {
        (k1, k2)
    }

    /// Gets the entry at the given row and col.
    fn entry_at(
        &self,
        row: Self::Index,
        col: Self::Index,
    ) -> Option<(Self::Index, Self::Index, &'_ Self::Value)> {
        self.get(row.clone(), col.clone()).map(|data| {
            let (k1, k2) = Self::entry_indices(row.clone(), col.clone());
            (k1, k2, data)
        })
    }

    /// Iterates over all entries in the given row.
    fn entries_in_row<'a>(
        &'a self,
        row: Self::Index,
    ) -> impl Iterator<Item = (Self::Index, &'a Self::Value)>
    where
        Self::Value: 'a;

    /// Iterates over all entries in the given col.
    fn entries_in_col<'a>(
        &'a self,
        col: Self::Index,
    ) -> impl Iterator<Item = (Self::Index, &'a Self::Value)>
    where
        Self::Value: 'a;

    /// Reserves capacity for at least `additional` more rows and columns to be added.
    fn reserve(&mut self, _additional: usize) {
        dbg!("missing impolementation for this adjacency matrix");
    }

    /// Reserves capacity for exactly `additional` more rows and columns to be added.
    fn reserve_exact(&mut self, _additional: usize) {
        dbg!("missing impolementation for this adjacency matrix");
    }

    /// Shrinks the adjacency matrix to fit its current size.
    fn shrink_to_fit(&mut self) {
        dbg!("missing impolementation for this adjacency matrix");
    }
}

/// Trait for selecting an adjacency matrix implementation based on symmetry and storage.
pub trait AdjacencyMatrixSelector<K, V>
where
    K: Hash + Eq + Clone,
{
    type Matrix: AdjacencyMatrix<Index = K, Value = V>;
}

#[cfg(feature = "bitvec")]
impl<K, V> AdjacencyMatrixSelector<K, V> for (Asymmetric, BitvecStorage)
where
    K: Into<usize> + From<usize> + Copy + Eq + Hash,
{
    type Matrix = AsymmetricBitvecAdjacencyMatrix<K, V>;
}

#[cfg(feature = "bitvec")]
impl<K, V> AdjacencyMatrixSelector<K, V> for (Symmetric, BitvecStorage)
where
    K: Into<usize> + From<usize> + Copy + Eq + Hash + Ord,
{
    type Matrix = SymmetricBitvecAdjacencyMatrix<K, V>;
}

impl<K, V> AdjacencyMatrixSelector<K, V> for (Symmetric, HashStorage)
where
    K: Hash + Eq + Copy + Ord + Debug,
{
    type Matrix = SymmetricHashAdjacencyMatrix<K, V>;
}

impl<K, V> AdjacencyMatrixSelector<K, V> for (Asymmetric, HashStorage)
where
    K: Hash + Eq + Copy,
{
    type Matrix = AsymmetricHashAdjacencyMatrix<K, V>;
}

// Helper type alias for convenient usage
pub type SelectMatrix<Sym, Stor, K, V> = <(Sym, Stor) as AdjacencyMatrixSelector<K, V>>::Matrix;
