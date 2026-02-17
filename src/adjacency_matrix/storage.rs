use std::{fmt::Debug, hash::Hash};

use crate::{
    DirectednessTrait,
    adjacency_matrix::{
        AdjacencyMatrix, bitvec::matrix::BitvecAdjacencyMatrix, hash::HashAdjacencyMatrix,
    },
};

/// Trait representing the compaction count for adjacency matrix storage
/// backends.  This is used to track modifications that may require compaction
/// of the storage while making the compaction count type omittable by setting
/// it to `()` for storage backends that do not require it.
pub(crate) trait CompactionCount:
    Clone + Copy + Debug + Default + Eq + Hash + PartialOrd + Ord + Send + Sync
{
    fn increment(self) -> Self;
}

impl CompactionCount for () {
    fn increment(self) -> Self {
        self
    }
}

impl CompactionCount for usize {
    fn increment(self) -> Self {
        self.wrapping_add(1)
    }
}

/// Trait defining storage backend behavior for adjacency matrices.
///
/// Implemented by [`BitvecStorage`] and [`HashStorage`] marker types.
pub trait Storage {
    #[allow(private_bounds)]
    type CompactionCount: CompactionCount;

    type Matrix<I, V, D>: AdjacencyMatrix<Index = I, Value = V>
    where
        I: Into<usize> + From<usize> + Copy + Eq + Hash + Ord + Debug,
        D: DirectednessTrait + Default;
}

/// Marker type for bitvec-based adjacency matrix storage.
pub struct BitvecStorage;

/// Marker type for hash-based adjacency matrix storage.
pub struct HashStorage;

impl Storage for BitvecStorage {
    #[cfg(not(feature = "unchecked"))]
    type CompactionCount = usize;
    #[cfg(feature = "unchecked")]
    type CompactionCount = ();
    type Matrix<I, V, D>
        = BitvecAdjacencyMatrix<I, V, D>
    where
        I: Into<usize> + From<usize> + Copy + Eq + Hash + Ord + Debug,
        D: DirectednessTrait + Default;
}

impl Storage for HashStorage {
    type CompactionCount = ();
    type Matrix<I, V, D>
        = HashAdjacencyMatrix<I, V, D>
    where
        I: Into<usize> + From<usize> + Copy + Eq + Hash + Ord + Debug,
        D: DirectednessTrait + Default;
}
