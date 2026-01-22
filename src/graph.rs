//! `Graph` and `GraphMut` are the core traits for working with graphs in this
//! library. `Graph` provides read-only access to the graph structure, while
//! `GraphMut` extends `Graph` with methods for modifying the graph.
//!
//! Mutating the data stored in vertices and edges is not provided directly
//! through these traits.  If you need to mutate the data, use interior
//! mutability (e.g., `RefCell`, `Cell`, `Mutex`, etc.) in your vertex and edge
//! data types.
//!
//! This module provides:
//!
//! - [`Graph`] trait: Core abstraction for graph data structures with support
//!   for vertices and edges
//! - [`GraphMut`] trait: Extension for mutable graph operations (add/remove
//!   vertices and edges)
//! - [`DfsIterator`]: Depth-first search iterator for graph traversal
//! - [`BfsIterator`]: Breadth-first search iterator for graph traversal
//!
//! # Features
//!
//! - Flexible vertex and edge data storage through associated types
//! - Support for both directed and undirected graphs
//! - Graph traversal algorithms: DFS, BFS
//! - Path finding utilities with Dijkstra's algorithm (requires `pathfinding`
//!   feature)
//! - Queries for vertices, edges, predecessors, and successors
use double_vec_queue::Queue;
use std::collections::HashSet;

#[cfg(feature = "pathfinding")]
use {
    pathfinding::num_traits::Zero,
    std::{collections::HashMap, hash::Hash},
};

use crate::{edge_ref::EdgeRef, vertex_ref::VertexRef};

pub struct DfsIterator<'g, G: Graph + ?Sized> {
    graph: &'g G,
    visited: HashSet<G::VertexId>,
    stack: Vec<G::VertexId>,
}

impl<'g, G> Iterator for DfsIterator<'g, G>
where
    G: Graph + ?Sized,
{
    type Item = G::VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(vid) = self.stack.pop() {
            if self.visited.contains(&vid) {
                continue;
            }
            self.visited.insert(vid.clone());
            for eid in self.graph.edges_out(vid.clone()) {
                let neighbor = self.graph.edge_target(eid);
                if !self.visited.contains(&neighbor) {
                    self.stack.push(neighbor);
                }
            }
            return Some(vid);
        }
        None
    }
}

pub struct BfsIterator<'g, G: Graph + ?Sized> {
    graph: &'g G,
    visited: HashSet<G::VertexId>,
    queue: Queue<G::VertexId>,
}

impl<'g, G> Iterator for BfsIterator<'g, G>
where
    G: Graph + ?Sized,
{
    type Item = G::VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(vid) = self.queue.pop() {
            if self.visited.contains(&vid) {
                continue;
            }
            self.visited.insert(vid.clone());
            for eid in self.graph.edges_out(vid.clone()) {
                let neighbor = self.graph.edge_target(eid);
                if !self.visited.contains(&neighbor) {
                    self.queue.push(neighbor);
                }
            }
            return Some(vid);
        }
        None
    }
}

/// A trait representing a graph data structure.
pub trait Graph {
    type EdgeData;
    type EdgeId: Eq + Hash + Clone;
    type VertexData;
    type VertexId: Eq + Hash + Clone;

    fn is_directed(&self) -> bool {
        true
    }

    /// # Vertices

    fn vertex(&self, id: Self::VertexId) -> VertexRef<'_, Self> {
        VertexRef::new(self, id)
    }

    fn verticies(&self) -> impl Iterator<Item = VertexRef<'_, Self>> + '_ {
        self.vertex_ids().into_iter().map(|vid| self.vertex(vid))
    }

    /// Gets a vector of all VertexIds in the graph.
    fn vertex_ids(&self) -> impl Iterator<Item = Self::VertexId>;

    /// Gets the data associated with a vertex.
    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData;

    /// Gets the number of vertices in the graph.
    fn num_vertices(&self) -> usize {
        self.vertex_ids().count()
    }

    /// Gets an iterator over the predacessors vertices of a given vertex, i.e.
    /// those vertices reachable by incoming edges.
    fn predacessors(&self, vertex: Self::VertexId) -> impl Iterator<Item = Self::VertexId> + '_ {
        let mut visited = HashSet::new();
        self.edges_in(vertex).filter_map(move |eid| {
            let vid = self.edge_source(eid);
            visited.insert(vid.clone()).then_some(vid)
        })
    }

    /// Gets an iterator over the successor vertices of a given vertex, i.e.
    /// those vertices reachable by outgoing edges.
    fn successors(&self, vertex: Self::VertexId) -> impl Iterator<Item = Self::VertexId> + '_ {
        let mut visited = HashSet::new();
        self.edges_out(vertex).filter_map(move |eid| {
            let vid = self.edge_target(eid);
            visited.insert(vid.clone()).then_some(vid)
        })
    }

    /// # Edges

    fn edge(&self, id: Self::EdgeId) -> EdgeRef<'_, Self> {
        EdgeRef::new(self, id)
    }

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> + '_ {
        self.edge_ids().into_iter().map(|eid| self.edge(eid))
    }

    /// Gets the data associated with an edge.
    fn edge_data(&self, from: &Self::EdgeId) -> &Self::EdgeData;

    /// Gets a vector of all edges in the graph.
    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        let mut edges = Vec::new();
        for from in self.vertex_ids() {
            edges.extend(self.edges_out(from));
        }
        edges.into_iter()
    }

    /// Gets an iterator over the outgoing edges from a given vertex.
    fn edges_out(&self, from: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edge_ids().filter(move |eid| {
            let (source, target) = self.edge_source_and_target(eid.clone());
            source == from || !self.is_directed() && target == from
        })
    }

    /// Gets an iterator over the incoming edges to a given vertex.
    fn edges_in(&self, into: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edge_ids().filter(move |eid| {
            let (source, target) = self.edge_source_and_target(eid.clone());
            target == into || !self.is_directed() && source == into
        })
    }

    /// Gets an iterator over the edges between two vertices.
    fn edges_between(
        &self,
        from: Self::VertexId,
        into: Self::VertexId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edges_out(from.clone()).filter(move |eid| {
            let edge_source = self.edge_source(eid.clone());
            let edge_target = self.edge_target(eid.clone());
            edge_source == from && edge_target == into
                || !self.is_directed() && edge_source == into && edge_target == from
        })
    }

    /// Gets the number of edges from one vertex to another.
    fn num_edges_between(&self, from: Self::VertexId, into: Self::VertexId) -> usize {
        self.edges_between(from, into).into_iter().count()
    }

    fn has_edge(&self, from: Self::VertexId, into: Self::VertexId) -> bool {
        self.num_edges_between(from, into) > 0
    }

    fn edge_source_and_target(&self, eid: Self::EdgeId) -> (Self::VertexId, Self::VertexId);

    /// Gets the source vertex of an edge.
    fn edge_source(&self, id: Self::EdgeId) -> Self::VertexId {
        self.edge_source_and_target(id).0
    }

    /// Gets the target vertex of an edge.
    fn edge_target(&self, id: Self::EdgeId) -> Self::VertexId {
        self.edge_source_and_target(id).1
    }

    fn has_edge_out(&self, from: &Self::VertexId) -> bool {
        self.num_edges_out(from.clone()) > 0
    }

    fn has_edge_in(&self, into: &Self::VertexId) -> bool {
        self.num_edges_in(into.clone()) > 0
    }

    /// Checks if there is an edge between two vertices.
    fn has_edge_between(&self, from: Self::VertexId, into: Self::VertexId) -> bool {
        self.num_edges_between(from, into) > 0
    }

    /// Gets the number of edges in the graph.
    fn num_edges(&self) -> usize {
        self.edge_ids().count()
    }

    fn num_edges_in(&self, into: Self::VertexId) -> usize {
        self.edges_in(into).into_iter().count()
    }

    fn num_edges_out(&self, from: Self::VertexId) -> usize {
        self.edges_out(from).into_iter().count()
    }

    /// Performs a breadth-first search starting from the given vertex.
    fn bfs(&self, start: Self::VertexId) -> BfsIterator<'_, Self> {
        self.bfs_multi(&[start])
    }

    /// Performs a breadth-first search starting from the given vertices.
    fn bfs_multi(&self, start: &[Self::VertexId]) -> BfsIterator<'_, Self> {
        BfsIterator {
            graph: self,
            visited: HashSet::new(),
            queue: start.iter().cloned().collect(),
        }
    }

    /// Performs a depth-first search starting from the given vertex.
    fn dfs(&self, start: Self::VertexId) -> DfsIterator<'_, Self> {
        self.dfs_multi(vec![start])
    }

    /// Performs a depth-first search starting from the given vertex.
    fn dfs_multi(&self, start: Vec<Self::VertexId>) -> DfsIterator<'_, Self> {
        DfsIterator {
            graph: self,
            visited: HashSet::new(),
            stack: start.into(),
        }
    }

    /// Finds shortest paths from a starting vertex to all other vertices using
    /// Dijkstra's algorithm.  Returns a map from each reachable vertex to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    fn shortest_paths<C: Zero + Ord + Copy>(
        &self,
        start: Self::VertexId,
        cost_fn: impl Fn(&Self::VertexId, &Self::VertexId) -> C,
    ) -> HashMap<Self::VertexId, (Vec<Self::VertexId>, C)> {
        use pathfinding::prelude::*;
        let parents: HashMap<Self::VertexId, (Self::VertexId, C)> =
            dijkstra_all(&start, |vid| -> Vec<(Self::VertexId, C)> {
                self.edges_out(vid.clone())
                    .map(|eid| {
                        let target_id = self.edge_target(eid);
                        (target_id.clone(), cost_fn(vid, &target_id))
                    })
                    .collect()
            });
        let mut result: HashMap<Self::VertexId, (Vec<Self::VertexId>, C)> = parents
            .iter()
            .map(|(k, (_, cost)): (&Self::VertexId, &(_, C))| {
                (k.clone(), (build_path(k, &parents), *cost))
            })
            .collect();
        result.insert(start.clone(), (vec![start.clone()], C::zero()));
        result
    }
}

pub trait GraphMut: Graph {
    /// Adds a vertex with the given data to the graph, returning its `VertexId`.
    fn add_vertex(&mut self, data: Self::VertexData) -> Self::VertexId;

    /// Removes a vertex from the graph, returning its data.  Any edges
    /// connected to the vertex are also be removed.
    fn remove_vertex(&mut self, id: &Self::VertexId) -> Self::VertexData;

    /// Adds an edge with the given data between two vertices and returns the
    /// `EdgeId`.  If an edge already exists between the two vertices, and the
    /// graph does not support parallel edges, the old edge is replaced and its
    /// data is returned as well.
    fn add_edge(
        &mut self,
        from: &Self::VertexId,
        to: &Self::VertexId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>);

    /// Remove an edge between two vertices, returning its data if it existed.
    fn remove_edge(&mut self, from: &Self::EdgeId) -> Option<Self::EdgeData>;
}
