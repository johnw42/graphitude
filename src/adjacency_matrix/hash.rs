use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use crate::{
    Directed, DirectednessTrait, Undirected,
    adjacency_matrix::{AdjacencyMatrix, HashStorage},
    util::sort_pair_if,
};

pub type AsymmetricHashAdjacencyMatrix<I, V> = HashAdjacencyMatrix<I, V, Directed>;
pub type SymmetricHashAdjacencyMatrix<I, V> = HashAdjacencyMatrix<I, V, Undirected>;

/// Hash-based asymmetric adjacency matrix for directed graphs.
///
/// Stores separate forward and backward edge indices for efficient traversal
/// in both directions.
#[derive(Clone, Debug)]
pub struct HashAdjacencyMatrix<I, V, D> {
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
    directedness: D,
}

impl<I, V, D> AdjacencyMatrix for HashAdjacencyMatrix<I, V, D>
where
    I: Hash + Eq + Clone + Ord + Debug,
    D: DirectednessTrait + Default,
{
    type Index = I;
    type Value = V;
    type Directedness = D;
    type Storage = HashStorage;

    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            reverse_entries: HashMap::new(),
            entry_count: 0,
            directedness: D::default(),
        }
    }

    fn insert(&mut self, row: I, col: I, data: V) -> Option<V> {
        let (i1, i2) = sort_pair_if(!self.directedness.is_directed(), row, col);

        self.reverse_entries
            .entry(i2.clone())
            .or_default()
            .insert(i1.clone());

        let old_data = self.entries.entry(i1).or_default().insert(i2, data);
        if old_data.is_none() {
            self.entry_count += 1;
        }
        old_data
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        let (i1, i2) = sort_pair_if(!self.directedness.is_directed(), row, col);
        self.entries.get(&i1).and_then(|m| m.get(&i2))
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        let (i1, i2) = sort_pair_if(!self.directedness.is_directed(), row, col);
        let value = self.entries.get_mut(&i1).and_then(|m| m.remove(&i2))?;
        if let Some(sources) = self.reverse_entries.get_mut(&i2)
            && sources.remove(&i1)
            && sources.is_empty()
        {
            self.reverse_entries.remove(&i2);
        }
        self.entry_count -= 1;
        Some(value)
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
        let forward_entries = self
            .entries
            .get(&row)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(i2, v)| (i2.clone(), v)));

        let backward_entries = if self.directedness.is_directed() {
            None
        } else {
            Some(
                self.reverse_entries
                    .get(&row)
                    .into_iter()
                    .flat_map(move |sources| {
                        let row = row.clone();
                        sources.iter().filter_map(move |i1| {
                            self.entries
                                .get(i1)
                                .and_then(|targets| targets.get(&row))
                                .map(|v| (i1.clone(), v))
                        })
                    }),
            )
        };

        forward_entries.chain(backward_entries.into_iter().flatten())
    }

    fn entries_in_col(&self, col: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let (directed, undirected) = if self.directedness.is_directed() {
            let sources = self
                .reverse_entries
                .get(&col)
                .cloned()
                .unwrap_or_else(|| HashSet::new());
            (
                Some(sources.into_iter().filter_map(move |row| {
                    self.entries
                        .get(&row)
                        .and_then(|targets| targets.get(&col))
                        .map(|data| (row.clone(), data))
                })),
                None,
            )
        } else {
            (None, Some(self.entries_in_row(col)))
        };
        directed
            .into_iter()
            .flatten()
            .chain(undirected.into_iter().flatten())
    }

    fn clear_row_and_column(&mut self, row: I, col: I) {
        if self.directedness.is_directed() {
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
        } else {
            let mut to_remove = Vec::with_capacity(self.entry_count);
            for key1 in [row, col] {
                for (key2, _) in self.entries_in_row(key1.clone()) {
                    to_remove.push((key1.clone(), key2));
                }
            }
            for (key1, key2) in to_remove {
                self.remove(key1, key2);
            }
        }
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
    mod directed {
        use super::super::*;

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

    mod undirected {
        use crate::edge_ends::EdgeEnds;

        use super::super::*;

        type SortedPair<T> = EdgeEnds<T, Undirected>;

        #[test]
        fn test_matrix_insert_and_get() {
            let mut matrix = SymmetricHashAdjacencyMatrix::new();
            matrix.insert(0, 0, "a");
            assert_eq!(matrix.get(0, 0), Some(&"a"));
            matrix.insert(1, 0, "b");
            assert_eq!(matrix.get(1, 0), Some(&"b"));
            matrix.insert(2, 0, "c");
            assert_eq!(matrix.get(2, 0), Some(&"c"));
            matrix.insert(6, 7, "d");
            assert_eq!(matrix.get(6, 7), Some(&"d"));
        }

        #[test]
        fn test_insert_and_get() {
            let mut matrix = SymmetricHashAdjacencyMatrix::<i32, &str>::new();
            matrix.insert(0, 1, "edge");
            assert_eq!(matrix.get(0, 1), Some(&"edge"));
            assert_eq!(matrix.get(1, 0), Some(&"edge"));
        }

        #[test]
        fn test_remove() {
            let mut matrix = SymmetricHashAdjacencyMatrix::new();
            matrix.insert(0, 1, "edge");
            assert_eq!(matrix.remove(1, 0), Some("edge"));
            assert_eq!(matrix.get(0, 1), None);
        }

        #[test]
        fn test_entry_at() {
            let mut matrix = SymmetricHashAdjacencyMatrix::new();
            matrix.insert(0, 1, "edge");
            let entry = matrix.entry_at(0, 0);
            assert_eq!(entry, None);
            let entry = matrix.entry_at(0, 1);
            assert_eq!(entry, Some((SortedPair::from((0, 1)), &"edge")));
            let entry_rev = matrix.entry_at(1, 0);
            assert_eq!(entry_rev, Some((SortedPair::from((0, 1)), &"edge")));
        }

        #[test]
        fn test_entries_in_row() {
            let mut matrix = SymmetricHashAdjacencyMatrix::new();
            matrix.insert(0, 1, "a");
            matrix.insert(0, 2, "b");
            matrix.insert(1, 2, "c");
            dbg!(&matrix);
            let entries: Vec<_> = matrix.entries_in_row(0).collect();
            assert_eq!(entries.len(), 2);
            assert!(entries.iter().any(|(to, _)| *to == 1));
            assert!(entries.iter().any(|(to, _)| *to == 2));
            let entries: Vec<_> = matrix.entries_in_row(1).collect();
            assert_eq!(entries.len(), 2);
            assert!(entries.iter().any(|(to, _)| *to == 0));
            assert!(entries.iter().any(|(to, _)| *to == 2));
            let entries: Vec<_> = matrix.entries_in_row(2).collect();
            assert_eq!(entries.len(), 2);
            assert!(entries.iter().any(|(to, _)| *to == 0));
            assert!(entries.iter().any(|(to, _)| *to == 1));
        }

        #[test]
        fn test_entries_in_col() {
            let mut matrix = SymmetricHashAdjacencyMatrix::new();
            matrix.insert(0, 1, "a");
            matrix.insert(2, 0, "b");
            let entries: Vec<_> = matrix.entries_in_col(0).collect();
            assert_eq!(entries.len(), 2);
        }

        #[test]
        fn test_len() {
            let mut matrix = SymmetricHashAdjacencyMatrix::new();
            assert_eq!(matrix.len(), 0);
            matrix.insert(0, 1, "edge");
            assert_eq!(matrix.get(0, 1), Some(&"edge"));
            assert_eq!(matrix.get(1, 0), Some(&"edge"));
            assert_eq!(matrix.len(), 1);
            matrix.insert(1, 0, "edge");
            assert_eq!(matrix.get(0, 1), Some(&"edge"));
            assert_eq!(matrix.get(1, 0), Some(&"edge"));
            assert_eq!(matrix.len(), 1);
            matrix.insert(2, 2, "loop");
            assert_eq!(matrix.len(), 2);
            matrix.remove(1, 0);
            assert_eq!(matrix.get(0, 1), None);
            assert_eq!(matrix.get(1, 0), None);
            assert_eq!(matrix.len(), 1);
            matrix.clear();
            assert_eq!(matrix.len(), 0);
        }

        #[test]
        fn test_clear_row_and_column() {
            let mut matrix = SymmetricHashAdjacencyMatrix::new();

            // Build a symmetric matrix
            // Edges involving node 2: (0,2), (1,2), (2,3), (2,4)
            // These should all be removed when clearing row/col 2
            matrix.insert(0, 2, "edge_0_2");
            matrix.insert(1, 2, "edge_1_2");
            matrix.insert(2, 3, "edge_2_3");
            matrix.insert(2, 4, "edge_2_4");

            // Add some other edges that should remain
            matrix.insert(0, 1, "edge_0_1");
            matrix.insert(3, 4, "edge_3_4");

            assert_eq!(matrix.len(), 6);

            // Clear row 2 and column 2 (which are the same in symmetric matrix)
            matrix.clear_row_and_column(2, 2);

            // Should have removed all edges involving node 2
            assert_eq!(matrix.len(), 2);

            // Verify edges involving node 2 are gone
            assert_eq!(matrix.get(0, 2), None);
            assert_eq!(matrix.get(2, 0), None);
            assert_eq!(matrix.get(1, 2), None);
            assert_eq!(matrix.get(2, 1), None);
            assert_eq!(matrix.get(2, 3), None);
            assert_eq!(matrix.get(3, 2), None);
            assert_eq!(matrix.get(2, 4), None);
            assert_eq!(matrix.get(4, 2), None);

            // Verify other edges remain
            assert_eq!(matrix.get(0, 1), Some(&"edge_0_1"));
            assert_eq!(matrix.get(1, 0), Some(&"edge_0_1"));
            assert_eq!(matrix.get(3, 4), Some(&"edge_3_4"));
            assert_eq!(matrix.get(4, 3), Some(&"edge_3_4"));
        }

        #[test]
        fn test_clear_row_and_column_with_different_indices() {
            let mut matrix = SymmetricHashAdjacencyMatrix::new();

            // Build a graph with edges
            matrix.insert(0, 1, "a");
            matrix.insert(0, 2, "b");
            matrix.insert(1, 2, "c");
            matrix.insert(1, 3, "d");
            matrix.insert(2, 3, "e");
            matrix.insert(3, 4, "f");

            assert_eq!(matrix.len(), 6);

            // Clear row 1 and column 2 (should remove all edges involving nodes 1 or 2)
            matrix.clear_row_and_column(1, 2);

            // Edges involving node 1: (0,1), (1,2), (1,3)
            // Edges involving node 2: (0,2), (1,2), (2,3)
            // Union: (0,1), (0,2), (1,2), (1,3), (2,3) = 5 edges removed
            // Remaining: (3,4)
            assert_eq!(matrix.len(), 1);
            assert_eq!(matrix.get(3, 4), Some(&"f"));

            // Verify all edges involving 1 or 2 are gone
            assert_eq!(matrix.get(0, 1), None);
            assert_eq!(matrix.get(0, 2), None);
            assert_eq!(matrix.get(1, 2), None);
            assert_eq!(matrix.get(1, 3), None);
            assert_eq!(matrix.get(2, 3), None);
        }
    }
}
