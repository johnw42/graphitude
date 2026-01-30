use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

#[cfg(feature = "pathfinding")]
use {pathfinding::num_traits::Zero, std::iter::once};

use crate::{
    LinkedGraph,
    directedness::{Directed, Directedness, Undirected},
};
use crate::{
    path::Path,
    search::{BfsIterator, DfsIterator},
};

/// A trait representing a node identifier in a graph.
pub trait NodeId: Eq + Hash + Clone + Debug {}

/// A trait representing an edge identifier in a graph.
pub trait EdgeId<N>: Eq + Hash + Clone + Debug
where
    N: Eq + Debug,
{
    /// Gets the source node of the edge.
    fn source(&self) -> N;

    /// Gets the target node of the edge.
    fn target(&self) -> N;

    /// Gets both ends of the edge as a tuple (source, target).
    fn ends(&self) -> (N, N) {
        (self.source(), self.target())
    }

    /// Given one end of the edge, returns the other end.  Returns `None` if the
    /// edge is a self-loop.  Panics if the given node is not an endpoint of the
    /// edge.
    fn other_end(&self, node_id: N) -> Option<N> {
        let (source, target) = self.ends();
        if source == node_id {
            Some(target)
        } else if target == node_id {
            Some(source)
        } else {
            assert_eq!(source, target); // self-loop
            None
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
/// - [`Self::is_maybe_valid_node_id`]
/// - [`Self::is_maybe_valid_edge_id`]
//
/// For the sake of performance, it is recommended that implementations of this
/// trait implement the following methods that have default implementions with a
/// more efficient implementation that calls [`Self::check_node_id`] or
/// [`Self::check_edge_id`], either directly or indirectly at the start of the
/// method:
///
/// - [`Self::is_valid_node_id`]
/// - [`Self::is_valid_edge_id`]
/// - [`Self::edges_from`]
/// - [`Self::edges_into`]
/// - [`Self::num_edges_from`]
/// - [`Self::num_edges_into`]
/// - [`Self::has_edge_from`]
/// - [`Self::has_edge_into`]
pub trait Graph: Sized {
    type NodeData;
    type NodeId: NodeId;
    type EdgeData;
    type EdgeId: EdgeId<Self::NodeId>;
    type Directedness: Directedness;

    /// Returns true if the graph is directed.
    fn is_directed(&self) -> bool {
        Self::Directedness::is_directed()
    }

    /// Creates a new graph with the same structure as this graph, but with
    /// all node and edge data replaced with `()`.  This is intended for debugging
    /// purposes, to allow inspection of the graph structure without when the
    /// data does not implement `Debug`.
    fn without_data(&self) -> impl Graph<NodeData = (), EdgeData = ()> + Debug {
        let mut g = LinkedGraph::new();
        g.copy_from_with(self, |_| (), |_| ());
        g
    }

    /// Creates a new path starting from the given starting node.  This is a
    /// convenience method to avoid having to import the `Path` type separately
    /// and specify its type argument explicity.
    fn new_path(&self, start: Self::NodeId) -> Path<Self::NodeId, Self::EdgeId> {
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

    /// Panics if the given NodeId is not valid in the graph, accordinging to
    /// [`Self::is_valid_node_id`].
    ///
    /// It is recommended to call this method from implementations of other methods
    /// that take NodeIds as parameters, to ensure that invalid NodeIds are
    /// caught early.
    fn assert_valid_node_id(&self, id: &Self::NodeId) {
        if let Err(reason) = self.maybe_check_valid_node_id(id) {
            panic!("Invalid NodeId: {:?}: {}", id, reason);
        }
    }

    /// Panics if the given NodeId is not valid in the graph, accordinging to
    /// [`Self::is_maybe_valid_node_id`], but only in debug builds.
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

    /// Panics if the given EdgeId is not valid in the graph, accordinging to
    /// [`Self::is_valid_edge_id`].
    ///
    /// It is recommended to call this method from implementations of other methods
    /// that take EdgeIds as parameters, to ensure that invalid EdgeIds are
    /// caught early.
    fn assert_valid_edge_id(&self, id: &Self::EdgeId) {
        if let Err(reason) = self.maybe_check_valid_edge_id(id) {
            panic!("Invalid EdgeId: {:?}: {}", id, reason);
        }
    }

    /// Panics if the given EdgeId is not valid in the graph, accordinging to
    /// [`Self::is_maybe_valid_edge_id`], but only in debug builds.
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
    fn shortest_paths<C: Zero + Ord + Copy>(
        &self,
        start: Self::NodeId,
        cost_fn: impl Fn(&Self::EdgeId) -> C,
    ) -> HashMap<Self::NodeId, (Vec<Self::NodeId>, C)> {
        let parents: HashMap<Self::NodeId, (Self::NodeId, C)> =
            pathfinding::prelude::dijkstra_all(&start, |nid| -> Vec<(Self::NodeId, C)> {
                let r: Vec<_> = self
                    .edges_from(nid.clone())
                    .map(|eid| {
                        let cost = cost_fn(&eid);
                        (eid.target(), cost)
                    })
                    .collect();
                r
            });
        once((start.clone(), (vec![start], C::zero())))
            .chain(
                parents
                    .iter()
                    .map(|(k, (_, cost)): (&Self::NodeId, &(_, C))| {
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
        Self::NodeData: Clone,
        Self::EdgeData: Clone,
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
            None::<fn(Self::NodeId, Self::NodeId)>,
            None::<fn(Self::EdgeId, Self::EdgeId)>,
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
        F1: FnMut(Self::NodeId, Self::NodeId),
        F2: FnMut(Self::EdgeId, Self::EdgeId),
    {
        let _ = &mut node_id_callback;
        let _ = &mut edge_id_callback;
    }

    /// Shrinks internal storage used by the graph to fit its current size.  May
    /// invalidate existing NodeIds and EdgeIds.  Does nothing by default.
    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_with::<fn(Self::NodeId, Self::NodeId), fn(Self::EdgeId, Self::EdgeId)>(
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
        F1: FnMut(Self::NodeId, Self::NodeId),
        F2: FnMut(Self::EdgeId, Self::EdgeId),
    {
        let _ = &mut node_id_callback;
        let _ = &mut edge_id_callback;
    }
}
