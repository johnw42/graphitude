use std::hash::Hash;

use crate::adjacency_matrix::{
    AdjacencyMatrix, AsymmetricAdjacencyMatrix, SymmetricAdjacencyMatrix,
};

pub struct Directed;
pub struct Undirected;

pub trait Directedness {
    type AdjacencyMatrix<K, V>: AdjacencyMatrix<Key = K, Value = V>
    where
        K: Eq + Hash + Clone + Ord;

    fn is_directed() -> bool;
}

impl Directedness for Directed {
    type AdjacencyMatrix<K, V>
        = AsymmetricAdjacencyMatrix<K, V>
    where
        K: Eq + Hash + Clone + Ord;

    fn is_directed() -> bool {
        true
    }
}

impl Directedness for Undirected {
    type AdjacencyMatrix<K, V>
        = SymmetricAdjacencyMatrix<K, V>
    where
        K: Eq + Hash + Clone + Ord;

    fn is_directed() -> bool {
        false
    }
}
