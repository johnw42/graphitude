use std::{collections::HashSet, fmt::Debug, hash::Hash};

#[cfg(feature = "pathfinding")]
use std::{collections::HashMap, ops::Add};

#[cfg(feature = "dot")]
use {
    crate::dot::{parser, renderer},
    std::io,
};

use crate::{
    debug_graph_view::DebugGraphView,
    end_pair::EndPair,
    map_collector::MapCollector,
    path::Path,
    prelude::*,
    search::{BfsIterator, BfsIteratorWithPaths, DfsIterator, DfsIteratorWithPaths},
};

/// A trait representing a node or edge identifier in a graph.
///
/// This trait has no methods but serves as a marker for types that can be used
/// as identifiers.  This has the unfortunate side-effect of preventing the use
/// of primitive types (e.g., `usize`, `u32`, etc.) as identifiers, since they
/// do not implement this trait.  To work around this, you can define a newtype
/// wrapper around the primitive type and implement `GraphElementId` for the
/// newtype.
pub trait GraphElementId: Eq + Hash + Clone + Debug + Ord + Send + Sync {}

/// A trait representing a directed or undirected graph data structure.  Methods
/// that return iterators over nodes or edges return them in an unspecified
/// order unless otherwise noted.
pub trait Graph {
    type Directedness: Directedness;
    type EdgeMultiplicity: EdgeMultiplicity;
    type NodeData;
    type EdgeData;
    type NodeId: GraphElementId;
    type EdgeId: GraphElementId;

    /// Returns true if the graph is directed.
    fn is_directed(&self) -> bool {
        Self::Directedness::IS_DIRECTED
    }

    /// Returns true if the graph allows parallel edges between the same pair of nodes.
    fn allows_parallel_edges(&self) -> bool {
        Self::EdgeMultiplicity::ALLOWS_PARALLEL_EDGES
    }

    /// Checks if the graph is empty (has no nodes or edges).
    fn is_empty(&self) -> bool {
        if self.node_ids().next().is_none() {
            debug_assert!(self.edge_ids().next().is_none());
            true
        } else {
            false
        }
    }

    /// Creates a new graph view in which node and edge data are hidden.
    fn to_debug(&self) -> impl Graph + Debug {
        self.to_debug_with(|_| (), |_| ())
    }

    /// Creates a new graph view with custom debug formatting for nodes and edges.
    fn to_debug_with<N, E>(
        &self,
        node_fmt: impl FnMut(&Self::NodeData) -> N,
        edge_fmt: impl FnMut(&Self::EdgeData) -> E,
    ) -> impl Graph + Debug
    where
        N: Debug,
        E: Debug,
    {
        DebugGraphView::new(self, node_fmt, edge_fmt)
    }

    /// Writes a DOT representation of the graph to the given output.
    #[cfg(feature = "dot")]
    fn write_dot<D>(
        &self,
        generator: &D,
        output: &mut impl io::Write,
    ) -> Result<(), renderer::DotError<D::Error>>
    where
        D: renderer::DotGenerator<Self>,
        Self: Sized,
    {
        renderer::generate_dot_file(self, generator, output)
    }

    /// Generates a DOT representation of the graph as a String.
    #[cfg(feature = "dot")]
    fn to_dot_string<D>(&self, generator: &D) -> Result<String, renderer::DotError<D::Error>>
    where
        D: renderer::DotGenerator<Self>,
        Self: Sized,
    {
        let mut output = Vec::new();
        self.write_dot(generator, &mut output)?;
        Ok(String::from_utf8(output).expect("Generated DOT is not valid UTF-8"))
    }

    /// Creates a new path starting from the given starting node.  This is a
    /// convenience method to avoid having to import the `Path` type separately
    /// and specify its type argument explicity.
    fn new_path(&self, start: &Self::NodeId) -> Path<'_, Self> {
        Path::new(self, start.clone())
    }

    // Nodes

    /// Gets a vector of all NodeIds in the graph.
    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId>;

    /// Gets the data associated with a node.
    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData;

    /// Gets the number of nodes in the graph.
    fn num_nodes(&self) -> usize {
        self.node_ids().count()
    }

    /// Gets an iterator over the predecessors nodes of a given node, i.e.
    /// those nodes reachable by incoming edges.
    fn predecessors<'a, 'b: 'a>(
        &'a self,
        node: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::NodeId> + 'a {
        let mut visited = HashSet::new();
        self.edges_into(node).filter_map(move |eid| {
            let ends = self.edge_ends(&eid);
            let nid = if Self::Directedness::IS_DIRECTED {
                ends.into_first()
            } else {
                ends.into_other_value(node).into_inner()
            };
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    /// Gets an iterator over the successor nodes of a given node, i.e.
    /// those nodes reachable by outgoing edges.
    fn successors<'a, 'b: 'a>(
        &'a self,
        node: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::NodeId> + 'a {
        let mut visited = HashSet::new();
        self.edges_from(node).filter_map(move |eid| {
            let ends = self.edge_ends(&eid);
            let nid = if Self::Directedness::IS_DIRECTED {
                ends.into_second()
            } else {
                ends.into_other_value(node).into_inner()
            };
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    // Edges

    /// Gets the data associated with an edge.
    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData;

    /// Gets a vector of all edges in the graph.
    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_;

    /// Gets the ends of an edge as a pair of node IDs.  For directed edges, the
    /// first node ID is the source and the second is the target.  For
    /// undirected edges, the IDs are in sorted order but otherwise arbitrary.
    fn edge_ends(
        &self,
        id: &Self::EdgeId,
    ) -> <Self::Directedness as Directedness>::EndPair<Self::NodeId>;

    /// Gets an iterator over the outgoing edges from a given node.
    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.edge_ids().filter(|eid| {
            let (source, target) = self.edge_ends(eid).into_values();
            source == *from || !Self::Directedness::IS_DIRECTED && target == *from
        })
    }

    /// Gets an iterator over the incoming edges to a given node.
    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.edge_ids().filter(|eid| {
            let (source, target) = self.edge_ends(eid).into_values();
            target == *into || !Self::Directedness::IS_DIRECTED && source == *into
        })
    }

    /// Gets an iterator over the edges from one node into another.
    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.edge_ids().filter(move |eid| {
            let (edge_source, edge_target) = self.edge_ends(eid).into_values();
            (edge_source == *from && edge_target == *into)
                || (!Self::Directedness::IS_DIRECTED
                    && edge_source == *into
                    && edge_target == *from)
        })
    }

    /// Checks if there is at least one outgoing edge from the given node.
    fn has_edge_from(&self, from: &Self::NodeId) -> bool {
        self.edges_from(from).next().is_some()
    }

    /// Checks if there is at least one incoming edge to the given node.
    fn has_edge_into(&self, into: &Self::NodeId) -> bool {
        self.edges_into(into).next().is_some()
    }

    /// Checks if there at least one edge from one node to another.
    fn has_edge_from_into(&self, from: &Self::NodeId, into: &Self::NodeId) -> bool {
        self.edges_from_into(from, into).next().is_some()
    }

    /// Gets the number of edges in the graph.
    fn num_edges(&self) -> usize {
        self.edge_ids().count()
    }

    /// Gets the number of incoming edges to a given node.
    fn num_edges_into(&self, into: &Self::NodeId) -> usize {
        self.edges_into(into).count()
    }

    /// Gets the number of outgoing edges from a given node.
    fn num_edges_from(&self, from: &Self::NodeId) -> usize {
        self.edges_from(from).count()
    }

    /// Gets the number of edges from one node into another.
    fn num_edges_from_into(&self, from: &Self::NodeId, into: &Self::NodeId) -> usize {
        self.edges_from_into(from, into).count()
    }

    // Searches

    /// Performs a breadth-first search starting from the given node.
    fn bfs(&self, start: &Self::NodeId) -> BfsIterator<'_, Self> {
        self.bfs_multi(vec![start.clone()])
    }

    /// Performs a breadth-first search starting from the given nodes.
    fn bfs_multi(&self, start: Vec<Self::NodeId>) -> BfsIterator<'_, Self> {
        BfsIterator::new(self, start)
    }

    /// Performs a depth-first search starting from the given node.
    fn dfs(&self, start: &Self::NodeId) -> DfsIterator<'_, Self> {
        self.dfs_multi(vec![start.clone()])
    }

    /// Performs a depth-first search starting from the given node.
    fn dfs_multi(&self, start: Vec<Self::NodeId>) -> DfsIterator<'_, Self> {
        DfsIterator::new(self, start)
    }

    /// Performs a breadth-first search starting from the given node.
    fn bfs_with_paths(&self, start: &Self::NodeId) -> BfsIteratorWithPaths<'_, Self> {
        self.bfs_multi_with_paths(vec![start.clone()])
    }

    /// Performs a breadth-first search starting from the given nodes.
    fn bfs_multi_with_paths(&self, start: Vec<Self::NodeId>) -> BfsIteratorWithPaths<'_, Self> {
        BfsIteratorWithPaths::new(self, start)
    }

    /// Performs a depth-first search starting from the given node.
    fn dfs_with_paths(&self, start: &Self::NodeId) -> DfsIteratorWithPaths<'_, Self> {
        self.dfs_multi_with_paths(vec![start.clone()])
    }

    /// Performs a depth-first search starting from the given nodes.
    fn dfs_multi_with_paths(&self, start: Vec<Self::NodeId>) -> DfsIteratorWithPaths<'_, Self> {
        DfsIteratorWithPaths::new(self, start)
    }

    // Pathfinding

    /// Finds shortest paths from a starting node to all other nodes using
    /// Dijkstra's algorithm.  Returns a map from each reachable node to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    fn shortest_paths<C: Default + Ord + Copy + Add<Output = C>>(
        &self,
        start: &Self::NodeId,
        distance_fn: impl Fn(&Self::EdgeId) -> C,
    ) -> HashMap<Self::NodeId, (Path<'_, Self>, C)> {
        // Find shortest paths using Dijkstra's algorithm.

        let mut distances: HashMap<Self::NodeId, C> = HashMap::new();
        let mut predecessors: HashMap<Self::NodeId, (Self::EdgeId, Self::NodeId)> = HashMap::new();
        let mut unvisited: HashSet<Self::NodeId> = self.node_ids().collect();

        distances.insert(start.clone(), C::default());

        while !unvisited.is_empty() {
            // Find unvisited node with minimum distance
            let current = unvisited
                .iter()
                .filter_map(|node| distances.get(node).map(|&dist| (node.clone(), dist)))
                .min_by_key(|(_, dist)| *dist);

            let (current_node, current_dist) = match current {
                Some(pair) => pair,
                None => break, // No more reachable nodes
            };

            unvisited.remove(&current_node);

            // Update distances to neighbors
            for edge_id in self.edges_from(&current_node) {
                let ends = self.edge_ends(&edge_id);
                let neighbor = ends.other_value(&current_node).into_inner();
                if unvisited.contains(&neighbor) {
                    let edge_distance = distance_fn(&edge_id);
                    let new_dist = current_dist + edge_distance;

                    let should_update = distances
                        .get(&neighbor)
                        .is_none_or(|&old_dist| new_dist < old_dist);

                    if should_update {
                        distances.insert(neighbor.clone(), new_dist);
                        predecessors.insert(neighbor.clone(), (edge_id, current_node.clone()));
                    }
                }
            }
        }

        // Build paths from predecessors
        let mut result: HashMap<<Self as Graph>::NodeId, (Path<Self>, C)> = HashMap::new();
        for (node, &dist) in &distances {
            if node == start {
                result.insert(
                    start.clone(),
                    (Path::new(self, start.clone()), C::default()),
                );
            } else {
                let mut current = node.clone();

                let mut path_edges = Vec::new();
                while let Some(pred) = predecessors.get(&current) {
                    path_edges.push(pred.0.clone());
                    current = pred.1.clone();
                }

                let mut path = Path::new(self, start.clone());
                for edge_id in path_edges.iter().rev() {
                    path.add_edge(edge_id.clone());
                }

                result.insert(node.clone(), (path, dist));
            }
        }

        result
    }

    /// Returns true if the graph implementation is known to be very slow for
    /// large graphs (e.g., due to using a dense adjacency matrix).  This is mainly
    /// intended to be used to skip certain tests that would take an unreasonable
    /// amount of time to complete.
    #[doc(hidden)]
    fn is_very_slow(&self) -> bool {
        false
    }
}

/// A trait which is automatically implemented for directed graphs, providing
/// methods specific to directed graphs.
pub trait GraphDirected: Graph {
    /// Finds the strongly connected component containing the given node.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_component(&self, start: &Self::NodeId) -> Vec<Self::NodeId> {
        pathfinding::prelude::strongly_connected_component(start, |nid| {
            self.successors(nid).collect::<Vec<_>>()
        })
    }

    /// Partitions the graph into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components(&self) -> Vec<Vec<Self::NodeId>> {
        pathfinding::prelude::strongly_connected_components(
            &self.node_ids().collect::<Vec<_>>(),
            |nid| self.successors(nid).collect::<Vec<_>>(),
        )
    }

    /// Partitions nodes reachable from a starting point into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components_from(&self, start: &Self::NodeId) -> Vec<Vec<Self::NodeId>> {
        pathfinding::prelude::strongly_connected_components_from(start, |nid| {
            self.successors(nid).collect::<Vec<_>>()
        })
    }
}

impl<G> GraphDirected for G where G: Graph<Directedness = Directed> {}

/// A trait which is automatically implemented for undirected graphs, providing
/// methods specific to undirected graphs.
pub trait GraphUndirected: Graph {
    #[cfg(feature = "pathfinding")]
    fn connected_components(&self) -> Vec<HashSet<Self::NodeId>> {
        pathfinding::prelude::connected_components(&self.node_ids().collect::<Vec<_>>(), |nid| {
            self.successors(nid).collect::<Vec<_>>()
        })
    }
}

impl<G> GraphUndirected for G where G: Graph<Directedness = Undirected> {}

/// A trait for graphs that support mutation operations.
///
/// This trait extends [`Graph`] with methods for adding and removing nodes and edges.
/// All graph implementations that support modification should implement this trait.
pub trait GraphMut: Graph {
    /// Gets a mutable reference to the data associated with a node.
    fn node_data_mut(&mut self, id: &Self::NodeId) -> &mut Self::NodeData;

    /// Gets a mutable reference to the data associated with an edge.
    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData;

    /// Removes all nodes and edges from the graph.
    fn clear(&mut self) {
        for nid in self.node_ids().collect::<Vec<_>>() {
            self.remove_node(&nid);
        }
    }

    /// Adds a node with the given data to the graph, returning its `NodeId`.
    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId;

    /// Removes a node from the graph, returning its data.  Any edges
    /// connected to the node are also be removed.
    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData;

    /// Adds an edge with the given data between two nodes and returns the
    /// `EdgeId`.  Use [`Self::add_edge`] for graphs that do not
    /// support parallel edges.
    fn add_new_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> Self::EdgeId
    where
        Self: Graph<EdgeMultiplicity = MultipleEdges>,
    {
        match self.add_edge(from, into, data) {
            (eid, None) => eid,
            (_, Some(_)) => {
                unreachable!("Edge already exists between {:?} and {:?}", from, into)
            }
        }
    }

    /// Adds an edge if possible, or replaces the data of an existing edge.  If
    /// a new edge is added, returns its `EdgeId` and `None`.  If an edge
    /// already exists, returns `(new_id, Some((old_id, old_data)))`.  The
    /// `new_id` may be the same as the `old_id` if the graph implementation
    /// reuses edge IDs when replacing edges.
    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<(Self::EdgeId, Self::EdgeData)>);

    /// Remove an edge between two nodes, returning its data.
    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData;

    /// Removes all edges from one node into another.
    fn remove_edges_from_into(&mut self, from: &Self::NodeId, into: &Self::NodeId) {
        for eid in self.edges_from_into(from, into).collect::<Vec<_>>() {
            self.remove_edge(&eid);
        }
    }

    /// Reserves capacity for at least the given number of additional nodes
    /// and edges.  Does nothing by default.
    fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        let _ = additional_nodes;
        let _ = additional_edges;
    }

    /// Reserves the exact capacity for the given number of additional nodes
    /// and edges.  Does nothing by default.
    fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        let _ = additional_nodes;
        let _ = additional_edges;
    }

    /// Compacts internal storage used by the graph to minimize memory usage
    /// without reallocation.  Does nothing by default.  May invalidate existing
    /// NodeIds and EdgeIds.  If `node_map_collector` or `edge_map_collector` is
    /// provided, it will be used to collect mappings from old NodeIds and
    /// EdgeIds to new ones.
    fn compact(
        &mut self,
        node_map_collector: Option<&mut dyn MapCollector<Self::NodeId>>,
        edge_map_collector: Option<&mut dyn MapCollector<Self::EdgeId>>,
    ) {
        let _ = node_map_collector;
        let _ = edge_map_collector;
    }

    /// Shrinks internal storage used by the graph to fit its current size.
    /// Does nothing by default.
    fn shrink_to_fit(&mut self) {}

    /// Parses a DOT representation of a graph from a string, using the given
    /// graph builder to construct the graph.
    #[cfg(feature = "dot")]
    fn from_dot_string<B>(data: &str, builder: &mut B) -> Result<Self, parser::ParseError<B>>
    where
        Self: Sized,
        B: parser::GraphBuilder<Graph = Self>,
    {
        parser::parse_dot_into_graph(data, builder)
    }
}
