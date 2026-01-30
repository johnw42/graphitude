use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::adjacency_matrix::{AdjacencyMatrix, Asymmetric, HashStorage};

/// Hash-based asymmetric adjacency matrix for directed graphs.
///
/// Stores separate forward and backward edge indices for efficient traversal
/// in both directions.
#[derive(Clone, Debug)]
pub struct AsymmetricHashAdjacencyMatrix<K, V> {
    edges: HashMap<K, HashMap<K, V>>,
    back_edges: HashMap<K, HashSet<K>>,
}

impl<K, V> AdjacencyMatrix for AsymmetricHashAdjacencyMatrix<K, V>
where
    K: Hash + Eq + Clone,
{
    type Index = K;
    type Value = V;
    type Symmetry = Asymmetric;
    type Storage = HashStorage;

    fn new() -> Self {
        AsymmetricHashAdjacencyMatrix {
            edges: HashMap::new(),
            back_edges: HashMap::new(),
        }
    }

    fn insert(&mut self, row: K, col: K, data: V) -> Option<V> {
        self.back_edges
            .entry(col.clone())
            .or_default()
            .insert(row.clone());
        self.edges.entry(row).or_default().insert(col, data)
    }

    fn get(&self, row: K, col: K) -> Option<&V> {
        self.edges.get(&row).and_then(|m| m.get(&col))
    }

    fn remove(&mut self, row: K, col: K) -> Option<V> {
        if let Some(value) = self.edges.get_mut(&row).and_then(|m| m.remove(&col)) {
            if let Some(back_edges) = self.back_edges.get_mut(&col) {
                if back_edges.remove(&row) && back_edges.is_empty() {
                    self.back_edges.remove(&col);
                }
            }
            Some(value)
        } else {
            None
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (K, K, &'a V)>
    where
        V: 'a,
    {
        self.edges.iter().flat_map(|(row, targets)| {
            targets
                .iter()
                .map(|(col, data)| (row.clone(), col.clone(), data))
        })
    }

    fn into_iter(self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        self.edges.into_iter().flat_map(|(row, targets)| {
            targets
                .into_iter()
                .map(move |(col, data)| (row.clone(), col, data))
        })
    }

    fn entries_in_row<'a>(&'a self, row: K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        self.edges
            .get(&row)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(col, data)| (col.clone(), data)))
    }

    fn entries_in_col<'a>(&'a self, col: K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        let sources = self
            .back_edges
            .get(&col)
            .cloned()
            .unwrap_or_else(|| HashSet::new());
        sources.into_iter().filter_map(move |row| {
            self.edges
                .get(&row)
                .and_then(|targets| targets.get(&col))
                .map(|data| (row.clone(), data))
        })
    }

    fn clear(&mut self) {
        self.edges.clear();
        self.back_edges.clear();
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
        assert_eq!(matrix.get(0, 1), Some(&"second"));
    }

    #[test]
    fn test_remove() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.remove(0, 1), Some("edge"));
        assert_eq!(matrix.get(0, 1), None);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut matrix: AsymmetricHashAdjacencyMatrix<usize, &str> =
            AsymmetricHashAdjacencyMatrix::new();
        assert_eq!(matrix.remove(0, 1), None);
    }

    #[test]
    fn test_entries() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(1, 0, "b");
        let entries: Vec<_> = matrix.iter().collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_entries_in_row() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(0, 2, "b");
        matrix.insert(1, 2, "c");
        let entries: Vec<_> = matrix.entries_in_row(0).collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|(to, _)| *to == 1));
        assert!(entries.iter().any(|(to, _)| *to == 2));
    }

    #[test]
    fn test_entries_in_col() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 0, "a");
        assert_eq!(matrix.get(0, 0), Some(&"a"));
        matrix.insert(1, 0, "b");
        assert_eq!(matrix.get(1, 0), Some(&"b"));
        matrix.insert(2, 0, "c");
        assert_eq!(matrix.get(2, 0), Some(&"c"));
        let entries: Vec<_> = matrix.entries_in_col(0).collect();
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().any(|(row, _)| *row == 0));
        assert!(entries.iter().any(|(row, _)| *row == 1));
        assert!(entries.iter().any(|(row, _)| *row == 2));
    }
}
