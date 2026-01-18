use double_vec_queue::Queue;
use std::collections::HashSet;

#[cfg(feature = "pathfinding")]
use std::{collections::HashMap, hash::Hash};

#[cfg(feature = "pathfinding")]
use pathfinding::num_traits::Zero;

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
            for neighbor in self.graph.neighbors(&vid) {
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
            for neighbor in self.graph.neighbors(&vid) {
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
    type VertexId: Eq + Hash + Clone;
    type VertexData;
    type EdgeData;

    /// Get an iterator over the neighbors of a given vertex.
    fn neighbors(&self, from: &Self::VertexId) -> impl IntoIterator<Item = Self::VertexId>;

    /// Get the data associated with a vertex.
    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData;

    /// Get the data associated with an edge, if it exists.
    fn edge_data(&self, from: &Self::VertexId, to: &Self::VertexId) -> Option<&Self::EdgeData>;

    /// Check if there is an edge between two vertices.
    fn has_edge(&self, from: &Self::VertexId, to: &Self::VertexId) -> bool {
        self.edge_data(from, to).is_some()
    }

    /// Get the number of vertices in the graph.
    fn num_vertices(&self) -> usize {
        self.vertex_ids().len()
    }

    /// Get the number of edges in the graph.
    fn num_edges(&self) -> usize {
        self.edge_ids().len()
    }

    /// Get a vector of all VertexIds in the graph.
    fn vertex_ids(&self) -> Vec<Self::VertexId>;

    /// Get a vector of all edges in the graph as (from, to) VertexId pairs.
    fn edge_ids(&self) -> Vec<(Self::VertexId, Self::VertexId)> {
        let mut edges = Vec::new();
        for from in self.vertex_ids() {
            for to in self.neighbors(&from) {
                edges.push((from.clone(), to));
            }
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
                self.neighbors(v)
                    .into_iter()
                    .map(|n| (n.clone(), cost_fn(v, &n)))
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

pub trait GraphMut: Graph {
    /// Add a vertex with the given data to the graph, returning its VertexId.
    fn add_vertex(&mut self, data: Self::VertexData) -> Self::VertexId;

    /// Add an edge with the given data between two vertices.
    fn add_edge(&mut self, from: &Self::VertexId, to: &Self::VertexId, data: Self::EdgeData);
}
