#[cfg(feature = "bitvec")]
use std::{fmt::Debug, hash::Hash};

use crate::util::sort_pair;

#[cfg(feature = "bitvec")]
use crate::{
    AdjacencyMatrix, AsymmetricHashAdjacencyMatrix, SymmetricHashAdjacencyMatrix,
    adjacency_matrix::{Asymmetric, Symmetric, Symmetry},
};

pub struct Directed;
pub struct Undirected;

pub trait Directedness {
    #[cfg(feature = "bitvec")]
    type Symmetry: Symmetry;
    #[cfg(feature = "bitvec")]
    type AdjacencyMatrix<K, V>: AdjacencyMatrix<Key = K, Value = V>
    where
        K: Eq + Hash + Clone + Ord + Debug;

    fn is_directed() -> bool;
    fn maybe_sort<K: Ord>(a: K, b: K) -> (K, K);
}

impl Directedness for Directed {
    #[cfg(feature = "bitvec")]
    type Symmetry = Asymmetric;
    #[cfg(feature = "bitvec")]
    type AdjacencyMatrix<K, V>
        = AsymmetricHashAdjacencyMatrix<K, V>
    where
        K: Eq + Hash + Clone + Ord + Debug;

    fn is_directed() -> bool {
        true
    }

    fn maybe_sort<K: Ord>(a: K, b: K) -> (K, K) {
        (a, b)
    }
}

impl Directedness for Undirected {
    #[cfg(feature = "bitvec")]
    type Symmetry = Symmetric;
    #[cfg(feature = "bitvec")]
    type AdjacencyMatrix<K, V>
        = SymmetricHashAdjacencyMatrix<K, V>
    where
        K: Eq + Hash + Clone + Ord + Debug;

    fn is_directed() -> bool {
        false
    }

    fn maybe_sort<K: Ord>(a: K, b: K) -> (K, K) {
        sort_pair(a, b)
    }
}
