use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

use crate::util::sort_pair;

use crate::adjacency_matrix::{AdjacencyMatrix, HashStorage, Symmetric};

#[derive(Clone, Debug)]
pub struct SymmetricHashAdjacencyMatrix<K, N>
where
    K: Hash + Eq + Clone + Ord + Debug,
{
    edges: HashMap<K, HashMap<K, *mut N>>,
}

impl<K, N> AdjacencyMatrix for SymmetricHashAdjacencyMatrix<K, N>
where
    K: Hash + Eq + Clone + Ord + Debug,
{
    type Key = K;
    type Value = N;
    type Symmetry = Symmetric;
    type Storage = HashStorage;

    fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    fn insert(&mut self, k1: K, k2: K, data: N) -> Option<N> {
        let to_insert = Box::leak(Box::new(data)) as *mut N;
        let old_data = self
            .edges
            .entry(k1.clone())
            .or_default()
            .insert(k2.clone(), to_insert);
        self.edges.entry(k2).or_default().insert(k1, to_insert);
        //dbg!(self.edges.iter().flat_map(|(k1, v)| v.keys().map(|k2| (k1,k2)).collect::<Vec<_>>()).collect::<Vec<_>>());
        old_data.map(|d| unsafe { std::ptr::read(d) })
    }

    fn get(&self, from: K, into: K) -> Option<&N> {
        let (k1, k2) = sort_pair(from, into);
        self.edges
            .get(&k1)
            .and_then(|m| m.get(&k2))
            .map(|ptr| unsafe { &**ptr })
    }

    fn remove(&mut self, from: K, into: K) -> Option<N> {
        let (k1, k2) = sort_pair(from, into);
        self.edges
            .get_mut(&k1)
            .and_then(|m| m.remove(&k2))
            .map(|v| unsafe { std::ptr::read(v) })
    }

    fn edges<'a>(&'a self) -> impl Iterator<Item = (K, K, &'a N)>
    where
        N: 'a,
    {
        self.edges.iter().flat_map(|(k1, targets)| {
            targets.iter().filter_map(|(k2, v)| {
                (*k1 <= *k2).then(|| (k1.clone(), k2.clone(), unsafe { &**v }))
            })
        })
    }


    fn edge_ends(k1: Self::Key, k2: Self::Key) -> (Self::Key, Self::Key) {
        sort_pair(k1, k2)
    }

    fn edges_from<'a>(&'a self, k1: K) -> impl Iterator<Item = (K, &'a N)>
    where
        N: 'a,
    {
        self.edges
            .get(&k1)
            .into_iter()
            .flat_map(|targets| targets.iter().map(|(k2, v)| (k2.clone(), unsafe { &**v })))
    }

    fn edges_into<'a>(&'a self, into: K) -> impl Iterator<Item = (K, &'a N)>
    where
        N: 'a,
    {
        self.edges_from(into)
    }
}

impl<K, N> Drop for SymmetricHashAdjacencyMatrix<K, N>
where
    K: Hash + Eq + Clone + Ord + Debug,
{
    fn drop(&mut self) {
        for (k1, inner_map) in self.edges.iter() {
            for (k2, data_ptr) in inner_map {
                if k1 <= k2 {
                    unsafe {
                        drop(Box::from_raw(*data_ptr));
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut matrix = SymmetricHashAdjacencyMatrix::new();
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
    fn test_edge_between() {
        let mut matrix = SymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "edge");
        let edge = matrix.edge_between(0, 0);
        assert_eq!(edge, None);
        let edge = matrix.edge_between(0, 1);
        assert_eq!(edge, Some((0, 1, &"edge")));
        let edge_rev = matrix.edge_between(1, 0);
        assert_eq!(edge_rev, Some((0, 1, &"edge")));
    }

    #[test]
    fn test_edges_from() {
        let mut matrix = SymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(0, 2, "b");
        matrix.insert(1, 2, "c");
        dbg!(&matrix);
        let edges: Vec<_> = matrix.edges_from(0).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(to, _)| *to == 1));
        assert!(edges.iter().any(|(to, _)| *to == 2));
        let edges: Vec<_> = matrix.edges_from(1).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(to, _)| *to == 0));
        assert!(edges.iter().any(|(to, _)| *to == 2));
        let edges: Vec<_> = matrix.edges_from(2).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(to, _)| *to == 0));
        assert!(edges.iter().any(|(to, _)| *to == 1));
    }

    #[test]
    fn test_edges_into() {
        let mut matrix = SymmetricHashAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(2, 0, "b");
        let edges: Vec<_> = matrix.edges_into(0).collect();
        assert_eq!(edges.len(), 2);
    }
}
