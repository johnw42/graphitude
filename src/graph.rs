use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

#[cfg(feature = "pathfinding")]
use {pathfinding::num_traits::Zero, std::iter::once};

use crate::directedness::{Directed, Directedness, Undirected};
use crate::search::{BfsIterator, DfsIterator};

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
    type EdgeData;
    type EdgeId<'g>: Eq + Hash + Clone + Debug where Self: 'g;
    type NodeData;
    type NodeId<'g>: Eq + Hash + Clone + Debug where Self: 'g;
    type Directedness: Directedness;

    fn is_directed(&self) -> bool {
        Self::Directedness::is_directed()
    }

    // Nodes

    /// Gets a vector of all NodeIds in the graph.
    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId<'_>>;

    /// Gets the data associated with a node.
    fn node_data<'a>(&'a self, id: Self::NodeId<'a>) -> &'a Self::NodeData;

    /// Gets the number of nodes in the graph.
    fn num_nodes(&self) -> usize {
        self.node_ids().count()
    }

    /// Checks if a NodeId is valid in the graph. This operation is
    /// potentially costly.
    fn is_valid_node_id<'a>(&'a self, id: &Self::NodeId<'a>) -> bool {
        self.node_ids().any(|nid| &nid == id)
    }

    /// Checks if a NodeId is valid in the graph to the extent that can be
    /// determined without iterating over all nodes.  This may return false
    /// positives for some graph implementations.
    /// 
    /// By default, this method always returns true.
    fn is_maybe_valid_node_id<'a>(&'a self, _id: &Self::NodeId<'a>) -> bool {
        true
    }

    /// Panics if the given NodeId is not valid in the graph, accordinging to
    /// [`Self::is_valid_node_id`].
    /// 
    /// It is recommended to call this method from implementations of other methods
    /// that take NodeIds as parameters, to ensure that invalid NodeIds are
    /// caught early.
    fn check_node_id<'a>(&'a self, id: &Self::NodeId<'a>) {
        assert!(
            self.is_maybe_valid_node_id(id),
            "Invalid NodeId: {:?}",
            id
        );
    }

    /// Panics if the given NodeId is not valid in the graph, accordinging to
    /// [`Self::is_maybe_valid_node_id`], but only in debug builds.
    fn debug_check_node_id<'a>(&'a self, id: &Self::NodeId<'a>) {
        debug_assert!(
            self.is_maybe_valid_node_id(id),
            "Invalid NodeId: {:?}",
            id
        );
    }

    /// Gets an iterator over the predacessors nodes of a given node, i.e.
    /// those nodes reachable by incoming edges.
    fn predacessors<'a>(&'a self, node: Self::NodeId<'a>) -> impl Iterator<Item = Self::NodeId<'a>> + 'a {
        let mut visited = HashSet::new();
        self.edges_into(node).filter_map(move |eid| {
            let nid = self.edge_source(eid);
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    /// Gets an iterator over the successor nodes of a given node, i.e.
    /// those nodes reachable by outgoing edges.
    fn successors<'a>(&'a self, node: Self::NodeId<'a>) -> impl Iterator<Item = Self::NodeId<'a>> + 'a {
        let mut visited = HashSet::new();
        self.edges_from(node.clone()).filter_map(move |eid| {
            let nid = if self.is_directed() {
                self.edge_target(eid)
            } else {
                let (source, target) = self.edge_ends(eid);
                if source == node { target } else { source }
            };
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    // Edges

    /// Gets the data associated with an edge.
    fn edge_data<'a>(&'a self, from: Self::EdgeId<'a>) -> &'a Self::EdgeData;

    /// Gets a vector of all edges in the graph.
    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId<'_>> + '_;

    /// Checks if a EdgeId is valid in the graph. This operation is
    /// potentially costly.
    fn is_valid_edge_id(&self, id: &Self::EdgeId<'_>) -> bool {
        self.edge_ids().any(|eid| &eid == id)
    }

    /// Checks if a EdgeId is valid in the graph to the extent that can be
    /// determined without iterating over all nodes.  This may return false
    /// positives for some graph implementations.
    /// 
    /// By default, this method always returns true.
    fn is_maybe_valid_edge_id(&self, _id: &Self::EdgeId<'_>) -> bool {
        true
    }

    /// Panics if the given EdgeId is not valid in the graph, accordinging to
    /// [`Self::is_valid_edge_id`].
    /// 
    /// It is recommended to call this method from implementations of other methods
    /// that take EdgeIds as parameters, to ensure that invalid EdgeIds are
    /// caught early.
    fn check_edge_id(&self, id: &Self::EdgeId<'_>) {
        assert!(
            self.is_maybe_valid_edge_id(id),
            "Invalid EdgeId: {:?}",
            id
        );
    }

    /// Panics if the given EdgeId is not valid in the graph, accordinging to
    /// [`Self::is_maybe_valid_edge_id`], but only in debug builds.
    fn debug_check_edge_id(&self, id: &Self::EdgeId<'_>) {
        debug_assert!(
            self.is_maybe_valid_edge_id(id),
            "Invalid EdgeId: {:?}",
            id
        );
    }
    /// Gets an iterator over the outgoing edges from a given node.
    fn edges_from<'a>(&'a self, from: Self::NodeId<'a>) -> impl Iterator<Item = Self::EdgeId<'a>> + 'a {
        self.edge_ids().filter(move |eid| {
            let (source, target) = self.edge_ends(eid.clone());
            source == from || !self.is_directed() && target == from
        })
    }

    /// Gets an iterator over the incoming edges to a given node.
    fn edges_into<'a>(&'a self, into: Self::NodeId<'a>) -> impl Iterator<Item = Self::EdgeId<'a>> + 'a {
        self.edge_ids().filter(move |eid| {
            let (source, target) = self.edge_ends(eid.clone());
            target == into || !self.is_directed() && source == into
        })
    }

    /// Gets an iterator over the edges between two nodes.
    fn edges_between<'a>(
        &'a self,
        from: Self::NodeId<'a>,
        into: Self::NodeId<'a>,
    ) -> impl Iterator<Item = Self::EdgeId<'a>> + 'a {
        self.edges_from(from.clone()).filter(move |eid| {
            let (edge_source, edge_target) = self.edge_ends(eid.clone());
            edge_source == from && edge_target == into
                || !self.is_directed() && edge_source == into && edge_target == from
        })
    }

    /// Gets the number of edges from one node to another.
    fn num_edges_between<'a>(&'a self, from: Self::NodeId<'a>, into: Self::NodeId<'a>) -> usize {
        self.edges_between(from, into).into_iter().count()
    }

    fn has_edge<'a>(&'a self, from: Self::NodeId<'a>, into: Self::NodeId<'a>) -> bool {
        self.edges_between(from, into).next().is_some()
    }

    /// Gets the source and target nodes of an edge.
    fn edge_ends<'a>(&'a self, eid: Self::EdgeId<'a>) -> (Self::NodeId<'a>, Self::NodeId<'a>);

    /// Gets the source node of an edge.
    fn edge_source<'a>(&'a self, id: Self::EdgeId<'a>) -> Self::NodeId<'a> {
        self.edge_ends(id).0
    }

    /// Gets the target node of an edge.
    fn edge_target<'a>(&'a self, id: Self::EdgeId<'a>) -> Self::NodeId<'a> {
        self.edge_ends(id).1
    }

    fn has_edge_from<'a>(&'a self, from: Self::NodeId<'a>) -> bool {
        self.edges_from(from).next().is_some()
    }

    fn has_edge_into<'a>(&'a self, into: Self::NodeId<'a>) -> bool {
        self.edges_into(into).next().is_some()
    }

    /// Checks if there is an edge between two nodes.
    fn has_edge_between<'a>(&'a self, from: Self::NodeId<'a>, into: Self::NodeId<'a>) -> bool {
        self.edges_between(from, into).next().is_some()
    }

    /// Gets the number of edges in the graph.
    fn num_edges(&self) -> usize {
        self.edge_ids().count()
    }

    fn num_edges_into<'a>(&'a self, into: Self::NodeId<'a>) -> usize {
        self.edges_into(into).into_iter().count()
    }

    fn num_edges_from<'a>(&'a self, from: Self::NodeId<'a>) -> usize {
        self.edges_from(from).into_iter().count()
    }

    /// Given an edge and one of its endpoint nodes, returns the other
    /// endpoint node.  Returns `None` if the edge is a self-loop.  Panics if
    /// the given node is not an endpoint of the edge.
    fn other_end<'a>(&'a self, edge: Self::EdgeId<'a>, node: Self::NodeId<'a>) -> Option<Self::NodeId<'a>> {
        let (source, target) = self.edge_ends(edge);
        if source == node {
            Some(target)
        } else if target == node {
            Some(source)
        } else {
            assert_eq!(source, target); // self-loop
            None
        }
    }

    // Searches

    /// Performs a breadth-first search starting from the given node.
    fn bfs<'a>(&'a self, start: Self::NodeId<'a>) -> BfsIterator<'a, Self> {
        self.bfs_multi(vec![start])
    }

    /// Performs a breadth-first search starting from the given nodes.
    fn bfs_multi<'a>(&'a self, start: Vec<Self::NodeId<'a>>) -> BfsIterator<'a, Self> {
        BfsIterator::new(self, start)
    }

    /// Performs a depth-first search starting from the given node.
    fn dfs<'a>(&'a self, start: Self::NodeId<'a>) -> DfsIterator<'a, Self> {
        self.dfs_multi(vec![start])
    }

    /// Performs a depth-first search starting from the given node.
    fn dfs_multi<'a>(&'a self, start: Vec<Self::NodeId<'a>>) -> DfsIterator<'a, Self> {
        DfsIterator::new(self, start)
    }

    // Pathfinding

    /// Finds shortest paths from a starting node to all other nodes using
    /// Dijkstra's algorithm.  Returns a map from each reachable node to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    fn shortest_paths<C: Zero + Ord + Copy>(
        &self,
        start: Self::NodeId<'_>,
        cost_fn: impl Fn(&Self::EdgeId<'_>) -> C,
    ) -> HashMap<Self::NodeId<'_>, (Vec<Self::NodeId<'_>>, C)> {
        let parents: HashMap<Self::NodeId<'_>, (Self::NodeId<'_>, C)> =
            pathfinding::prelude::dijkstra_all(&start, |nid| -> Vec<(Self::NodeId<'_>, C)> {
                let r: Vec<_> = self
                    .edges_from(nid.clone())
                    .map(|eid| {
                        let cost = cost_fn(&eid);
                        (self.edge_target(eid), cost)
                    })
                    .collect();
                r
            });
        once((start.clone(), (vec![start], C::zero())))
            .chain(
                parents
                    .iter()
                    .map(|(k, (_, cost)): (&Self::NodeId<'_>, &(_, C))| {
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
    fn strongly_connected_component<'a>(&'a self, start: &Self::NodeId<'a>) -> Vec<Self::NodeId<'a>> {
        pathfinding::prelude::strongly_connected_component(start, |nid| {
            self.successors(nid.clone())
        })
    }

    /// Partitions the graph into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components(&self) -> Vec<Vec<Self::NodeId<'_>>> {
        pathfinding::prelude::strongly_connected_components(
            &self.node_ids().collect::<Vec<_>>(),
            |nid| self.successors(nid.clone()),
        )
    }

    /// Partitions nodes reachable from a starting point into strongly connected components.
    #[cfg(feature = "pathfinding")]
    fn strongly_connected_components_from(
        &self,
        start: &Self::NodeId<'_>,
    ) -> Vec<Vec<Self::NodeId<'_>>> {
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
    fn connected_components(&self) -> Vec<HashSet<Self::NodeId<'_>>> {
        pathfinding::prelude::connected_components(&self.node_ids().collect::<Vec<_>>(), |nid| {
            self.successors(nid.clone())
        })
    }
}

impl<G> GraphUndirected for G where G: Graph<Directedness = Undirected> {}

pub trait GraphMut: Graph {
    /// Removes all nodes and edges from the graph.
    fn clear(&mut self) {
        for nid in self.node_ids().collect::<Vec<_>>() {
            self.remove_node(nid);
        }
    }

    /// Adds a node with the given data to the graph, returning its `NodeId`.
    fn add_node<'a>(&'a mut self, data: Self::NodeData) -> Self::NodeId<'a>;

    /// Removes a node from the graph, returning its data.  Any edges
    /// connected to the node are also be removed.
    fn remove_node<'a>(&'a mut self, id: Self::NodeId<'a>) -> Self::NodeData;

    /// Adds an edge with the given data between two nodes and returns the
    /// `EdgeId`.  If an edge already exists between the two nodes, and the
    /// graph does not support parallel edges, the old edge is replaced.
    /// Use [`Self::add_or_replace_edge`] to get the old edge data as well.
    fn add_edge<'a>(
        &'a mut self,
        from: Self::NodeId<'a>,
        to: Self::NodeId<'a>,
        data: Self::EdgeData,
    ) -> Self::EdgeId<'a> {
        self.add_or_replace_edge(from, to, data).0
    }

    /// Adds an edge with the given data between two nodes and returns the
    /// `EdgeId`.  If an edge already exists between the two nodes, and the
    /// graph does not support parallel edges, the old edge is replaced and its
    /// data is returned as well.
    fn add_or_replace_edge<'a>(
        &'a mut self,
        from: Self::NodeId<'a>,
        to: Self::NodeId<'a>,
        data: Self::EdgeData,
    ) -> (Self::EdgeId<'a>, Option<Self::EdgeData>);

    /// Remove an edge between two nodes, returning its data if it existed.
    fn remove_edge<'a>(&'a mut self, from: Self::EdgeId<'a>) -> Option<Self::EdgeData>;

    /// Copies all nodes and edges from another graph into this graph.
    fn copy_from<S>(&mut self, source: &S) -> HashMap<S::NodeId<'_>, Self::NodeId<'_>>
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
    ) -> HashMap<S::NodeId<'_>, Self::NodeId<'_>>
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
            let (from, to) = source.edge_ends(eid.clone());
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
        node_map: &HashMap<S::NodeId<'_>, Self::NodeId<'_>>,
    ) -> HashMap<S::EdgeId<'_>, Self::EdgeId<'_>>
    where
        S: Graph,
    {
        let mut edge_map = HashMap::new();
        for eid in source.edge_ids() {
            let (from1, to1) = source.edge_ends(eid.clone());
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
}
