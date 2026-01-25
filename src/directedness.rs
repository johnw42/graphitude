use std::{fmt::Debug, hash::Hash};

use crate::{
    AdjacencyMatrix, AsymmetricHashAdjacencyMatrix, SymmetricHashAdjacencyMatrix,
    adjacency_matrix::{Asymmetric, Symmetric, Symmetry},
    util::sort_pair,
};

pub struct Directed;
pub struct Undirected;

pub trait Directedness {
    type Symmetry: Symmetry;
    type AdjacencyMatrix<K, V>: AdjacencyMatrix<Key = K, Value = V>
    where
        K: Eq + Hash + Clone + Ord + Debug;

    fn is_directed() -> bool;
    fn maybe_sort<K: Ord>(a: K, b: K) -> (K, K);
}

impl Directedness for Directed {
    type Symmetry = Asymmetric;
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
    type Symmetry = Symmetric;
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
