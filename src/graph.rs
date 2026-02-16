use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    ops::Add,
};

#[cfg(feature = "dot")]
use std::io;

#[cfg(feature = "dot")]
use crate::dot::{parser, renderer};
use crate::{
    debug_graph_view::DebugGraphView,
    edge_ends::EdgeEnds,
    path::Path,
    prelude::*,
    search::{BfsIterator, DfsIterator},
}; // Import the trait that defines EdgeEnds::new

/// A trait representing a node identifier in a graph.
///
/// This trait has no methods but serves as a marker for types that can be used
/// as node identifiers.  This has the unfortunately side-effect of preventing
/// the use of primitive types (e.g., `usize`, `u32`, etc.) as node identifiers,
/// since they do not implement this trait.  To work around this, you can define
/// a newtype wrapper around the primitive type and implement `NodeIdTrait` for the
/// newtype.
pub trait NodeIdTrait: Eq + Hash + Clone + Debug + Ord + Send + Sync {}

/// Return type of [`EdgeId::other_end`].
pub enum OtherEnd<N: NodeIdTrait> {
    /// The source node of the edge for which a target was passed.
    Source(N),
    /// The target node of the edge for which a source was passed.
    Target(N),
    /// The edge is a self-loop; both ends are the same node.
    SelfLoop(N),
}

impl<N: NodeIdTrait> OtherEnd<N> {
    /// Consumes the `OtherEnd`, returning the inner node ID.
    pub fn into_inner(self) -> N {
        match self {
            OtherEnd::Source(n) => n,
            OtherEnd::Target(n) => n,
            OtherEnd::SelfLoop(n) => n,
        }
    }
}

/// A trait representing an edge identifier in a graph.  When Directedness is
/// `Directed`, the source and target are distinct; when Directedness is
/// `Undirected`, the source and target are always be ordered such that
/// `source <= target`.
///
/// Implementors must ensure the following conditions:
/// - Either `ends` is implemented, or both `source` and `target` are implemented.
/// - If `Directedness` is `Undirected`, then the source must always be less than or
///   equal to the target, according to the `Ord` implementation of the `NodeId`
///   type.
pub trait EdgeIdTrait: Eq + Hash + Clone + Debug + Send + Sync {
    type NodeId: NodeIdTrait;
    type Directedness: DirectednessTrait;

    fn directedness(&self) -> Self::Directedness;

    /// Gets the source node of the edge.
    fn source(&self) -> Self::NodeId {
        self.ends().into_source()
    }

    /// Gets the target node of the edge.
    fn target(&self) -> Self::NodeId {
        self.ends().into_target()
    }

    /// Gets both ends of the edge.
    fn ends(&self) -> EdgeEnds<Self::NodeId, Self::Directedness> {
        self.directedness().make_pair(self.source(), self.target())
    }
}

/// Return type of [`Graph::add_or_update_edge`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddEdgeResult<I, D> {
    /// A new edge was added with the given ID.
    Added(I),
    /// An existing edge was updated with new data, and the old data is returned.
    Updated(D),
}

impl<I, D> AddEdgeResult<I, D> {
    /// Returns the new edge ID if the result was `Added`, otherwise panics.
    pub fn unwrap(self) -> I {
        match self {
            AddEdgeResult::Added(id) => id,
            AddEdgeResult::Updated(_) => panic!("Called unwrap on an Updated edge result"),
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
pub trait Graph {
    type Directedness: DirectednessTrait;
    type EdgeMultiplicity: EdgeMultiplicityTrait;
    type NodeData;
    type NodeId: NodeIdTrait;
    type EdgeData;
    type EdgeId: EdgeIdTrait<NodeId = Self::NodeId, Directedness = Self::Directedness>;

    fn directedness(&self) -> Self::Directedness;

    fn edge_multiplicity(&self) -> Self::EdgeMultiplicity;

    /// Returns true if the graph is directed.
    fn is_directed(&self) -> bool {
        self.directedness().is_directed()
    }

    /// Returns true if the graph allows parallel edges between the same pair of nodes.
    fn allows_parallel_edges(&self) -> bool {
        self.edge_multiplicity().allows_parallel_edges()
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
        node_fmt: impl Fn(&Self::NodeData) -> N,
        edge_fmt: impl Fn(&Self::EdgeData) -> E,
    ) -> impl Graph + Debug
    where
        N: Debug,
        E: Debug,
    {
        DebugGraphView::<N, E, _, _>::new(self, node_fmt, edge_fmt)
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
    fn new_path(&self, start: &Self::NodeId) -> Path<Self::EdgeId> {
        Path::new(start.clone())
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

    /// Checks if a NodeId is valid in the graph, returning a reason if it is
    /// not. This operation is potentially costly.
    fn check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        if self.node_ids().any(|nid| &nid == id) {
            Ok(())
        } else {
            Err("NodeId not found in graph")
        }
    }

    /// Checks if a NodeId is valid in the graph to the extent that can be
    /// determined without iterating over all nodes, returning a reason if it is
    /// not.  This may return false positives for some graph implementations.
    ///
    /// By default, this method always returns Ok(()).
    fn maybe_check_valid_node_id(&self, _id: &Self::NodeId) -> Result<(), &'static str> {
        Ok(())
    }

    /// Panics if the given NodeId is not valid in the graph, according to
    /// [`Self::maybe_check_valid_node_id`].
    ///
    /// It is recommended to call this method from implementations of other methods
    /// that take NodeIds as parameters, to ensure that invalid NodeIds are
    /// caught early.
    fn assert_valid_node_id(&self, id: &Self::NodeId) {
        if let Err(reason) = self.maybe_check_valid_node_id(id) {
            panic!("Invalid NodeId: {:?}: {}", id, reason);
        }
    }

    /// Panics if the given NodeId is not valid in the graph, according to
    /// [`Self::maybe_check_valid_node_id`], but only in debug builds.
    fn debug_assert_valid_node_id(&self, id: &Self::NodeId) {
        #[cfg(debug_assertions)]
        if let Err(reason) = self.maybe_check_valid_node_id(id) {
            panic!("Invalid NodeId: {:?}: {}", id, reason);
        }
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
                eid.source()
            } else {
                let (source, target) = eid.ends().into_values();
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
                eid.target()
            } else {
                let (source, target) = eid.ends().into_values();
                if source == *node { target } else { source }
            };
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    // Edges

    /// Gets the data associated with an edge.
    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData;

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
            let (source, target) = (eid.source(), eid.target());
            source == *from || !self.directedness().is_directed() && target == *from
        })
    }

    /// Gets an iterator over the incoming edges to a given node.
    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.edge_ids().filter(|eid| {
            let (source, target) = (eid.source(), eid.target());
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
            let (edge_source, edge_target) = (eid.source(), eid.target());
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

    // Pathfinding

    /// Finds shortest paths from a starting node to all other nodes using
    /// Dijkstra's algorithm.  Returns a map from each reachable node to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    fn shortest_paths<C: Default + Ord + Copy + Add<Output = C>>(
        &self,
        start: &Self::NodeId,
        distance_fn: impl Fn(&Self::EdgeId) -> C,
    ) -> HashMap<Self::NodeId, (Path<Self::EdgeId>, C)> {
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
                let ends = edge_id.ends();
                let neighbor = ends.other_value(&current_node).into_inner();
                if unvisited.contains(neighbor) {
                    let edge_distance = distance_fn(&edge_id);
                    let new_dist = current_dist + edge_distance;

                    let should_update = distances
                        .get(neighbor)
                        .is_none_or(|&old_dist| new_dist < old_dist);

                    if should_update {
                        distances.insert(neighbor.clone(), new_dist);
                        predecessors.insert(neighbor.clone(), (edge_id, current_node.clone()));
                    }
                }
            }
        }

        // Build paths from predecessors
        let mut result: HashMap<<Self as Graph>::NodeId, (Path<Self::EdgeId>, C)> = HashMap::new();
        for (node, &dist) in &distances {
            if node == start {
                result.insert(start.clone(), (Path::new(start.clone()), C::default()));
            } else {
                let mut current = node.clone();

                let mut path_edges = Vec::new();
                while let Some(pred) = predecessors.get(&current) {
                    path_edges.push(pred.0.clone());
                    current = pred.1.clone();
                }

                let mut path = Path::new(start.clone());
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
    /// Creates a new empty graph.
    fn new(directedness: Self::Directedness, edge_multiplicity: Self::EdgeMultiplicity) -> Self
    where
        Self: Sized;

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
        to: &Self::NodeId,
        data: Self::EdgeData,
    ) -> Self::EdgeId
    where
        Self: Graph<EdgeMultiplicity = MultipleEdges>,
    {
        match self.add_edge(from, to, data) {
            AddEdgeResult::Added(eid) => eid,
            AddEdgeResult::Updated(_) => {
                unreachable!("Edge already exists between {:?} and {:?}", from, to)
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
        to: &Self::NodeId,
        data: Self::EdgeData,
    ) -> AddEdgeResult<Self::EdgeId, Self::EdgeData>;

    /// Remove an edge between two nodes, returning its data.
    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData;

    /// Copies all nodes and edges from another graph into this graph.
    fn copy_from<S>(&mut self, source: &S) -> HashMap<S::NodeId, Self::NodeId>
    where
        S: Graph<NodeData = Self::NodeData, EdgeData = Self::EdgeData>,
        Self::NodeData: Clone,
        Self::EdgeData: Clone,
    {
        self.copy_from_with(source, &mut Clone::clone, &mut Clone::clone)
    }

    /// Copies all nodes and edges from another graph into this graph,
    /// transforming the node and edge data using the provided mapping
    /// functions.
    fn copy_from_with<S, F, G>(
        &mut self,
        source: &S,
        mut map_node: F,
        mut map_edge: G,
    ) -> HashMap<S::NodeId, Self::NodeId>
    where
        S: Graph + ?Sized,
        F: FnMut(&S::NodeData) -> Self::NodeData,
        G: FnMut(&S::EdgeData) -> Self::EdgeData,
    {
        let mut node_map = HashMap::new();
        for nid in source.node_ids() {
            let vdata = map_node(source.node_data(&nid));
            let new_nid = self.add_node(vdata);
            node_map.insert(nid, new_nid);
        }
        for eid in source.edge_ids() {
            let (from, to) = (eid.source(), eid.target());
            let edata = map_edge(source.edge_data(&eid));
            let new_from = node_map.get(&from).expect("missing node");
            let new_to = node_map.get(&to).expect("missing node");
            self.add_edge(new_from, new_to, edata);
        }
        node_map
    }

    /// Creates a mapping from edges in this graph to edges in another graph,
    /// based on a provided node mapping from [`Self::copy_from`] or
    /// [`Self::copy_from_with`].
    fn make_edge_map<S>(
        &self,
        source: &S,
        node_map: &HashMap<S::NodeId, Self::NodeId>,
    ) -> HashMap<S::EdgeId, Self::EdgeId>
    where
        S: Graph,
    {
        let mut edge_map = HashMap::new();
        for eid in source.edge_ids() {
            let (from1, to1) = (eid.source(), eid.target());
            if let Some(from2) = node_map.get(&from1)
                && let Some(to2) = node_map.get(&to1)
            {
                let eid2 = self
                    .edges_from_into(from2, to2)
                    .find(|_| true)
                    .expect("missing edge");
                edge_map.insert(eid, eid2);
            }
        }
        edge_map
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
    /// NodeIds and EdgeIds.
    fn compact(&mut self) {
        self.compact_with(|_, _| {}, |_, _| {});
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
