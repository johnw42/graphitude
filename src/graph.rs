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
use std::iter::once;
use std::{collections::HashSet, fmt::Debug};

#[cfg(feature = "pathfinding")]
use {
    pathfinding::num_traits::Zero,
    std::{collections::HashMap, hash::Hash},
};

use crate::directedness::{Directed, Directedness, Undirected};
use crate::search::{BfsIterator, DfsIterator};
use crate::{edge_ref::EdgeRef, vertex_ref::VertexRef};

/// A trait representing a graph data structure.  Methods that return iterators
/// over vertices or edges return them in an unspecified order unless otherwise
/// noted.
pub trait Graph: Sized {
    type EdgeData;
    type EdgeId: Eq + Hash + Clone + Debug;
    type VertexData;
    type VertexId: Eq + Hash + Clone + Debug;
    type Directedness: Directedness;

    fn is_directed(&self) -> bool {
        Self::Directedness::is_directed()
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
        self.edges_into(vertex).filter_map(move |eid| {
            let vid = self.edge_source(eid);
            visited.insert(vid.clone()).then_some(vid)
        })
    }

    /// Gets an iterator over the successor vertices of a given vertex, i.e.
    /// those vertices reachable by outgoing edges.
    fn successors(&self, vertex: Self::VertexId) -> impl Iterator<Item = Self::VertexId> + '_ {
        let mut visited = HashSet::new();
        self.edges_from(vertex).filter_map(move |eid| {
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
            edges.extend(self.edges_from(from));
        }
        edges.into_iter()
    }

    /// Gets an iterator over the outgoing edges from a given vertex.
    fn edges_from(&self, from: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edge_ids().filter(move |eid| {
            let (source, target) = self.edge_ends(eid.clone());
            source == from || !self.is_directed() && target == from
        })
    }

    /// Gets an iterator over the incoming edges to a given vertex.
    fn edges_into(&self, into: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edge_ids().filter(move |eid| {
            let (source, target) = self.edge_ends(eid.clone());
            target == into || !self.is_directed() && source == into
        })
    }

    /// Gets an iterator over the edges between two vertices.
    fn edges_between(
        &self,
        from: Self::VertexId,
        into: Self::VertexId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edges_from(from.clone()).filter(move |eid| {
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
        self.edges_between(from, into).next().is_some()
    }

    /// Gets the source and target vertices of an edge.
    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::VertexId, Self::VertexId);

    /// Gets the source vertex of an edge.
    fn edge_source(&self, id: Self::EdgeId) -> Self::VertexId {
        self.edge_ends(id).0
    }

    /// Gets the target vertex of an edge.
    fn edge_target(&self, id: Self::EdgeId) -> Self::VertexId {
        self.edge_ends(id).1
    }

    fn has_edge_from(&self, from: &Self::VertexId) -> bool {
        self.edges_from(from.clone()).next().is_some()
    }

    fn has_edge_into(&self, into: &Self::VertexId) -> bool {
        self.edges_into(into.clone()).next().is_some()
    }

    /// Checks if there is an edge between two vertices.
    fn has_edge_between(&self, from: Self::VertexId, into: Self::VertexId) -> bool {
        self.edges_between(from, into).next().is_some()
    }

    /// Gets the number of edges in the graph.
    fn num_edges(&self) -> usize {
        self.edge_ids().count()
    }

    fn num_edges_into(&self, into: Self::VertexId) -> usize {
        self.edges_into(into).into_iter().count()
    }

    fn num_edges_from(&self, from: Self::VertexId) -> usize {
        self.edges_from(from).into_iter().count()
    }

    /// Performs a breadth-first search starting from the given vertex.
    fn bfs(&self, start: Self::VertexId) -> BfsIterator<'_, Self> {
        self.bfs_multi(vec![start])
    }

    /// Performs a breadth-first search starting from the given vertices.
    fn bfs_multi(&self, start: Vec<Self::VertexId>) -> BfsIterator<'_, Self> {
        BfsIterator::new(self, start)
    }

    /// Performs a depth-first search starting from the given vertex.
    fn dfs(&self, start: Self::VertexId) -> DfsIterator<'_, Self> {
        self.dfs_multi(vec![start])
    }

    /// Performs a depth-first search starting from the given vertex.
    fn dfs_multi(&self, start: Vec<Self::VertexId>) -> DfsIterator<'_, Self> {
        DfsIterator::new(self, start)
    }

    /// Finds shortest paths from a starting vertex to all other vertices using
    /// Dijkstra's algorithm.  Returns a map from each reachable vertex to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    fn shortest_paths<C: Zero + Ord + Copy>(
        &self,
        start: &Self::VertexId,
        cost_fn: impl Fn(&Self::EdgeId) -> C,
    ) -> HashMap<Self::VertexId, (Vec<Self::VertexId>, C)> {
        let parents: HashMap<Self::VertexId, (Self::VertexId, C)> =
            pathfinding::prelude::dijkstra_all(start, |vid| -> Vec<(Self::VertexId, C)> {
                self.edges_from(vid.clone())
                    .map(|eid| (self.edge_target(eid.clone()), cost_fn(&eid)))
                    .collect()
            });
        once((start.clone(), (vec![start.clone()], C::zero())))
            .chain(
                parents
                    .iter()
                    .map(|(k, (_, cost)): (&Self::VertexId, &(_, C))| {
                        (
                            k.clone(),
                            (pathfinding::prelude::build_path(k, &parents), *cost),
                        )
                    }),
            )
            .collect()
    }
}

/// A trait which is automatically implemented for directed graphs, providing
/// methods specific to directed graphs.
pub trait GraphDirected: Graph {
    /// Finds the strongly connected component containing the given vertex.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_component(&self, start: &Self::VertexId) -> Vec<Self::VertexId> {
        pathfinding::prelude::strongly_connected_component(start, |vid| {
            self.successors(vid.clone())
        })
    }

    /// Partitions the graph into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components(&self) -> Vec<Vec<Self::VertexId>> {
        pathfinding::prelude::strongly_connected_components(
            &self.vertex_ids().collect::<Vec<_>>(),
            |vid| self.successors(vid.clone()),
        )
    }

    /// Partitions nodes reachable from a starting point into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components_from(
        &self,
        start: &Self::VertexId,
    ) -> Vec<Vec<Self::VertexId>> {
        pathfinding::prelude::strongly_connected_components_from(start, |vid| {
            self.successors(vid.clone())
        })
    }
}

impl<G> GraphDirected for G where G: Graph<Directedness = Directed> {}

/// A trait which is automatically implemented for undirected graphs, providing
/// methods specific to undirected graphs.
pub trait GraphUndirected: Graph {
    #[cfg(feature = "pathfinding")]
    fn connected_components(&self) -> Vec<HashSet<Self::VertexId>> {
        pathfinding::prelude::connected_components(&self.vertex_ids().collect::<Vec<_>>(), |vid| {
            self.successors(vid.clone())
        })
    }
}

impl<G> GraphUndirected for G where G: Graph<Directedness = Undirected> {}

pub trait GraphMut: Graph {
    /// Removes all vertices and edges from the graph.
    fn clear(&mut self) {
        for vid in self.vertex_ids().collect::<Vec<_>>() {
            self.remove_vertex(&vid);
        }
    }

    /// Adds a vertex with the given data to the graph, returning its `VertexId`.
    fn add_vertex(&mut self, data: Self::VertexData) -> Self::VertexId;

    /// Removes a vertex from the graph, returning its data.  Any edges
    /// connected to the vertex are also be removed.
    fn remove_vertex(&mut self, id: &Self::VertexId) -> Self::VertexData;

    /// Adds an edge with the given data between two vertices and returns the
    /// `EdgeId`.  If an edge already exists between the two vertices, and the
    /// graph does not support parallel edges, the old edge is replaced.
    /// Use [`Self::add_or_replace_edge`] to get the old edge data as well.
    fn add_edge(
        &mut self,
        from: &Self::VertexId,
        to: &Self::VertexId,
        data: Self::EdgeData,
    ) -> Self::EdgeId {
        self.add_or_replace_edge(from, to, data).0
    }

    /// Adds an edge with the given data between two vertices and returns the
    /// `EdgeId`.  If an edge already exists between the two vertices, and the
    /// graph does not support parallel edges, the old edge is replaced and its
    /// data is returned as well.
    fn add_or_replace_edge(
        &mut self,
        from: &Self::VertexId,
        to: &Self::VertexId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>);

    /// Remove an edge between two vertices, returning its data if it existed.
    fn remove_edge(&mut self, from: &Self::EdgeId) -> Option<Self::EdgeData>;

    /// Copies all vertices and edges from another graph into this graph.
    fn copy_from<S>(&mut self, source: &S) -> HashMap<S::VertexId, Self::VertexId>
    where
        S: Graph<VertexData = Self::VertexData, EdgeData = Self::EdgeData>,
        Self::VertexData: Clone,
        Self::EdgeData: Clone,
    {
        self.copy_from_with(source, &mut Clone::clone, &mut Clone::clone)
    }

    /// Copies all vertices and edges from another graph into this graph,
    /// transforming the vertex and edge data using the provided mapping
    /// functions.
    fn copy_from_with<S, F, G>(
        &mut self,
        source: &S,
        map_vertex: &mut F,
        map_edge: &mut G,
    ) -> HashMap<S::VertexId, Self::VertexId>
    where
        S: Graph,
        Self::VertexData: Clone,
        Self::EdgeData: Clone,
        F: FnMut(&S::VertexData) -> Self::VertexData,
        G: FnMut(&S::EdgeData) -> Self::EdgeData,
    {
        let mut vertex_map = HashMap::new();
        for vid in source.vertex_ids() {
            let vdata = map_vertex(source.vertex_data(&vid));
            let new_vid = self.add_vertex(vdata);
            vertex_map.insert(vid, new_vid);
        }
        for eid in source.edge_ids() {
            let (from, to) = source.edge_ends(eid.clone());
            let edata = map_edge(source.edge_data(&eid));
            let new_from = vertex_map.get(&from).expect("missing vertex");
            let new_to = vertex_map.get(&to).expect("missing vertex");
            self.add_edge(new_from, new_to, edata);
        }

        vertex_map
    }

    /// Creates a mapping from edges in this graph to edges in another graph,
    /// based on a provided vertex mapping from [`Self::copy_from`] or
    /// [`Self::copy_from_with`].
    fn make_edge_map<S>(
        &self,
        source: &S,
        vertex_map: &HashMap<S::VertexId, Self::VertexId>,
    ) -> HashMap<S::EdgeId, Self::EdgeId>
    where
        S: Graph,
    {
        let mut edge_map = HashMap::new();
        for eid in source.edge_ids() {
            let (from1, to1) = source.edge_ends(eid.clone());
            if let Some(from2) = vertex_map.get(&from1)
                && let Some(to2) = vertex_map.get(&to1)
            {
                let eid2 = self
                    .edges_between(from2.clone(), to2.clone())
                    .find(|_| true)
                    .expect("missing edge");
                edge_map.insert(eid, eid2);
            }
        }
        edge_map
    }
}
