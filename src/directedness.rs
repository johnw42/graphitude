use std::hash::Hash;

use crate::{
    AdjacencyMatrix, AsymmetricHashAdjacencyMatrix, SymmetricHashAdjacencyMatrix,
    adjacency_matrix::{Asymmetric, Symmetric, Symmetry},
};

pub struct Directed;
pub struct Undirected;

pub trait Directedness {
    type Symmetry: Symmetry;
    type AdjacencyMatrix<K, V>: AdjacencyMatrix<Key = K, Value = V>
    where
        K: Eq + Hash + Clone + Ord;

    fn is_directed() -> bool;
}

impl Directedness for Directed {
    type Symmetry = Asymmetric;
    type AdjacencyMatrix<K, V>
        = AsymmetricHashAdjacencyMatrix<K, V>
    where
        K: Eq + Hash + Clone + Ord;

    fn is_directed() -> bool {
        true
    }
}

impl Directedness for Undirected {
    type Symmetry = Symmetric;
    type AdjacencyMatrix<K, V>
        = SymmetricHashAdjacencyMatrix<K, V>
    where
        K: Eq + Hash + Clone + Ord;

    fn is_directed() -> bool {
        false
    }
}
