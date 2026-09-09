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
    iter::lazy_exact_sized::LazyExactSizedIterator,
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
        if self.nodes().next().is_none() {
            debug_assert!(self.edges().next().is_none());
            true
        } else {
            false
        }
    }

    // =========================================================================
    // Nodes
    // =========================================================================

    /// Gets a vector of all NodeIds in the graph.
    fn nodes(&self) -> impl ExactSizeIterator<Item = Self::NodeId>;

    /// Gets the data associated with a node.
    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData;

    /// Gets the number of nodes in the graph.
    fn num_nodes(&self) -> usize {
        self.nodes().len()
    }

    /// Gets an iterator over the predecessors nodes of a given node, i.e.
    /// those nodes reachable by incoming edges.
    fn predecessors<'g>(
        &'g self,
        node: &'g Self::NodeId,
    ) -> impl ExactSizeIterator<Item = Self::NodeId> + 'g {
        let mut visited = HashSet::new();
        LazyExactSizedIterator::new(self.edges_into(node).filter_map(move |eid| {
            let ends = self.edge_ends(&eid);
            let nid = if Self::Directedness::IS_DIRECTED {
                ends.into_first()
            } else {
                ends.into_other_value(node).into_inner()
            };
            visited.insert(nid.clone()).then_some(nid)
        }))
    }

    /// Gets an iterator over the successor nodes of a given node, i.e.
    /// those nodes reachable by outgoing edges.
    fn successors<'g>(
        &'g self,
        node: &'g Self::NodeId,
    ) -> impl ExactSizeIterator<Item = Self::NodeId> + 'g {
        let mut visited = HashSet::new();
        LazyExactSizedIterator::new(self.edges_from(node).filter_map(move |eid| {
            let ends = self.edge_ends(&eid);
            let nid = if Self::Directedness::IS_DIRECTED {
                ends.into_second()
            } else {
                ends.into_other_value(node).into_inner()
            };
            visited.insert(nid.clone()).then_some(nid)
        }))
    }

    // =========================================================================
    // Edges
    // =========================================================================

    /// Gets a vector of all edges in the graph.
    fn edges(&self) -> impl ExactSizeIterator<Item = Self::EdgeId> + '_;

    /// Gets the data associated with an edge.
    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData;

    /// Gets the number of edges in the graph.
    fn num_edges(&self) -> usize {
        self.edges().len()
    }

    /// Gets the ends of an edge as a pair of node IDs.  For directed edges, the
    /// first node ID is the source and the second is the target.  For
    /// undirected edges, the IDs are in sorted order but otherwise arbitrary.
    fn edge_ends(
        &self,
        id: &Self::EdgeId,
    ) -> <Self::Directedness as Directedness>::EndPair<Self::NodeId>;

    /// Gets an iterator over the outgoing edges from a given node.
    fn edges_from<'g>(&'g self, from: &'g Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'g {
        self.edges().filter(|eid| {
            let (source, target) = self.edge_ends(eid).into();
            source == *from || !Self::Directedness::IS_DIRECTED && target == *from
        })
    }

    /// Gets an iterator over the incoming edges to a given node.
    fn edges_into<'g>(&'g self, into: &'g Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'g {
        self.edges().filter(|eid| {
            let (source, target) = self.edge_ends(eid).into();
            target == *into || !Self::Directedness::IS_DIRECTED && source == *into
        })
    }

    /// Gets an iterator over the edges from one node into another.
    fn edges_from_into<'g>(
        &'g self,
        from: &'g Self::NodeId,
        into: &'g Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'g {
        self.edges().filter(move |eid| {
            let (edge_source, edge_target) = self.edge_ends(eid).into();
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

    // =========================================================================
    // Searches
    // =========================================================================

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

    /// Finds shortest paths from a starting node to all other nodes using
    /// Dijkstra's algorithm.  Returns a map from each reachable node to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    fn shortest_paths<'g, C: Default + Ord + Copy + Add<Output = C>>(
        &'g self,
        start: &Self::NodeId,
        distance_fn: impl Fn(&Self::EdgeId) -> C,
    ) -> HashMap<Self::NodeId, (Path<'g, Self>, C)> {
        crate::dijkstra::shortest_paths(self, start, distance_fn)
    }

    // =========================================================================
    // Misc
    // =========================================================================

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
        D: renderer::DotRenderer<Self>,
        Self: Sized,
    {
        renderer::generate_dot_file(self, generator, output)
    }

    /// Generates a DOT representation of the graph as a String.
    #[cfg(feature = "dot")]
    fn to_dot_string<D>(&self, generator: &D) -> Result<String, renderer::DotError<D::Error>>
    where
        D: renderer::DotRenderer<Self>,
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

    /// Returns true if the graph implementation is known to be very slow for
    /// large graphs (e.g., due to using a dense adjacency matrix).  This is mainly
    /// intended to be used to skip certain tests that would take an unreasonable
    /// amount of time to complete.
    #[doc(hidden)]
    fn is_very_slow(&self) -> bool {
        false
    }

    /// Finds the strongly connected component containing the given node.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_component(&self, start: &Self::NodeId) -> Vec<Self::NodeId>
    where
        Self: Graph<Directedness = Directed>,
    {
        pathfinding::prelude::strongly_connected_component(start, |nid| {
            self.successors(nid).collect::<Vec<_>>()
        })
    }

    /// Partitions the graph into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components(&self) -> Vec<Vec<Self::NodeId>>
    where
        Self: Graph<Directedness = Directed>,
    {
        pathfinding::prelude::strongly_connected_components(
            &self.nodes().collect::<Vec<_>>(),
            |nid| self.successors(nid).collect::<Vec<_>>(),
        )
    }

    /// Partitions nodes reachable from a starting point into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components_from(&self, start: &Self::NodeId) -> Vec<Vec<Self::NodeId>>
    where
        Self: Graph<Directedness = Directed>,
    {
        pathfinding::prelude::strongly_connected_components_from(start, |nid| {
            self.successors(nid).collect::<Vec<_>>()
        })
    }

    #[cfg(feature = "pathfinding")]
    fn connected_components(&self) -> Vec<HashSet<Self::NodeId>>
    where
        Self: Graph<Directedness = Undirected>,
    {
        pathfinding::prelude::connected_components(&self.nodes().collect::<Vec<_>>(), |nid| {
            self.successors(nid).collect::<Vec<_>>()
        })
    }
}

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
        for nid in self.nodes().collect::<Vec<_>>() {
            self.remove_node(&nid);
        }
    }

    /// Adds a node with the given data to the graph, returning its `NodeId`.
    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId;

    /// Removes a node from the graph, returning its data.  Any edges
    /// connected to the node are also be removed.
    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData;

    /// Removes all nodes for which the given predicate returns false.  Any edges
    /// connected to removed nodes are also removed.
    fn retain_nodes(&mut self, retain: impl FnMut(&Self::NodeId) -> bool) {
        let mut retain = retain;
        for nid in self.nodes().collect::<Vec<_>>() {
            if !retain(&nid) {
                self.remove_node(&nid);
            }
        }
    }

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
    #[allow(clippy::type_complexity)]
    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<(Self::EdgeId, Self::EdgeData)>);

    /// Remove an edge between two nodes, returning its data.
    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData;

    /// Removes all edges for which the given predicate returns false.
    fn retain_edges(&mut self, retain: impl FnMut(&Self::EdgeId) -> bool) {
        let mut retain = retain;
        for eid in self.edges().collect::<Vec<_>>() {
            if !retain(&eid) {
                self.remove_edge(&eid);
            }
        }
    }

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
