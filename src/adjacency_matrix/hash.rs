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

    fn get_mut(&mut self, row: I, col: I) -> Option<&mut V> {
        let (i1, i2) = sort_pair_if(!self.directedness.is_directed(), row, col);
        self.entries.get_mut(&i1).and_then(|m| m.get_mut(&i2))
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
                            if *i1 == row {
                                return None;
                            }
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
    use crate::adjacency_matrix_tests;

    adjacency_matrix_tests!(
        directed,
        HashAdjacencyMatrix<usize, T, Directed>);
    adjacency_matrix_tests!(
        undirected,  
        HashAdjacencyMatrix<usize, T, Undirected>);
}
