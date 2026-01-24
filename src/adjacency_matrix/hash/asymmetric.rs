use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::adjacency_matrix::{AdjacencyMatrix, Asymmetric, HashStorage};

#[derive(Clone, Debug)]
pub struct AsymmetricHashAdjacencyMatrix<K, V> {
    edges: HashMap<K, HashMap<K, V>>,
    back_edges: HashMap<K, HashSet<K>>,
}

impl<K, V> AdjacencyMatrix for AsymmetricHashAdjacencyMatrix<K, V>
where
    K: Hash + Eq + Clone,
{
    type Key = K;
    type Value = V;
    type Symmetry = Asymmetric;
    type Storage = HashStorage;

    fn new() -> Self {
        AsymmetricHashAdjacencyMatrix {
            edges: HashMap::new(),
            back_edges: HashMap::new(),
        }
    }

    fn insert(&mut self, from: K, into: K, data: V) -> Option<V> {
        self.back_edges
            .entry(into.clone())
            .or_default()
            .insert(from.clone());
        self.edges.entry(from).or_default().insert(into, data)
    }

    fn get(&self, from: &K, into: &K) -> Option<&V> {
        self.edges.get(from).and_then(|m| m.get(into))
    }

    fn remove(&mut self, from: &K, into: &K) -> Option<V> {
        if let Some(value) = self.edges.get_mut(from).and_then(|m| m.remove(into)) {
            if let Some(back_edges) = self.back_edges.get_mut(into) {
                if back_edges.remove(from) && back_edges.is_empty() {
                    self.back_edges.remove(into);
                }
            }
            Some(value)
        } else {
            None
        }
    }

    fn edges<'a>(&'a self) -> impl Iterator<Item = (K, K, &'a V)>
    where
        V: 'a,
    {
        self.edges.iter().flat_map(|(from, targets)| {
            targets
                .iter()
                .map(|(into, data)| (from.clone(), into.clone(), data))
        })
    }

    fn edges_from<'a>(&'a self, from: &K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        self.edges
            .get(from)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(into, data)| (into.clone(), data)))
    }

    fn edges_into<'a>(&'a self, into: &K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        let sources = self
            .back_edges
            .get(into)
            .cloned()
            .unwrap_or_else(|| HashSet::new());
        sources.into_iter().filter_map(move |from| {
            self.edges
                .get(&from)
                .and_then(|targets| targets.get(into))
                .map(|data| (from.clone(), data))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_overwrites() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "first");
        assert_eq!(matrix.insert(0, 1, "second"), Some("first"));
        assert_eq!(matrix.get(&0, &1), Some(&"second"));
    }

    #[test]
    fn test_remove() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.remove(&0, &1), Some("edge"));
        assert_eq!(matrix.get(&0, &1), None);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut matrix: AsymmetricHashAdjacencyMatrix<usize, &str> =
            AsymmetricHashAdjacencyMatrix::new();
        assert_eq!(matrix.remove(&0, &1), None);
    }

    #[test]
    fn test_edges() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(1, 0, "b");
        let edges: Vec<_> = matrix.edges().collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_edges_from() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(0, 2, "b");
        matrix.insert(1, 2, "c");
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(to, _)| *to == 1));
        assert!(edges.iter().any(|(to, _)| *to == 2));
    }

    #[test]
    fn test_edges_into() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 0, "a");
        assert_eq!(matrix.get(&0, &0), Some(&"a"));
        matrix.insert(1, 0, "b");
        assert_eq!(matrix.get(&1, &0), Some(&"b"));
        matrix.insert(2, 0, "c");
        assert_eq!(matrix.get(&2, &0), Some(&"c"));
        let edges: Vec<_> = matrix.edges_into(&0).collect();
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().any(|(from, _)| *from == 0));
        assert!(edges.iter().any(|(from, _)| *from == 1));
        assert!(edges.iter().any(|(from, _)| *from == 2));
    }
}
