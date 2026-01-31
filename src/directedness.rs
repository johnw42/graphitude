#[cfg(feature = "bitvec")]
use std::{fmt::Debug, hash::Hash};

use crate::pairs::{OrderedPair, Pair, UnorderedPair};

#[cfg(feature = "bitvec")]
use crate::{
    AdjacencyMatrix, AsymmetricHashAdjacencyMatrix, SymmetricHashAdjacencyMatrix,
    adjacency_matrix::{Asymmetric, Symmetric, Symmetry},
};

/// Marker type representing directed graph edges.
pub struct Directed;

/// Marker type representing undirected graph edges.
pub struct Undirected;

/// Trait defining the directedness behavior of graph edges.
///
/// This trait is implemented by [`Directed`] and [`Undirected`] marker types
/// to provide compile-time specialization of graph behavior.
pub trait Directedness: Sized {
    #[cfg(feature = "bitvec")]
    type Symmetry: Symmetry;
    #[cfg(feature = "bitvec")]
    type AdjacencyMatrix<K, N>: AdjacencyMatrix<Index = K, Value = N>
    where
        K: Eq + Hash + Clone + Ord + Debug;

    type Pair<T: Eq + Hash + Clone + Debug + Ord>: Pair<T> + Eq + Hash + Clone + Debug + Ord;

    fn is_directed() -> bool;
}

impl Directedness for Directed {
    #[cfg(feature = "bitvec")]
    type Symmetry = Asymmetric;
    #[cfg(feature = "bitvec")]
    type AdjacencyMatrix<K, N>
        = AsymmetricHashAdjacencyMatrix<K, N>
    where
        K: Eq + Hash + Clone + Ord + Debug;

    type Pair<T: Eq + Hash + Clone + Debug + Ord> = OrderedPair<T>;

    fn is_directed() -> bool {
        true
    }
}

impl Directedness for Undirected {
    #[cfg(feature = "bitvec")]
    type Symmetry = Symmetric;
    #[cfg(feature = "bitvec")]
    type AdjacencyMatrix<K, N>
        = SymmetricHashAdjacencyMatrix<K, N>
    where
        K: Eq + Hash + Clone + Ord + Debug;

    type Pair<T: Eq + Hash + Clone + Debug + Ord> = UnorderedPair<T>;

    fn is_directed() -> bool {
        false
    }
}
