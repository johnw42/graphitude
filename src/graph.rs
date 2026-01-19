use double_vec_queue::Queue;
use std::collections::HashSet;

#[cfg(feature = "pathfinding")]
use std::{collections::HashMap, hash::Hash};

#[cfg(feature = "pathfinding")]
use pathfinding::num_traits::Zero;

// use crate::{edge_ref::EdgeRef, vertex_ref::VertexRef};

pub struct DfsIterator<'g, G: Graph + ?Sized> {
    graph: &'g G,
    visited: HashSet<G::VertexId>,
    stack: Vec<G::VertexId>,
}

impl<'g, G> Iterator for DfsIterator<'g, G>
where
    G: Graph,
{
    type Item = G::VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(vid) = self.stack.pop() {
            if self.visited.contains(&vid) {
                continue;
            }
            self.visited.insert(vid.clone());
            for eid in self.graph.edges_out(&vid) {
                let neighbor = self.graph.edge_target(&eid);
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
    G: Graph,
{
    type Item = G::VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(vid) = self.queue.pop() {
            if self.visited.contains(&vid) {
                continue;
            }
            self.visited.insert(vid.clone());
            for eid in self.graph.edges_out(&vid) {
                let neighbor = self.graph.edge_target(&eid);
                if !self.visited.contains(&neighbor) {
                    self.queue.push(neighbor);
                }
            }
            return Some(vid);
        }
        None
    }
}

pub trait Graph {
    type EdgeData;
    type EdgeId: Eq + Hash + Clone;
    type VertexData;
    type VertexId: Eq + Hash + Clone;

    fn is_directed(&self) -> bool {
        true
    }

    /// # Vertices

    /// Get the data associated with a vertex.
    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData;

    // fn vertex_ref(&self, id: &Self::VertexId) -> VertexRef<'_, Self> {
    //     VertexRef::new(self, id.clone())
    // }

    /// Get the number of vertices in the graph.
    fn num_vertices(&self) -> usize {
        self.vertex_ids().len()
    }

    /// # Edges

    /// Get the data associated with an edge.
    fn edge_data(&self, from: &Self::EdgeId) -> &Self::EdgeData;

    // fn edge_ref(&self, id: &Self::EdgeId) -> EdgeRef<'_, Self> {
    //     EdgeRef::new(self, id.clone())
    // }

    /// Get an iterator over the outgoing edges from a given vertex.
    fn edges_out(&self, from: &Self::VertexId) -> impl IntoIterator<Item = Self::EdgeId> {
        self.edge_ids().into_iter().filter(move |eid| {
            self.edge_source(eid) == *from || !self.is_directed() && self.edge_target(eid) == *from
        })
    }

    /// Get an iterator over the incoming edges to a given vertex.
    fn edges_in(&self, into: &Self::VertexId) -> impl IntoIterator<Item = Self::EdgeId> {
        self.edge_ids().into_iter().filter(move |eid| {
            self.edge_target(eid) == *into || !self.is_directed() && self.edge_source(eid) == *into
        })
    }

    /// Get an iterator over the edges between two vertices.
    fn edges_between(
        &self,
        from: &Self::VertexId,
        into: &Self::VertexId,
    ) -> impl IntoIterator<Item = Self::EdgeId> {
        self.edges_out(from).into_iter().filter(move |eid| {
            self.edge_source(eid) == *from && self.edge_target(eid) == *into
                || !self.is_directed()
                    && self.edge_source(eid) == *into
                    && self.edge_target(eid) == *from
        })
    }

    /// Get the source vertex of an edge.
    fn edge_source(&self, id: &Self::EdgeId) -> Self::VertexId;

    /// Get the target vertex of an edge.
    fn edge_target(&self, id: &Self::EdgeId) -> Self::VertexId;

    fn has_edge_out(&self, from: &Self::VertexId) -> bool {
        self.num_edges_out(from) > 0
    }

    fn has_edge_in(&self, into: &Self::VertexId) -> bool {
        self.num_edges_in(into) > 0
    }

    /// Check if there is an edge between two vertices.
    fn has_edge_between(&self, from: &Self::VertexId, into: &Self::VertexId) -> bool {
        self.num_edges_between(from, into) > 0
    }

    /// Get the number of edges in the graph.
    fn num_edges(&self) -> usize {
        self.edge_ids().len()
    }

    fn num_edges_in(&self, into: &Self::VertexId) -> usize {
        self.edges_in(into).into_iter().count()
    }

    fn num_edges_out(&self, from: &Self::VertexId) -> usize {
        self.edges_out(from).into_iter().count()
    }

    /// Get the number of edges in the graph.
    fn num_edges_between(&self, from: &Self::VertexId, into: &Self::VertexId) -> usize {
        self.edges_between(from, into).into_iter().count()
    }

    /// Get a vector of all VertexIds in the graph.
    fn vertex_ids(&self) -> Vec<Self::VertexId>;

    /// Get a vector of all edges in the graph.
    fn edge_ids(&self) -> Vec<Self::EdgeId> {
        let mut edges = Vec::new();
        for from in self.vertex_ids() {
            edges.extend(self.edges_out(&from));
        }
        edges
    }

    /// Perform a depth-first search starting from the given vertex.
    fn bfs(&self, start: &Self::VertexId) -> BfsIterator<'_, Self> {
        BfsIterator {
            graph: self,
            visited: HashSet::new(),
            queue: {
                let mut q = Queue::new();
                q.push(start.clone());
                q
            },
        }
    }

    /// Perform a depth-first search starting from the given vertex.
    fn dfs(&self, start: &Self::VertexId) -> DfsIterator<'_, Self> {
        DfsIterator {
            graph: self,
            visited: HashSet::new(),
            stack: vec![start.clone()],
        }
    }

    /// Find shortest paths from a starting vertex to all other vertices using
    /// Dijkstra's algorithm.  Returns a map from each reachable vertex to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    fn shortest_paths<C: Zero + Ord + Copy>(
        &self,
        start: &Self::VertexId,
        cost_fn: impl Fn(&Self::VertexId, &Self::VertexId) -> C,
    ) -> HashMap<Self::VertexId, (Vec<Self::VertexId>, C)> {
        use pathfinding::prelude::*;
        let parents: HashMap<Self::VertexId, (Self::VertexId, C)> =
            dijkstra_all(start, |v: &Self::VertexId| -> Vec<(Self::VertexId, C)> {
                self.edges_out(v)
                    .into_iter()
                    .map(|eid| (self.edge_target(&eid), cost_fn(v, &self.edge_target(&eid))))
                    .collect()
            });
        let mut result: HashMap<Self::VertexId, (Vec<Self::VertexId>, C)> = parents
            .iter()
            .map(|(k, (_, cost))| (k.clone(), (build_path(k, &parents), *cost)))
            .collect();
        result.insert(start.clone(), (vec![start.clone()], C::zero()));
        result
    }
}

pub trait GraphMutData: Graph {
    /// Get the data associated with an edge.
    fn edge_data_mut(&mut self, from: &Self::EdgeId) -> &mut Self::EdgeData;

    /// Get the data associated with an vertex.
    fn vertex_data_mut(&mut self, from: &Self::VertexId) -> &mut Self::VertexData;
}

pub trait GraphMutStructure: Graph {
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
