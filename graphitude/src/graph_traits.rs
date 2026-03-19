use std::{collections::HashSet, fmt::Debug, hash::Hash};

#[cfg(feature = "pathfinding")]
use crate::{end_pair::EndPair, prelude::*, util::other_value};

/// A trait representing a node identifier in a graph.
///
/// This trait has no methods but serves as a marker for types that can be used
/// as node identifiers.  This has the unfortunatel side-effect of preventing
/// the use of primitive types (e.g., `usize`, `u32`, etc.) as node identifiers,
/// since they do not implement this trait.  To work around this, you can define
/// a newtype wrapper around the primitive type and implement `NodeIdTrait` for the
/// newtype.
pub trait NodeIdTrait: Eq + Hash + Clone + Debug + Ord + Send + Sync {}

/// A trait representing an edge identifier in a graph.
///
/// Implementors mu implement either `left` and `right`, or `ends`.
pub trait EdgeIdTrait: Eq + Hash + Clone + Debug + Ord + Send + Sync {
    type NodeId: NodeIdTrait;
    type Directedness: DirectednessTrait;

    fn into_ends(self) -> EndPair<Self::NodeId, Self::Directedness>;

    /// Gets the directedness of the edge, which will match the directedness of
    /// the graph it belongs to.
    fn directedness(&self) -> Self::Directedness;

    /// Gets one end of the edge.  For directed edges, this is the source node.
    /// For undirected edges, this is one of the two nodes, but it is not
    /// specified which one.
    fn left(&self) -> Self::NodeId {
        self.ends().0
    }

    /// Gets the other end of the edge.  For directed edges, this is the target
    /// node.  For undirected edges, this is the other of the two nodes, but it
    /// is not specified which one is which.
    fn right(&self) -> Self::NodeId {
        self.ends().1
    }

    /// Gets both ends of the edge.  Returns `(self.left(), self.right())`.
    /// Implementors must implement either this method or `left` and `right`.
    fn ends(&self) -> (Self::NodeId, Self::NodeId) {
        (self.left(), self.right())
    }

    /// Gets the other end of the edge given one end.  If the edge is directed,
    /// the direction is ignored and the other end is returned.  If the edge is
    /// undirected, the other end is returned regardless of which end is passed
    /// in.  If the edge is a self-loop and the given end is the same as both
    /// ends of the edge, then the same node ID is returned.  Panics if the given
    /// node ID is not one of the ends of the edge.
    fn other_end(&self, node: &Self::NodeId) -> Self::NodeId {
        let (n1, n2) = self.ends();
        other_value(n1, n2, node).into_inner()
    }

    /// Tests if the edge has the given node as one of its ends.
    fn has_end(&self, node: &Self::NodeId) -> bool {
        let (n1, n2) = self.ends();
        *node == n1 || *node == n2
    }

    /// Tests if the edge has the given nodes as its ends, regardless of order.
    fn has_ends(&self, node1: &Self::NodeId, node2: &Self::NodeId) -> bool {
        let (n1, n2) = self.ends();
        (*node1 == n1 && *node2 == n2) || (*node1 == n2 && *node2 == n1)
    }
}

/// Return type of [`Graph::add_edge`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddEdgeResult<I, D> {
    /// A new edge was added with the given ID.
    Added(I),
    /// An existing edge was updated with new data. The edge ID and the old data are returned.
    Updated(I, D),
}

impl<I, D> AddEdgeResult<I, D> {
    /// Returns the new edge ID if the result was `Added`, or the old edge ID if the result was `Updated`.
    pub fn edge_id(self) -> I {
        match self {
            AddEdgeResult::Added(id) => id,
            AddEdgeResult::Updated(id, _) => id,
        }
    }

    pub fn map_edge_id<J>(self, f: impl FnOnce(I) -> J) -> AddEdgeResult<J, D> {
        match self {
            AddEdgeResult::Added(id) => AddEdgeResult::Added(f(id)),
            AddEdgeResult::Updated(id, data) => AddEdgeResult::Updated(f(id), data),
        }
    }
}

/// A trait representing a directed or undirected graph data structure.  Methods
/// that return iterators over nodes or edges return them in an unspecified
/// order unless otherwise noted.
///
/// For the sake of catching errors more reliably, it is recommended that
/// implementations of this trait implement the following methods that have
/// default implementions:
///
/// - [`Self::maybe_check_valid_node_id`]
/// - [`Self::maybe_check_valid_edge_id`]
///
/// For the sake of performance, it is recommended that implementations of this
/// trait implement the following methods that have default implementions with a
/// more efficient implementation that calls [`Self::maybe_check_valid_node_id`] or
/// [`Self::maybe_check_valid_edge_id`], either directly or indirectly at the start of the
/// method:
///
/// - [`Self::check_valid_node_id`]
/// - [`Self::check_valid_edge_id`]
/// - [`Self::edges_from`]
/// - [`Self::edges_into`]
/// - [`Self::num_edges_from`]
/// - [`Self::num_edges_into`]
/// - [`Self::has_edge_from`]
/// - [`Self::has_edge_into`]
pub trait GraphImpl {
    /// The directedness of the graph.
    type Directedness: DirectednessTrait;

    /// The edge multiplicity of the graph.
    type EdgeMultiplicity: EdgeMultiplicityTrait;

    /// The data stored in each node of the graph.
    type NodeData;

    /// The data stored in each edge of the graph.
    type EdgeData;

    /// The type of the node identifiers used by the graph.
    type NodeId: NodeIdTrait;

    /// The type of the edge identifiers used by the graph.
    type EdgeId: EdgeIdTrait<NodeId = Self::NodeId, Directedness = Self::Directedness>;

    /// The directedness of the graph.
    fn directedness(&self) -> Self::Directedness;

    /// The edge multiplicity of the graph.
    fn edge_multiplicity(&self) -> Self::EdgeMultiplicity;

    /// Checks if the graph is empty (has no nodes or edges).
    fn is_empty(&self) -> bool {
        if self.node_ids().next().is_none() {
            debug_assert!(self.edge_ids().next().is_none());
            true
        } else {
            false
        }
    }

    // Nodes

    /// Gets a vector of all NodeIds in the graph.
    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId>;

    /// Gets the data associated with a node.
    fn node_data<'a>(&'a self, id: &Self::NodeId) -> &'a Self::NodeData;

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
            let nid = if self.directedness().is_directed() {
                eid.left()
            } else {
                let (source, target) = eid.ends();
                if source == *node { target } else { source }
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
            let nid = if self.directedness().is_directed() {
                eid.right()
            } else {
                let (source, target) = eid.ends();
                if source == *node { target } else { source }
            };
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    // Edges

    /// Gets the data associated with an edge.
    fn edge_data<'a>(&'a self, id: &Self::EdgeId) -> &'a Self::EdgeData;

    /// Gets a vector of all edges in the graph.
    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_;

    /// Checks if a EdgeId is valid in the graph to the extent that can be
    /// determined without iterating over all edges, returning a reason if it is
    /// not.  This may return false positives for some graph implementations.
    fn check_valid_edge_id(&self, id: &Self::EdgeId) -> Result<(), &'static str> {
        if self.edge_ids().any(|eid| &eid == id) {
            Ok(())
        } else {
            Err("EdgeId not found in graph")
        }
    }

    /// Checks if a EdgeId is valid in the graph to the extent that can be
    /// determined without iterating over all edges, returning a reason if it is
    /// not.  May return false positives for some graph implementations.
    ///
    /// By default, this method always returns Ok(()).
    fn maybe_check_valid_edge_id(&self, _id: &Self::EdgeId) -> Result<(), &'static str> {
        Ok(())
    }

    /// Panics if the given EdgeId is not valid in the graph, according to
    /// [`Self::maybe_check_valid_edge_id`].
    ///
    /// It is recommended to call this method from implementations of other methods
    /// that take EdgeIds as parameters, to ensure that invalid EdgeIds are
    /// caught early.
    fn assert_valid_edge_id(&self, id: &Self::EdgeId) {
        if let Err(reason) = self.maybe_check_valid_edge_id(id) {
            panic!("Invalid EdgeId: {:?}: {}", id, reason);
        }
    }

    /// Panics if the given EdgeId is not valid in the graph, according to
    /// [`Self::maybe_check_valid_edge_id`], but only in debug builds.
    fn debug_assert_valid_edge_id(&self, id: &Self::EdgeId) {
        #[cfg(debug_assertions)]
        if let Err(reason) = self.maybe_check_valid_edge_id(id) {
            panic!("Invalid EdgeId: {:?}: {}", id, reason);
        }
    }

    /// Gets an iterator over the outgoing edges from a given node.
    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.edge_ids().filter(|eid| {
            let (source, target) = eid.ends();
            source == *from || !self.directedness().is_directed() && target == *from
        })
    }

    /// Gets an iterator over the incoming edges to a given node.
    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.edge_ids().filter(|eid| {
            let (source, target) = eid.ends();
            target == *into || !self.directedness().is_directed() && source == *into
        })
    }

    /// Gets an iterator over the edges from one node into another.
    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.edge_ids().filter(move |eid| {
            let (edge_source, edge_target) = eid.ends();
            (edge_source == *from && edge_target == *into)
                || (!self.directedness().is_directed()
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
pub trait GraphDirected: GraphImpl {
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

impl<G> GraphDirected for G where G: GraphImpl<Directedness = Directed> {}

/// A trait which is automatically implemented for undirected graphs, providing
/// methods specific to undirected graphs.
pub trait GraphUndirected: GraphImpl {
    #[cfg(feature = "pathfinding")]
    fn connected_components(&self) -> Vec<HashSet<Self::NodeId>> {
        pathfinding::prelude::connected_components(&self.node_ids().collect::<Vec<_>>(), |nid| {
            self.successors(nid).collect::<Vec<_>>()
        })
    }
}

impl<G> GraphUndirected for G where G: GraphImpl<Directedness = Undirected> {}

/// A trait for graphs that support mutation operations.
///
/// This trait extends [`GraphImpl`] with methods for adding and removing nodes and edges.
/// All graph implementations that support modification should implement this trait.
pub trait GraphImplMut: GraphImpl {
    /// Creates a new empty graph.
    fn new(directedness: Self::Directedness, edge_multiplicity: Self::EdgeMultiplicity) -> Self
    where
        Self: Sized;

    /// Gets a mutable reference to the data associated with a node.
    fn node_data_mut<'a>(&'a mut self, id: &'a Self::NodeId) -> &'a mut Self::NodeData;

    /// Gets a mutable reference to the data associated with an edge.
    fn edge_data_mut<'a>(&'a mut self, id: &'a Self::EdgeId) -> &'a mut Self::EdgeData;

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
        Self: GraphImpl<EdgeMultiplicity = MultipleEdges>,
    {
        match self.add_edge(from, into, data) {
            AddEdgeResult::Added(eid) => eid,
            AddEdgeResult::Updated(_, _) => {
                unreachable!("Edge already exists between {:?} and {:?}", from, into)
            }
        }
    }

    /// Add an edge if possible, or replaces the data of an existing edge.  If
    /// no edge exists, or the graph supports parallel edges, the new edge is
    /// added and and its edge ID is returned.  Otherwise, the data of the
    /// existing edge is replaced with the new data, and the old data is
    /// returned.
    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> AddEdgeResult<Self::EdgeId, Self::EdgeData>;

    /// Remove an edge between two nodes, returning its data.
    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData;

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
    /// NodeIds and EdgeIds.  Calls a closure for each node ID mapping
    /// (old_id, new_id) and edge ID mapping (old_id, new_id) as they are created.
    fn compact_with(
        &mut self,
        mut node_id_callback: impl FnMut(&'_ Self::NodeId, &'_ Self::NodeId),
        mut edge_id_callback: impl FnMut(&'_ Self::EdgeId, &'_ Self::EdgeId),
    ) {
        let _ = &mut node_id_callback;
        let _ = &mut edge_id_callback;
    }

    /// Shrinks internal storage used by the graph to fit its current size.
    /// Does nothing by default.
    fn shrink_to_fit(&mut self) {}
}
