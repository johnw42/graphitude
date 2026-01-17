#[cfg(feature = "pathfinding")]
use std::{collections::HashMap, hash::Hash};

#[cfg(feature = "pathfinding")]
use pathfinding::num_traits::Zero;

pub trait Graph<'g> {
    type VertexId: Eq + Hash + Clone + 'g;
    type VertexData;
    type EdgeData;

    /// Get an iterator over the neighbors of a given vertex.
    fn neighbors(&'g self, from: &Self::VertexId) -> impl IntoIterator<Item = Self::VertexId>;

    /// Get the data associated with a vertex.
    fn vertex_data(&'g self, id: &Self::VertexId) -> &'g Self::VertexData;

    /// Get the data associated with an edge, if it exists.
    fn edge_data(&'g self, from: &Self::VertexId, to: &Self::VertexId) -> Option<&'g Self::EdgeData>;

    /// Check if there is an edge between two vertices.
    fn has_edge(&'g self, from: &Self::VertexId, to: &Self::VertexId) -> bool {
        self.edge_data(from, to).is_some()
    }

    /// Find shortest paths from a starting vertex to all other vertices using
    /// Dijkstra's algorithm.  Returns a map from each reachable vertex to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    fn shortest_paths<C: Zero + Ord + Copy>(
        &'g self,
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

pub trait GraphMut<'g>: Graph<'g> {
    /// Add a vertex with the given data to the graph, returning its VertexId.
    fn add_vertex(&'g mut self, data: Self::VertexData) -> Self::VertexId;

    /// Add an edge with the given data between two vertices.
    fn add_edge(
        &'g mut self,
        from: &Self::VertexId,
        to: &Self::VertexId,
        data: Self::EdgeData,
    );
}   