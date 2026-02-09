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
    /// Maps each row index to a map of column indices and their associated values.
    /// This represents the forward edges in a directed graph.
    entries: HashMap<I, HashMap<I, V>>,
    /// Maps each column index to a set of row indices that have entries in
    /// `entries`.  Invariant: `entries[row][col]` exists if and only if
    /// `reverse_entries[col]` contains `row`.  This allows efficient retrieval
    /// of all rows that have an entry for a given column.
    reverse_entries: HashMap<I, HashSet<I>>,
    /// Tracks the total number of entries in the adjacency matrix for efficient length queries.
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
            entries: HashMap::new(),
            reverse_entries: HashMap::new(),
            entry_count: 0,
        }
    }

    fn insert(&mut self, row: I, col: I, data: V) -> Option<V> {
        self.reverse_entries
            .entry(col.clone())
            .or_default()
            .insert(row.clone());
        let old_data = self.entries.entry(row).or_default().insert(col, data);
        if old_data.is_none() {
            self.entry_count += 1;
        }
        old_data
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        self.entries.get(&row).and_then(|m| m.get(&col))
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        if let Some(value) = self.entries.get_mut(&row).and_then(|m| m.remove(&col)) {
            if let Some(reverse_entries) = self.reverse_entries.get_mut(&col)
                && reverse_entries.remove(&row)
                && reverse_entries.is_empty()
            {
                self.reverse_entries.remove(&col);
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
        self.entries.iter().flat_map(|(row, targets)| {
            targets
                .iter()
                .map(|(col, data)| (row.clone(), col.clone(), data))
        })
    }

    fn into_iter(self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        self.entries.into_iter().flat_map(|(row, targets)| {
            targets
                .into_iter()
                .map(move |(col, data)| (row.clone(), col, data))
        })
    }

    fn entries_in_row(&self, row: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        self.entries
            .get(&row)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(col, data)| (col.clone(), data)))
    }

    fn clear_row_and_column(&mut self, row: I, col: I) {
        if let Some(removed) = self.entries.remove(&row) {
            if let Some(reverse_entries) = self.reverse_entries.get_mut(&col) {
                reverse_entries.remove(&row);
                if reverse_entries.is_empty() {
                    self.reverse_entries.remove(&col);
                }
            }
            self.entry_count -= removed.len();
        }
        if let Some(removed) = self.reverse_entries.remove(&col) {
            for source in removed {
                if let Some(targets) = self.entries.get_mut(&source) {
                    if targets.remove(&col).is_some() {
                        self.entry_count -= 1;
                    }
                    if targets.is_empty() {
                        self.entries.remove(&source);
                    }
                }
            }
        }
    }

    fn entries_in_col(&self, col: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let sources = self
            .reverse_entries
            .get(&col)
            .cloned()
            .unwrap_or_else(|| HashSet::new());
        sources.into_iter().filter_map(move |row| {
            self.entries
                .get(&row)
                .and_then(|targets| targets.get(&col))
                .map(|data| (row.clone(), data))
        })
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.reverse_entries.clear();
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
