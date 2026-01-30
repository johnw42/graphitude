use std::collections::HashSet;
use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use crate::util::sort_pair;

use crate::adjacency_matrix::{AdjacencyMatrix, HashStorage, Symmetric};

/// Hash-based symmetric adjacency matrix for undirected graphs.
///
/// Stores only one entry per edge pair (row, col) where row <= col, saving memory
/// for undirected graphs. Lookups work for both (row, col) and (col, row).
#[derive(Clone, Debug)]
pub struct SymmetricHashAdjacencyMatrix<I, V>
where
    I: Hash + Eq + Clone + Ord + Debug,
{
    // Invariant: for any (row, col) in entries, row <= col.
    entries: HashMap<I, HashMap<I, V>>,
    // Invariant: for any (col, row) in reverse_entries, col >= row, and entries contains (row, col).
    reverse_entries: HashMap<I, HashSet<I>>,
}

impl<I, V> AdjacencyMatrix for SymmetricHashAdjacencyMatrix<I, V>
where
    I: Hash + Eq + Clone + Ord + Debug,
{
    type Index = I;
    type Value = V;
    type Symmetry = Symmetric;
    type Storage = HashStorage;

    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            reverse_entries: HashMap::new(),
        }
    }

    fn insert(&mut self, row: I, col: I, data: V) -> Option<V> {
        let (i1, i2) = sort_pair(row, col);
        self.reverse_entries
            .entry(i2.clone())
            .or_default()
            .insert(i1.clone());
        self.entries.entry(i1).or_default().insert(i2, data)
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        let (i1, i2) = sort_pair(row, col);
        self.entries.get(&i1).and_then(|m| m.get(&i2))
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        let (i1, i2) = sort_pair(row, col);
        self.entries.get_mut(&i1).and_then(|m| m.remove(&i2))
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (I, I, &'a V)>
    where
        V: 'a,
    {
        self.entries
            .iter()
            .flat_map(|(i1, targets)| targets.iter().map(|(i2, v)| (i1.clone(), i2.clone(), v)))
    }

    fn into_iter(self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        self.entries.into_iter().flat_map(|(i1, targets)| {
            targets.into_iter().map(move |(i2, v)| {
                debug_assert!(i1 <= i2);
                (i1.clone(), i2, v)
            })
        })
    }

    fn entry_indices(i1: Self::Index, i2: Self::Index) -> (Self::Index, Self::Index) {
        sort_pair(i1, i2)
    }

    fn entries_in_row<'a>(&'a self, row: I) -> impl Iterator<Item = (I, &'a V)>
    where
        V: 'a,
    {
        let forward_entries = self
            .entries
            .get(&row)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(i2, v)| (i2.clone(), v)));
        let backward_entries =
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
                });
        forward_entries.chain(backward_entries)
    }

    fn entries_in_col<'a>(&'a self, col: I) -> impl Iterator<Item = (I, &'a V)>
    where
        V: 'a,
    {
        self.entries_in_row(col)
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_insert_and_get() {
        let mut matrix = SymmetricHashAdjacencyMatrix::<i32, &str>::new();
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
        let mut matrix = SymmetricHashAdjacencyMatrix::<i32, &str>::new();
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.remove(1, 0), Some("edge"));
        assert_eq!(matrix.get(0, 1), None);
    }

    #[test]
    fn test_entry_at() {
        let mut matrix = SymmetricHashAdjacencyMatrix::<i32, &str>::new();
        matrix.insert(0, 1, "edge");
        let entry = matrix.entry_at(0, 0);
        assert_eq!(entry, None);
        let entry = matrix.entry_at(0, 1);
        assert_eq!(entry, Some((0, 1, &"edge")));
        let entry_rev = matrix.entry_at(1, 0);
        assert_eq!(entry_rev, Some((0, 1, &"edge")));
    }

    #[test]
    fn test_entries_in_row() {
        let mut matrix = SymmetricHashAdjacencyMatrix::<i32, &str>::new();
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
        let mut matrix = SymmetricHashAdjacencyMatrix::<i32, &str>::new();
        matrix.insert(0, 1, "a");
        matrix.insert(2, 0, "b");
        let entries: Vec<_> = matrix.entries_in_col(0).collect();
        assert_eq!(entries.len(), 2);
    }
}
