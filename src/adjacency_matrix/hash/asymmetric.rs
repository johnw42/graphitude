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
pub struct AsymmetricHashAdjacencyMatrix<I, V> {
    edges: HashMap<I, HashMap<I, V>>,
    back_edges: HashMap<I, HashSet<I>>,
    entry_count: usize,
}

impl<I, V> AdjacencyMatrix for AsymmetricHashAdjacencyMatrix<I, V>
where
    I: Hash + Eq + Clone + Ord,
{
    type Index = I;
    type Value = V;
    type Symmetry = Asymmetric;
    type Storage = HashStorage;

    fn new() -> Self {
        AsymmetricHashAdjacencyMatrix {
            edges: HashMap::new(),
            back_edges: HashMap::new(),
            entry_count: 0,
        }
    }

    fn insert(&mut self, row: I, col: I, data: V) -> Option<V> {
        self.back_edges
            .entry(col.clone())
            .or_default()
            .insert(row.clone());
        let old_data = self.edges.entry(row).or_default().insert(col, data);
        if old_data.is_none() {
            self.entry_count += 1;
        }
        old_data
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        self.edges.get(&row).and_then(|m| m.get(&col))
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        if let Some(value) = self.edges.get_mut(&row).and_then(|m| m.remove(&col)) {
            if let Some(back_edges) = self.back_edges.get_mut(&col)
                && back_edges.remove(&row)
                && back_edges.is_empty()
            {
                self.back_edges.remove(&col);
            }
            self.entry_count -= 1;
            Some(value)
        } else {
            None
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (I, I, &'a V)>
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

    fn entries_in_row(&self, row: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        self.edges
            .get(&row)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(col, data)| (col.clone(), data)))
    }

    fn clear_row_and_column(&mut self, row: I, col: I) {
        if let Some(removed) = self.edges.remove(&row) {
            if let Some(back_edges) = self.back_edges.get_mut(&col) {
                back_edges.remove(&row);
                if back_edges.is_empty() {
                    self.back_edges.remove(&col);
                }
            }
            self.entry_count -= removed.len();
        }
        if let Some(removed) = self.back_edges.remove(&col) {
            for source in removed {
                if let Some(targets) = self.edges.get_mut(&source) {
                    if targets.remove(&col).is_some() {
                        self.entry_count -= 1;
                    }
                    if targets.is_empty() {
                        self.edges.remove(&source);
                    }
                }
            }
        }
    }

    fn entries_in_col(&self, col: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
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
        self.entry_count = 0;
    }

    fn len(&self) -> usize {
        self.entry_count
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

    #[test]
    fn test_len() {
        let mut matrix = AsymmetricHashAdjacencyMatrix::new();
        assert_eq!(matrix.len(), 0);
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.len(), 1);
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.len(), 1);
        matrix.insert(1, 0, "edge");
        assert_eq!(matrix.len(), 2);
        matrix.remove(0, 1);
        assert_eq!(matrix.len(), 1);
        matrix.clear();
        assert_eq!(matrix.len(), 0);
    }
}
