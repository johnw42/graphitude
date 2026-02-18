use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use derivative::Derivative;

use crate::{
    DirectednessTrait,
    adjacency_matrix::{AdjacencyMatrix, HashStorage, trait_def::format_debug},
};

/// Hash-based asymmetric adjacency matrix for directed graphs.
///
/// Stores separate forward and backward edge indices for efficient traversal
/// in both directions.
#[derive(Clone, Derivative)]
#[derivative(Default(bound = "D: Default"))]
pub struct HashAdjacencyMatrix<V, D> {
    /// Maps each row index to a map of column indices and their associated values.
    /// This represents the forward edges in a directed graph.
    entries: HashMap<usize, HashMap<usize, V>>,
    /// Maps each column index to a set of row indices that have entries in
    /// `entries`.  Invariant: `entries[row][col]` exists if and only if
    /// `reverse_entries[col]` contains `row`.  This allows efficient retrieval
    /// of all rows that have an entry for a given column.
    reverse_entries: HashMap<usize, HashSet<usize>>,
    size_bound: usize,
    directedness: D,
}

impl<V, D> AdjacencyMatrix for HashAdjacencyMatrix<V, D>
where
    D: DirectednessTrait + Default,
{
    type Value = V;
    type Directedness = D;
    type Storage = HashStorage;

    fn with_size(size: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(size),
            reverse_entries: HashMap::with_capacity(size),
            size_bound: 0,
            directedness: D::default(),
        }
    }

    fn size_bound(&self) -> usize {
        self.size_bound
    }

    fn insert(&mut self, row: usize, col: usize, data: V) -> Option<V> {
        self.size_bound = self.size_bound.max(row.max(col) + 1);

        let (i1, i2) = self.directedness.sort_pair((row, col));

        self.reverse_entries.entry(i2).or_default().insert(i1);

        self.entries.entry(i1).or_default().insert(i2, data)
    }

    fn get(&self, row: usize, col: usize) -> Option<&V> {
        let (i1, i2) = self.directedness.sort_pair((row, col));
        self.entries.get(&i1).and_then(|m| m.get(&i2))
    }

    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut V> {
        let (i1, i2) = self.directedness.sort_pair((row, col));
        self.entries.get_mut(&i1).and_then(|m| m.get_mut(&i2))
    }

    fn remove(&mut self, row: usize, col: usize) -> Option<V> {
        let (i1, i2) = self.directedness.sort_pair((row, col));
        let targets = self.entries.get_mut(&i1)?;
        let value = targets.remove(&i2);
        if targets.is_empty() {
            self.entries.remove(&i1);
        }
        if let Some(sources) = self.reverse_entries.get_mut(&i2)
            && sources.remove(&i1)
            && sources.is_empty()
        {
            self.reverse_entries.remove(&i2);
        }
        value
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, usize, &'a V)>
    where
        V: 'a,
    {
        self.entries
            .iter()
            .flat_map(|(row, targets)| targets.iter().map(|(col, data)| (*row, *col, data)))
    }

    fn into_iter(self) -> impl Iterator<Item = (usize, usize, V)> {
        self.entries
            .into_iter()
            .flat_map(|(row, targets)| targets.into_iter().map(move |(col, data)| (row, col, data)))
    }

    fn entries_in_row(&self, row: usize) -> impl Iterator<Item = (usize, &'_ V)> + '_ {
        let forward_entries = self
            .entries
            .get(&row)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(i2, v)| (*i2, v)));

        let backward_entries = if self.directedness.is_directed() {
            None
        } else {
            Some(
                self.reverse_entries
                    .get(&row)
                    .into_iter()
                    .flat_map(move |sources| {
                        sources.iter().filter_map(move |i1| {
                            if *i1 == row {
                                return None;
                            }
                            self.entries
                                .get(i1)
                                .and_then(|targets| targets.get(&row))
                                .map(|v| (*i1, v))
                        })
                    }),
            )
        };

        forward_entries.chain(backward_entries.into_iter().flatten())
    }

    fn entries_in_col(&self, col: usize) -> impl Iterator<Item = (usize, &'_ V)> + '_ {
        let (directed, undirected) = if self.directedness.is_directed() {
            let sources = self.reverse_entries.get(&col).cloned().unwrap_or_default();
            (
                Some(sources.into_iter().filter_map(move |row| {
                    self.entries
                        .get(&row)
                        .and_then(|targets| targets.get(&col))
                        .map(|data| (row, data))
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

    fn clear_row_and_column(&mut self, row: usize, col: usize) {
        if self.directedness.is_directed() {
            if self.entries.remove(&row).is_some()
                && let Some(reverse_entries) = self.reverse_entries.get_mut(&col)
            {
                reverse_entries.remove(&row);
                if reverse_entries.is_empty() {
                    self.reverse_entries.remove(&col);
                }
            }
            if let Some(removed) = self.reverse_entries.remove(&col) {
                for source in removed {
                    if let Some(targets) = self.entries.get_mut(&source) {
                        targets.remove(&col);
                        if targets.is_empty() {
                            self.entries.remove(&source);
                        }
                    }
                }
            }
        } else {
            let mut to_remove = Vec::new();
            for key1 in [row, col] {
                for (key2, _) in self.entries_in_row(key1) {
                    to_remove.push((key1, key2));
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
    }

    fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
        self.reverse_entries.reserve(additional);
    }

    fn reserve_exact(&mut self, additional: usize) {
        // HashMap doesn't have a reserve_exact method, so we can just call reserve.
        self.reserve(additional);
    }

    fn shrink_to_fit(&mut self) {
        self.entries.shrink_to_fit();
        self.reverse_entries.shrink_to_fit();
        let mut size_bound = 0;
        dbg!(self.entries.len(), self.reverse_entries.len());
        for (index, targets) in self.entries.iter_mut() {
            size_bound = size_bound.max(index + 1);
            targets.shrink_to_fit();
        }
        for (index, sources) in self.reverse_entries.iter_mut() {
            size_bound = size_bound.max(index + 1);
            sources.shrink_to_fit();
        }
        self.size_bound = size_bound;
    }
}

impl<V, D> Debug for HashAdjacencyMatrix<V, D>
where
    V: Debug,
    D: DirectednessTrait + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "HashAdjacencyMatrix")
    }
}

#[cfg(test)]
mod tests {
    use crate::adjacency_matrix_tests;

    adjacency_matrix_tests!(
        directed,
        HashAdjacencyMatrix<T, Directed>);
    adjacency_matrix_tests!(
        undirected,  
        HashAdjacencyMatrix<T, Undirected>);
}
