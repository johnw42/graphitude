use std::{fmt::Debug, hash::Hash};

use crate::{
    AsymmetricBitvecAdjacencyMatrix, AsymmetricHashAdjacencyMatrix, DirectednessTrait,
    SymmetricBitvecAdjacencyMatrix, SymmetricHashAdjacencyMatrix, edge_ends::EdgeEnds,
};
#[cfg(feature = "bitvec")]
use crate::{Directed, Undirected};

/// Bitvec-based adjacency matrix implementations.
pub mod bitvec;

/// Hash-based adjacency matrix implementations.
pub mod hash;

/// Storage types for adjacency matrices.
mod storage;

pub use storage::{BitvecStorage, HashStorage, Storage};

pub(crate) use storage::CompactionCount;

type Index<M> = <M as AdjacencyMatrix>::Index;
type Pair<M> = EdgeEnds<Index<M>, <M as AdjacencyMatrix>::Directedness>;

/// Trait for adjacency matrix data structures.
///
/// Provides methods for inserting, removing, and querying entries in an adjacency matrix.
/// Supports both symmetric (undirected) and asymmetric (directed) matrix implementations.
pub trait AdjacencyMatrix
where
    Self: Sized,
{
    type Index: Hash + Eq + Clone + Ord + Debug;
    type Value;
    type Directedness: DirectednessTrait + Default;
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

    /// Clears all entries in the given row and and the given column.
    fn clear_row_and_column(&mut self, row: Self::Index, col: Self::Index);

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

    /// Returns the number of entries in the adjacency matrix.
    fn len(&self) -> usize {
        self.iter().count()
    }

    fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    /// For internal use.  Gets the canonical indices for the given indices.  This will return a pair
    /// `(i1, i2)` such that for symmetric matrices, `i1 <= i2`.
    #[doc(hidden)]
    fn entry_indices(i1: Self::Index, i2: Self::Index) -> Pair<Self> {
        Self::Directedness::default().make_pair(i1, i2)
    }

    /// Gets the entry at the given row and col.
    fn entry_at(
        &self,
        row: Self::Index,
        col: Self::Index,
    ) -> Option<(Pair<Self>, &'_ Self::Value)> {
        self.get(row.clone(), col.clone())
            .map(|data| (Self::entry_indices(row.clone(), col.clone()), data))
    }

    /// Iterates over all entries in the given row.
    fn entries_in_row(
        &self,
        row: Self::Index,
    ) -> impl Iterator<Item = (Self::Index, &'_ Self::Value)> + '_;

    /// Iterates over all entries in the given col.
    fn entries_in_col(
        &self,
        col: Self::Index,
    ) -> impl Iterator<Item = (Self::Index, &'_ Self::Value)> + '_;

    /// Reserves capacity for at least `additional` more rows and columns to be added.
    fn reserve(&mut self, _additional: usize) {
        // TODO: implement for specific adjacency matrices
    }

    /// Reserves capacity for exactly `additional` more rows and columns to be added.
    fn reserve_exact(&mut self, _additional: usize) {
        // TODO: implement for specific adjacency matrices
    }

    /// Shrinks the adjacency matrix to fit its current size.
    fn shrink_to_fit(&mut self) {
        // TODO: implement for specific adjacency matrices
    }
}

/// Trait for selecting an adjacency matrix implementation based on symmetry and storage.
///
/// This trait maps combinations of [`SymmetryTrait`] and [`Storage`] types to concrete
/// adjacency matrix implementations.
pub trait AdjacencyMatrixSelector<I, V>
where
    I: Hash + Eq + Clone,
{
    type Matrix: AdjacencyMatrix<Index = I, Value = V>;
}

#[cfg(feature = "bitvec")]
impl<I, V> AdjacencyMatrixSelector<I, V> for (Directed, BitvecStorage)
where
    I: Into<usize> + From<usize> + Copy + Eq + Hash + Ord + Debug,
{
    type Matrix = AsymmetricBitvecAdjacencyMatrix<I, V>;
}

#[cfg(feature = "bitvec")]
impl<I, V> AdjacencyMatrixSelector<I, V> for (Undirected, BitvecStorage)
where
    I: Into<usize> + From<usize> + Copy + Eq + Hash + Ord + Debug,
{
    type Matrix = SymmetricBitvecAdjacencyMatrix<I, V>;
}

impl<I, V> AdjacencyMatrixSelector<I, V> for (Directed, HashStorage)
where
    I: Hash + Eq + Copy + Ord + Debug,
{
    type Matrix = AsymmetricHashAdjacencyMatrix<I, V>;
}

impl<I, V> AdjacencyMatrixSelector<I, V> for (Undirected, HashStorage)
where
    I: Hash + Eq + Copy + Ord + Debug,
{
    type Matrix = SymmetricHashAdjacencyMatrix<I, V>;
}

/// Type alias for selecting an adjacency matrix implementation.
///
/// Resolves to the appropriate matrix type based on the directedness and storage parameters.
/// # Type Parameters
/// * `D` - The directedness type ([`Directed`] or [`Undirected`])
/// * `S` - The storage type ([`BitvecStorage`] or [`HashStorage`])
/// * `I` - The index type
/// * `V` - The value type
pub type SelectMatrix<D, S, I, V> = <(D, S) as AdjacencyMatrixSelector<I, V>>::Matrix;
