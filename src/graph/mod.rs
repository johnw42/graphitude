#[cfg(feature = "dot")]
pub mod dot_parser_impl;
#[cfg(feature = "dot")]
pub mod dot_types;

use std::ops::Add;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use crate::mapping_result::MappingResult;
use crate::{
    debug_graph_view::DebugGraphView,
    directedness::{Directed, Directedness, Undirected},
    pairs::Pair,
    path::Path,
    search::{BfsIterator, DfsIterator},
    util::{OtherValue, other_value},
};

/// A trait representing a node identifier in a graph.
///
///  This trait has no methods but serves as a marker for types that can be used
/// as node identifiers.  This has the unfortunately side-effect of preventing
/// the use of primitive types (e.g., `usize`, `u32`, etc.) as node identifiers,
/// since they do not implement this trait.  To work around this, you can define
/// a newtype wrapper around the primitive type and implement `NodeId` for the
/// newtype.
pub trait NodeId: Eq + Hash + Clone + Debug + Ord {}

/// Return type of [`EdgeId::other_end`].
pub enum OtherEnd<N: NodeId> {
    /// The source node of the edge for which a target was passed.
    Source(N),
    /// The target node of the edge for which a source was passed.
    Target(N),
    /// The edge is a self-loop; both ends are the same node.
    SelfLoop(N),
}

impl<N: NodeId> OtherEnd<N> {
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
pub trait EdgeId: Eq + Hash + Clone + Debug {
    type NodeId: NodeId;
    type Directedness: Directedness;

    /// Gets the source node of the edge.
    fn source(&self) -> Self::NodeId {
        self.ends().into_first()
    }

    /// Gets the target node of the edge.
    fn target(&self) -> Self::NodeId {
        self.ends().into_second()
    }

    /// Gets both ends of the edge as a tuple (source, target).
    fn ends(&self) -> <Self::Directedness as Directedness>::Pair<Self::NodeId> {
        (self.source(), self.target()).into()
    }

    /// Given one end of the edge, returns the other end.  Returns `None` if the
    /// edge is a self-loop.  Panics if the given node is not an endpoint of the
    /// edge.
    fn other_end(&self, node_id: Self::NodeId) -> OtherEnd<Self::NodeId> {
        match other_value(self.ends().into(), node_id) {
            OtherValue::First(node) => OtherEnd::Source(node),
            OtherValue::Second(node) => OtherEnd::Target(node),
            OtherValue::Both(node) => OtherEnd::SelfLoop(node),
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
//
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
pub trait Graph: Sized {
    type Directedness: Directedness;
    type NodeData;
    type NodeId: NodeId;
    type EdgeData;
    type EdgeId: EdgeId<NodeId = Self::NodeId, Directedness = Self::Directedness>;

    /// Returns true if the graph is directed.
    fn is_directed(&self) -> bool {
        Self::Directedness::is_directed()
    }

    /// Creates a new graph view in which node and edge data are hidden.
    fn with_debug(&self) -> impl Graph + Debug {
        self.with_debug_formatting(|_| (), |_| ())
    }

    /// Creates a new graph view with custom debug formatting for nodes and edges.
    fn with_debug_formatting<N, E>(
        &self,
        node_fmt: impl Fn(&Self::NodeData) -> N,
        edge_fmt: impl Fn(&Self::EdgeData) -> E,
    ) -> impl Graph + Debug
    where
        N: Debug,
        E: Debug,
    {
        DebugGraphView::<N, E, Self::Directedness>::new(self, node_fmt, edge_fmt)
    }

    #[cfg(feature = "dot")]
    fn generate_dot_file(&self) -> Vec<u8>
    where
        Self::NodeData: Debug,
        Self::EdgeData: Debug,
    {
        struct GraphWrapper<'a, G: Graph> {
            graph: &'a G,
            node_id_map: HashMap<G::NodeId, usize>,
        }

        impl<'a, G: Graph> GraphWrapper<'a, G> {
            fn new(graph: &'a G) -> Self {
                let node_id_map = graph
                    .node_ids()
                    .enumerate()
                    .map(|(i, nid)| (nid, i))
                    .collect();
                Self { graph, node_id_map }
            }
        }

        impl<'a, G> dot::Labeller<'a, G::NodeId, G::EdgeId> for GraphWrapper<'a, G>
        where
            G: Graph,
            G::NodeData: Debug,
            G::EdgeData: Debug,
        {
            fn graph_id(&'a self) -> dot::Id<'a> {
                dot::Id::new("G").unwrap()
            }

            fn node_id(&'a self, n: &G::NodeId) -> dot::Id<'a> {
                let idx = self.node_id_map.get(n).unwrap();
                dot::Id::new(format!("n{}", idx)).unwrap()
            }

            fn node_label(&'a self, n: &G::NodeId) -> dot::LabelText<'a> {
                let data = self.graph.node_data(n.clone());
                dot::LabelText::LabelStr(format!("{:?}", data).into())
            }

            fn edge_label(&'a self, e: &G::EdgeId) -> dot::LabelText<'a> {
                let data = self.graph.edge_data(e.clone());
                dot::LabelText::LabelStr(format!("{:?}", data).into())
            }
        }

        impl<'a, G> dot::GraphWalk<'a, G::NodeId, G::EdgeId> for GraphWrapper<'a, G>
        where
            G: Graph,
        {
            fn nodes(&'a self) -> dot::Nodes<'a, G::NodeId> {
                self.graph.node_ids().collect::<Vec<_>>().into()
            }

            fn edges(&'a self) -> dot::Edges<'a, G::EdgeId> {
                self.graph.edge_ids().collect::<Vec<_>>().into()
            }

            fn source(&'a self, edge: &G::EdgeId) -> G::NodeId {
                edge.source()
            }

            fn target(&'a self, edge: &G::EdgeId) -> G::NodeId {
                edge.target()
            }
        }

        let wrapper = GraphWrapper::new(self);
        let mut output = Vec::new();

        if self.is_directed() {
            dot::render(&wrapper, &mut output).unwrap();
        } else {
            dot::render(&wrapper, &mut output).unwrap();
        }

        output
    }

    /// Creates a new path starting from the given starting node.  This is a
    /// convenience method to avoid having to import the `Path` type separately
    /// and specify its type argument explicity.
    fn new_path(&self, start: Self::NodeId) -> Path<Self::EdgeId> {
        Path::new(start)
    }

    // Nodes

    /// Gets a vector of all NodeIds in the graph.
    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId>;

    /// Gets the data associated with a node.
    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData;

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

    /// Gets an iterator over the predacessors nodes of a given node, i.e.
    /// those nodes reachable by incoming edges.
    fn predacessors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> + '_ {
        let mut visited = HashSet::new();
        self.edges_into(node).filter_map(move |eid| {
            let nid = eid.source();
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    /// Gets an iterator over the successor nodes of a given node, i.e.
    /// those nodes reachable by outgoing edges.
    fn successors(&self, node: Self::NodeId) -> impl Iterator<Item = Self::NodeId> + '_ {
        let mut visited = HashSet::new();
        self.edges_from(node.clone()).filter_map(move |eid| {
            let nid = if self.is_directed() {
                eid.target()
            } else {
                let (source, target) = (eid.source(), eid.target());
                if source == node { target } else { source }
            };
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    // Edges

    /// Gets the data associated with an edge.
    fn edge_data(&self, from: Self::EdgeId) -> &Self::EdgeData;

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
    fn edges_from(&self, from: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edge_ids().filter(move |eid| {
            let (source, target) = (eid.source(), eid.target());
            source == from || !self.is_directed() && target == from
        })
    }

    /// Gets an iterator over the incoming edges to a given node.
    fn edges_into(&self, into: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edge_ids().filter(move |eid| {
            let (source, target) = (eid.source(), eid.target());
            target == into || !self.is_directed() && source == into
        })
    }

    /// Gets an iterator over the edges between two nodes.
    fn edges_between(
        &self,
        from: Self::NodeId,
        into: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.edges_from(from.clone()).filter(move |eid| {
            let (edge_source, edge_target) = (eid.source(), eid.target());
            edge_source == from && edge_target == into
                || !self.is_directed() && edge_source == into && edge_target == from
        })
    }

    /// Gets the number of edges from one node to another.
    fn num_edges_between(&self, from: Self::NodeId, into: Self::NodeId) -> usize {
        self.edges_between(from, into).into_iter().count()
    }

    /// Checks if there is at least one edge from one node to another.
    fn has_edge(&self, from: Self::NodeId, into: Self::NodeId) -> bool {
        self.edges_between(from, into).next().is_some()
    }

    /// Checks if there is at least one outgoing edge from the given node.
    fn has_edge_from(&self, from: Self::NodeId) -> bool {
        self.edges_from(from).next().is_some()
    }

    /// Checks if there is at least one incoming edge to the given node.
    fn has_edge_into(&self, into: Self::NodeId) -> bool {
        self.edges_into(into).next().is_some()
    }

    /// Checks if there is an edge between two nodes.
    fn has_edge_between(&self, from: Self::NodeId, into: Self::NodeId) -> bool {
        self.edges_between(from, into).next().is_some()
    }

    /// Gets the number of edges in the graph.
    fn num_edges(&self) -> usize {
        self.edge_ids().count()
    }

    /// Gets the number of incoming edges to a given node.
    fn num_edges_into(&self, into: Self::NodeId) -> usize {
        self.edges_into(into).into_iter().count()
    }

    /// Gets the number of outgoing edges from a given node.
    fn num_edges_from(&self, from: Self::NodeId) -> usize {
        self.edges_from(from).into_iter().count()
    }

    // Searches

    /// Performs a breadth-first search starting from the given node.
    fn bfs(&self, start: Self::NodeId) -> BfsIterator<'_, Self> {
        self.bfs_multi(vec![start])
    }

    /// Performs a breadth-first search starting from the given nodes.
    fn bfs_multi(&self, start: Vec<Self::NodeId>) -> BfsIterator<'_, Self> {
        BfsIterator::new(self, start)
    }

    /// Performs a depth-first search starting from the given node.
    fn dfs(&self, start: Self::NodeId) -> DfsIterator<'_, Self> {
        self.dfs_multi(vec![start])
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
        start: Self::NodeId,
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
            for edge_id in self.edges_from(current_node.clone()) {
                let neighbor = edge_id.other_end(current_node.clone()).into_inner();
                if unvisited.contains(&neighbor) {
                    let edge_distance = distance_fn(&edge_id);
                    let new_dist = current_dist + edge_distance;

                    let should_update = distances
                        .get(&neighbor)
                        .map_or(true, |&old_dist| new_dist < old_dist);

                    if should_update {
                        distances.insert(neighbor.clone(), new_dist);
                        predecessors.insert(neighbor, (edge_id, current_node.clone()));
                    }
                }
            }
        }

        // Build paths from predecessors
        let mut result: HashMap<<Self as Graph>::NodeId, (Path<Self::EdgeId>, C)> = HashMap::new();
        for (node, &dist) in &distances {
            if node == &start {
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
}

/// A trait which is automatically implemented for directed graphs, providing
/// methods specific to directed graphs.
pub trait GraphDirected: Graph {
    /// Finds the strongly connected component containing the given node.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_component(&self, start: &Self::NodeId) -> Vec<Self::NodeId> {
        pathfinding::prelude::strongly_connected_component(start, |nid| {
            self.successors(nid.clone())
        })
    }

    /// Partitions the graph into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components(&self) -> Vec<Vec<Self::NodeId>> {
        pathfinding::prelude::strongly_connected_components(
            &self.node_ids().collect::<Vec<_>>(),
            |nid| self.successors(nid.clone()),
        )
    }

    /// Partitions nodes reachable from a starting point into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components_from(&self, start: &Self::NodeId) -> Vec<Vec<Self::NodeId>> {
        pathfinding::prelude::strongly_connected_components_from(start, |nid| {
            self.successors(nid.clone())
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
            self.successors(nid.clone())
        })
    }
}

impl<G> GraphUndirected for G where G: Graph<Directedness = Undirected> {}

/// A trait for graphs that support mutation operations.
///
/// This trait extends [`Graph`] with methods for adding and removing nodes and edges.
/// All graph implementations that support modification should implement this trait.
pub trait GraphMut: Graph {
    /// Creates a new, empty graph.
    fn new() -> Self;

    /// Removes all nodes and edges from the graph.
    fn clear(&mut self) {
        for nid in self.node_ids().collect::<Vec<_>>() {
            self.remove_node(nid);
        }
    }

    /// Adds a node with the given data to the graph, returning its `NodeId`.
    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId;

    /// Removes a node from the graph, returning its data.  Any edges
    /// connected to the node are also be removed.
    fn remove_node(&mut self, id: Self::NodeId) -> Self::NodeData;

    /// Adds an edge with the given data between two nodes and returns the
    /// `EdgeId`.  If an edge already exists between the two nodes, and the
    /// graph does not support parallel edges, the old edge is replaced.
    /// Use [`Self::add_or_replace_edge`] to get the old edge data as well.
    fn add_edge(
        &mut self,
        from: Self::NodeId,
        to: Self::NodeId,
        data: Self::EdgeData,
    ) -> Self::EdgeId {
        self.add_or_replace_edge(from, to, data).0
    }

    /// Adds an edge with the given data between two nodes and returns the
    /// `EdgeId`.  If an edge already exists between the two nodes, and the
    /// graph does not support parallel edges, the old edge is replaced and its
    /// data is returned as well.
    fn add_or_replace_edge(
        &mut self,
        from: Self::NodeId,
        to: Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>);

    /// Remove an edge between two nodes, returning its data.
    fn remove_edge(&mut self, from: Self::EdgeId) -> Self::EdgeData;

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
        S: Graph,
        F: FnMut(&S::NodeData) -> Self::NodeData,
        G: FnMut(&S::EdgeData) -> Self::EdgeData,
    {
        let mut node_map = HashMap::new();
        for nid in source.node_ids() {
            let vdata = map_node(source.node_data(nid.clone()));
            let new_nid = self.add_node(vdata);
            node_map.insert(nid, new_nid);
        }
        for eid in source.edge_ids() {
            let (from, to) = (eid.source(), eid.target());
            let edata = map_edge(source.edge_data(eid));
            let new_from = node_map.get(&from).expect("missing node");
            let new_to = node_map.get(&to).expect("missing node");
            self.add_edge(new_from.clone(), new_to.clone(), edata);
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
                    .edges_between(from2.clone(), to2.clone())
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
        self.compact_with(
            None::<fn(MappingResult<Self::NodeId>)>,
            None::<fn(MappingResult<Self::EdgeId>)>,
        );
    }

    /// Compacts internal storage used by the graph to minimize memory usage
    /// without reallocation.  Does nothing by default.  May invalidate existing
    /// NodeIds and EdgeIds.  Calls a closure for each node ID mapping
    /// (old_id, new_id) and edge ID mapping (old_id, new_id) as they are created.
    fn compact_with<F1, F2>(
        &mut self,
        mut node_id_callback: Option<F1>,
        mut edge_id_callback: Option<F2>,
    ) where
        F1: FnMut(MappingResult<Self::NodeId>),
        F2: FnMut(MappingResult<Self::EdgeId>),
    {
        let _ = &mut node_id_callback;
        let _ = &mut edge_id_callback;
    }

    /// Shrinks internal storage used by the graph to fit its current size.  May
    /// invalidate existing NodeIds and EdgeIds.  Does nothing by default.
    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_with::<fn(MappingResult<Self::NodeId>), fn(MappingResult<Self::EdgeId>)>(
            None, None,
        );
    }

    /// Shrinks internal storage used by the graph to fit its current size.  May
    /// invalidate existing NodeIds and EdgeIds.  Does nothing by default.
    /// Calls a closure for each node ID mapping (old_id, new_id)
    /// and edge ID mapping (old_id, new_id) as they are created.
    fn shrink_to_fit_with<F1, F2>(
        &mut self,
        mut node_id_callback: Option<F1>,
        mut edge_id_callback: Option<F2>,
    ) where
        F1: FnMut(MappingResult<Self::NodeId>),
        F2: FnMut(MappingResult<Self::EdgeId>),
    {
        let _ = &mut node_id_callback;
        let _ = &mut edge_id_callback;
    }

    #[cfg(feature = "dot")]
    fn new_from_dot<B>(
        data: &str,
        builder: &mut B,
    ) -> Result<Self, dot_parser_impl::DotParseError<B>>
    where
        B: dot_parser_impl::DotGraphBuilder<NodeData = Self::NodeData, EdgeData = Self::EdgeData>,
    {
        dot_parser_impl::parse_dot_into_graph(data, builder)
    }
}
