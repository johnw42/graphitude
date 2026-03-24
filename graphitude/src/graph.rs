#[cfg(feature = "dot")]
use std::io;
#[cfg(feature = "pathfinding")]
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    ops::Add,
};

#[cfg(feature = "dot")]
use crate::dot;

use crate::{
    AddEdgeResult, Directedness, EdgeMultiplicity, GraphCopier, GraphImpl, GraphImplMut,
    MultipleEdges,
    format_debug::format_debug,
    path::Path,
    search::{BfsIterator, BfsIteratorWithPaths, DfsIterator, DfsIteratorWithPaths},
};

mod ids {
    use derivative::Derivative;

    use crate::{
        Directed, EdgeIdImpl, GraphImpl, NodeIdImpl, end_pair::EndPair, util::NonDereferenceable,
    };

    #[derive(Derivative)]
    #[derivative(
        Clone(bound = ""),
        Debug(bound = ""),
        Hash(bound = ""),
        PartialEq(bound = ""),
        Eq(bound = ""),
        PartialOrd(bound = ""),
        Ord(bound = "")
    )]
    pub struct NodeId<G: GraphImpl + ?Sized> {
        inner: G::NodeId,
        graph: NonDereferenceable<G>,
    }

    impl<G: GraphImpl + ?Sized> NodeIdImpl for NodeId<G> {}

    #[derive(Derivative)]
    #[derivative(
        Clone(bound = ""),
        Debug(bound = ""),
        Hash(bound = ""),
        PartialEq(bound = ""),
        Eq(bound = ""),
        PartialOrd(bound = ""),
        Ord(bound = "")
    )]
    pub struct EdgeId<G: GraphImpl + ?Sized> {
        inner: G::EdgeId,
        graph: NonDereferenceable<G>,
    }

    impl<G: GraphImpl + ?Sized> EdgeId<G> {
        #[inline(always)]
        fn wrap(&self, inner: G::NodeId) -> NodeId<G> {
            NodeId {
                inner,
                graph: self.graph,
            }
        }

        /// Gets the directedness of the edge, which will match the directedness of
        /// the graph it belongs to.
        pub fn directedness(&self) -> G::Directedness {
            self.inner.directedness()
        }

        /// Gets one end of the edge.  For directed edges, this is the source node.
        /// For undirected edges, this is one of the two nodes, but it is not
        /// specified which one.
        pub fn left(&self) -> NodeId<G> {
            self.wrap(self.inner.left())
        }

        /// Gets the other end of the edge.  For directed edges, this is the target
        /// node.  For undirected edges, this is the other of the two nodes, but it
        /// is not specified which one is which.
        pub fn right(&self) -> NodeId<G> {
            self.wrap(self.inner.right())
        }

        /// Gets both ends of the edge.  Returns `(self.left(), self.right())`.
        /// Implementors must implement either this method or `left` and `right`.
        pub fn ends(&self) -> (NodeId<G>, NodeId<G>) {
            let (left, right) = self.inner.ends();
            (self.wrap(left), self.wrap(right))
        }

        /// Gets the other end of the edge given one end.  If the edge is directed,
        /// the direction is ignored and the other end is returned.  If the edge is
        /// undirected, the other end is returned regardless of which end is passed
        /// in.  If the edge is a self-loop and the given end is the same as both
        /// ends of the edge, then the same node ID is returned.  Panics if the given
        /// node ID is not one of the ends of the edge.
        pub fn other_end(&self, node: &NodeId<G>) -> NodeId<G> {
            self.wrap(self.inner.other_end(&node.inner))
        }

        /// Tests if the edge has the given node as one of its ends.
        pub fn has_end(&self, node: &NodeId<G>) -> bool {
            self.inner.has_end(&node.inner)
        }

        /// Tests if the edge has the given nodes as its ends, regardless of order.
        pub fn has_ends(&self, node1: &NodeId<G>, node2: &NodeId<G>) -> bool {
            self.inner.has_ends(&node1.inner, &node2.inner)
        }

        /// Gets the source node of the edge.
        pub fn source(&self) -> NodeId<G>
        where
            G: EdgeIdImpl<Directedness = Directed>,
        {
            self.left()
        }

        /// Gets the target node of the edge.
        pub fn target(&self) -> NodeId<G>
        where
            G: EdgeIdImpl<Directedness = Directed>,
        {
            self.right()
        }
    }

    // The trait impl mostly just calls inherent methods of the same name.  The
    // methods are inherent instead of trait methods to allow them to be called
    // on EdgeIds without needing to import the trait, which is more ergonomic
    // since these methods are commonly used and it's not important to be able
    // to call them on trait objects or generic parameters.
    impl<G: GraphImpl + ?Sized> EdgeIdImpl for EdgeId<G> {
        type Directedness = G::Directedness;
        type NodeId = NodeId<G>;

        fn directedness(&self) -> G::Directedness {
            self.directedness()
        }

        fn left(&self) -> NodeId<G> {
            self.left()
        }

        fn right(&self) -> NodeId<G> {
            self.right()
        }

        fn ends(&self) -> (NodeId<G>, NodeId<G>) {
            self.ends()
        }

        fn other_end(&self, node: &NodeId<G>) -> NodeId<G> {
            self.other_end(node)
        }

        fn has_end(&self, node: &NodeId<G>) -> bool {
            self.has_end(node)
        }

        fn has_ends(&self, node1: &NodeId<G>, node2: &NodeId<G>) -> bool {
            self.has_ends(node1, node2)
        }

        fn into_ends(self) -> EndPair<Self::NodeId, Self::Directedness> {
            let directedness = self.directedness();
            let (left, right) = self.inner.into_ends().into_values();
            EndPair::new(
                NodeId {
                    inner: left,
                    graph: self.graph,
                },
                NodeId {
                    inner: right,
                    graph: self.graph,
                },
                directedness,
            )
        }
    }

    /// A trait that abstracts over the common behavior of NodeId and EdgeId,
    /// allowing them to be wrapped and unwrapped from their inner
    /// graph-specific IDs.
    pub(super) trait IdWrapper<G: GraphImpl + ?Sized> {
        type Inner;

        fn wrap(inner: Self::Inner, graph: *const G) -> Self;
        fn unwrap(&self, graph: *const G) -> &Self::Inner;
    }

    impl<G: GraphImpl + ?Sized> IdWrapper<G> for NodeId<G> {
        type Inner = G::NodeId;

        #[inline(always)]
        fn wrap(inner: Self::Inner, graph: *const G) -> Self {
            NodeId {
                inner,
                graph: graph.into(),
            }
        }

        #[inline(always)]
        fn unwrap(&self, graph: *const G) -> &Self::Inner {
            assert_eq!(self.graph, graph.into());
            &self.inner
        }
    }

    impl<G: GraphImpl + ?Sized> IdWrapper<G> for EdgeId<G> {
        type Inner = G::EdgeId;

        #[inline(always)]
        fn wrap(inner: Self::Inner, graph: *const G) -> Self {
            EdgeId {
                inner,
                graph: graph.into(),
            }
        }

        #[inline(always)]
        fn unwrap(&self, graph: *const G) -> &Self::Inner {
            assert_eq!(self.graph, graph.into());
            &self.inner
        }
    }
}

use ids::IdWrapper;
pub use ids::{EdgeId, NodeId};

pub struct Graph<G: GraphImpl + ?Sized> {
    inner: Box<G>,
}

impl<G> Graph<G>
where
    G: GraphImpl,
{
    #[inline(always)]
    fn wrap<W: IdWrapper<G>>(&self, node_id: W::Inner) -> W {
        W::wrap(node_id, &*self.inner)
    }

    #[inline(always)]
    fn unwrap<'a, 'b, W: IdWrapper<G>>(&'a self, node_id: &'b W) -> &'b W::Inner {
        node_id.unwrap(&*self.inner)
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// The directedness of the graph.
    pub fn directedness(&self) -> G::Directedness {
        self.inner.directedness()
    }

    /// The edge multiplicity of the graph.
    pub fn edge_multiplicity(&self) -> G::EdgeMultiplicity {
        self.inner.edge_multiplicity()
    }

    /// Returns true if the graph is directed.
    pub fn is_directed(&self) -> bool {
        self.directedness().is_directed()
    }

    /// Returns true if the graph allows parallel edges between the same pair of nodes.
    pub fn allows_parallel_edges(&self) -> bool {
        self.edge_multiplicity().allows_parallel_edges()
    }

    /// Writes a DOT representation of the graph to the given output.
    #[cfg(feature = "dot")]
    pub fn write_dot<D>(
        &self,
        generator: &D,
        output: &mut impl io::Write,
    ) -> Result<(), dot::renderer::DotError<D::Error>>
    where
        D: dot::renderer::DotGenerator<G>,
        Self: Sized,
    {
        dot::renderer::generate_dot_file(self, generator, output)
    }

    /// Generates a DOT representation of the graph as a String.
    #[cfg(feature = "dot")]
    pub fn to_dot_string<D>(
        &self,
        generator: &D,
    ) -> Result<String, dot::renderer::DotError<D::Error>>
    where
        D: dot::renderer::DotGenerator<G>,
        Self: Sized,
    {
        let mut output = Vec::new();
        self.write_dot(generator, &mut output)?;
        Ok(String::from_utf8(output).expect("Generated DOT is not valid UTF-8"))
    }

    /// Creates a new path starting from the given starting node.  This is a
    /// convenience method to avoid having to import the `Path` type separately
    /// and specify its type argument explicity.
    pub fn new_path(&self, start: &NodeId<G>) -> Path<G> {
        Path::new(start.clone())
    }

    /// Gets a vector of all NodeIds in the graph.
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId<G>> {
        self.inner.node_ids().map(|nid| self.wrap(nid))
    }

    /// Gets the data associated with a node.
    pub fn node_data<'a>(&'a self, id: &NodeId<G>) -> &'a G::NodeData {
        self.inner.node_data(self.unwrap(id))
    }

    /// Gets the number of nodes in the graph.
    pub fn num_nodes(&self) -> usize {
        self.inner.num_nodes()
    }

    /// Gets an iterator over the predecessors nodes of a given node, i.e.
    /// those nodes reachable by incoming edges.
    pub fn predecessors<'a, 'b: 'a>(
        &'a self,
        node: &'b NodeId<G>,
    ) -> impl Iterator<Item = NodeId<G>> + 'a {
        let mut visited = HashSet::new();
        self.edges_into(node).filter_map(move |eid| {
            let nid = if self.is_directed() {
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
    pub fn successors<'a, 'b: 'a>(
        &'a self,
        node: &'b NodeId<G>,
    ) -> impl Iterator<Item = NodeId<G>> + 'a {
        let mut visited = HashSet::new();
        self.edges_from(node).filter_map(move |eid| {
            let nid = if self.is_directed() {
                eid.right()
            } else {
                let (source, target) = eid.ends();
                if source == *node { target } else { source }
            };
            visited.insert(nid.clone()).then_some(nid)
        })
    }

    /// Gets the data associated with an edge.
    pub fn edge_data<'a>(&'a self, id: &EdgeId<G>) -> &'a G::EdgeData {
        self.inner.edge_data(self.unwrap(id))
    }

    /// Gets a vector of all edges in the graph.
    pub fn edge_ids(&self) -> impl Iterator<Item = EdgeId<G>> + '_ {
        self.inner
            .edge_ids()
            .map(|eid| EdgeId::wrap(eid, &*self.inner))
    }

    /// Gets an iterator over the outgoing edges from a given node.
    pub fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b NodeId<G>,
    ) -> impl Iterator<Item = EdgeId<G>> + 'a {
        self.inner
            .edges_from(self.unwrap(from))
            .map(|id| self.wrap(id))
    }

    /// Gets an iterator over the incoming edges to a given node.
    pub fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b NodeId<G>,
    ) -> impl Iterator<Item = EdgeId<G>> + 'a {
        self.inner
            .edges_into(self.unwrap(into))
            .map(|id| self.wrap(id))
    }

    /// Gets an iterator over the edges from one node into another.
    pub fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b NodeId<G>,
        into: &'b NodeId<G>,
    ) -> impl Iterator<Item = EdgeId<G>> + 'a {
        self.inner
            .edges_from_into(self.unwrap(from), self.unwrap(into))
            .map(|id| self.wrap(id))
    }

    /// Checks if there is at least one outgoing edge from the given node.
    pub fn has_edge_from(&self, from: &NodeId<G>) -> bool {
        self.inner.has_edge_from(self.unwrap(from))
    }

    /// Checks if there is at least one incoming edge to the given node.
    pub fn has_edge_into(&self, into: &NodeId<G>) -> bool {
        self.inner.has_edge_into(self.unwrap(into))
    }

    /// Checks if there at least one edge from one node to another.
    pub fn has_edge_from_into(&self, from: &NodeId<G>, into: &NodeId<G>) -> bool {
        self.inner
            .has_edge_from_into(self.unwrap(from), self.unwrap(into))
    }

    /// Gets the number of edges in the graph.
    pub fn num_edges(&self) -> usize {
        self.inner.num_edges()
    }

    /// Gets the number of incoming edges to a given node.
    pub fn num_edges_into(&self, into: &NodeId<G>) -> usize {
        self.inner.num_edges_into(self.unwrap(into))
    }

    /// Gets the number of outgoing edges from a given node.
    pub fn num_edges_from(&self, from: &NodeId<G>) -> usize {
        self.inner.num_edges_from(self.unwrap(from))
    }

    /// Gets the number of edges from one node into another.
    pub fn num_edges_from_into(&self, from: &NodeId<G>, into: &NodeId<G>) -> usize {
        self.inner
            .num_edges_from_into(self.unwrap(from), self.unwrap(into))
    }

    // Searches

    /// Performs a breadth-first search starting from the given node.
    pub fn bfs(&self, start: &NodeId<G>) -> BfsIterator<'_, G> {
        self.bfs_multi(vec![start.clone()])
    }

    /// Performs a breadth-first search starting from the given nodes.
    pub fn bfs_multi(&self, start: Vec<NodeId<G>>) -> BfsIterator<'_, G> {
        BfsIterator::new(self, start)
    }

    /// Performs a depth-first search starting from the given node.
    pub fn dfs(&self, start: &NodeId<G>) -> DfsIterator<'_, G> {
        self.dfs_multi(vec![start.clone()])
    }

    /// Performs a depth-first search starting from the given node.
    pub fn dfs_multi(&self, start: Vec<NodeId<G>>) -> DfsIterator<'_, G> {
        DfsIterator::new(self, start)
    }

    /// Performs a breadth-first search starting from the given node.
    pub fn bfs_with_paths(&self, start: &NodeId<G>) -> BfsIteratorWithPaths<'_, G> {
        self.bfs_multi_with_paths(vec![start.clone()])
    }

    /// Performs a breadth-first search starting from the given nodes.
    pub fn bfs_multi_with_paths(&self, start: Vec<NodeId<G>>) -> BfsIteratorWithPaths<'_, G> {
        BfsIteratorWithPaths::new(self, start)
    }

    /// Performs a depth-first search starting from the given node.
    pub fn dfs_with_paths(&self, start: &NodeId<G>) -> DfsIteratorWithPaths<'_, G> {
        self.dfs_multi_with_paths(vec![start.clone()])
    }

    /// Performs a depth-first search starting from the given nodes.
    pub fn dfs_multi_with_paths(&self, start: Vec<NodeId<G>>) -> DfsIteratorWithPaths<'_, G> {
        DfsIteratorWithPaths::new(self, start)
    }

    // Pathfinding

    /// Finds shortest paths from a starting node to all other nodes using
    /// Dijkstra's algorithm.  Returns a map from each reachable node to a
    /// tuple of the path taken and the total cost.
    #[cfg(feature = "pathfinding")]
    pub fn shortest_paths<C: Default + Ord + Copy + Add<Output = C>>(
        &self,
        start: &NodeId<G>,
        distance_fn: impl Fn(&EdgeId<G>) -> C,
    ) -> HashMap<NodeId<G>, (Path<G>, C)> {
        // Find shortest paths using Dijkstra's algorithm.

        use std::collections::HashMap;

        let mut distances: HashMap<NodeId<G>, C> = HashMap::new();
        let mut predecessors: HashMap<NodeId<G>, (EdgeId<G>, NodeId<G>)> = HashMap::new();
        let mut unvisited: HashSet<NodeId<G>> = self.node_ids().collect();

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
                let neighbor = edge_id.other_end(&current_node);
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
        let mut result: HashMap<NodeId<G>, (Path<G>, C)> = HashMap::new();
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
}

impl<G: GraphImplMut> Graph<G> {
    /// Creates a new graph with the given directedness and edge multiplicity.
    pub fn new(directedness: G::Directedness, edge_multiplicity: G::EdgeMultiplicity) -> Self
    where
        Self: Sized,
    {
        Self {
            inner: Box::new(G::new(directedness, edge_multiplicity)),
        }
    }

    /// Gets a mutable reference to the data associated with a node.
    pub fn node_data_mut<'a>(&'a mut self, id: &'a NodeId<G>) -> &'a mut G::NodeData {
        self.inner.node_data_mut(self.unwrap(id))
    }

    /// Gets a mutable reference to the data associated with an edge.
    pub fn edge_data_mut<'a>(&'a mut self, id: &'a EdgeId<G>) -> &'a mut G::EdgeData {
        self.inner.edge_data_mut(self.unwrap(id))
    }

    /// Removes all nodes and edges from the graph.
    pub fn clear(&mut self) {
        for nid in self.node_ids().collect::<Vec<_>>() {
            self.remove_node(&nid);
        }
    }

    /// Adds a node with the given data to the graph and returns its NodeId.
    pub fn add_node(&mut self, data: G::NodeData) -> NodeId<G> {
        let node_id = self.inner.add_node(data);
        self.wrap(node_id)
    }

    /// Removes a node from the graph, returning its data.  Any edges
    /// connected to the node are also be removed.
    pub fn remove_node(&mut self, id: &NodeId<G>) -> G::NodeData {
        self.inner.remove_node(self.unwrap(id))
    }

    /// Adds an edge with the given data between two nodes and returns the
    /// `EdgeId`.  Use [`Self::add_edge`] for graphs that do not
    /// support parallel edges.
    pub fn add_new_edge(
        &mut self,
        from: &NodeId<G>,
        into: &NodeId<G>,
        data: G::EdgeData,
    ) -> EdgeId<G>
    where
        G: GraphImpl<EdgeMultiplicity = MultipleEdges>,
    {
        match self.add_edge(from, into, data) {
            AddEdgeResult::Added(eid) => eid,
            AddEdgeResult::Updated(_, _) => {
                unreachable!("Edge already exists between {:?} and {:?}", from, into)
            }
        }
    }

    /// Adds an edge from one node to another with the given data and returns its EdgeId.
    pub fn add_edge(
        &mut self,
        from: &NodeId<G>,
        into: &NodeId<G>,
        data: G::EdgeData,
    ) -> AddEdgeResult<EdgeId<G>, G::EdgeData> {
        self.inner
            .add_edge(self.unwrap(from), self.unwrap(into), data)
            .map_edge_id(|id| self.wrap(id))
    }

    /// Remove an edge between two nodes, returning its data.
    pub fn remove_edge(&mut self, id: &EdgeId<G>) -> G::EdgeData {
        self.inner.remove_edge(self.unwrap(id))
    }

    /// Reserves capacity for at least the given number of additional nodes
    /// and edges.  Does nothing by default.
    pub fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        let _ = additional_nodes;
        let _ = additional_edges;
    }

    /// Reserves the exact capacity for the given number of additional nodes
    /// and edges.  Does nothing by default.
    pub fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        let _ = additional_nodes;
        let _ = additional_edges;
    }

    /// Compacts internal storage used by the graph to minimize memory usage
    /// without reallocation.  Does nothing by default.  May invalidate existing
    /// NodeIds and EdgeIds.
    pub fn compact(&mut self) {
        self.compact_with(|_, _| {}, |_, _| {});
    }

    /// Compacts internal storage used by the graph to minimize memory usage
    /// without reallocation.  Does nothing by default.  May invalidate existing
    /// NodeIds and EdgeIds.  Calls a closure for each node ID mapping
    /// (old_id, new_id) and edge ID mapping (old_id, new_id) as they are created.
    pub fn compact_with(
        &mut self,
        mut node_id_callback: impl FnMut(&'_ NodeId<G>, &'_ NodeId<G>),
        mut edge_id_callback: impl FnMut(&'_ EdgeId<G>, &'_ EdgeId<G>),
    ) {
        let inner = &*self.inner as *const G;
        self.inner.compact_with(
            |old_id, new_id| {
                node_id_callback(
                    &NodeId::wrap(old_id.clone(), inner),
                    &NodeId::wrap(new_id.clone(), inner),
                )
            },
            |old_id, new_id| {
                edge_id_callback(
                    &EdgeId::wrap(old_id.clone(), inner),
                    &EdgeId::wrap(new_id.clone(), inner),
                )
            },
        );
    }

    /// Shrinks internal storage used by the graph to fit its current size.
    /// Does nothing by default.
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    /// Parses a DOT representation of a graph from a string, using the given
    /// graph builder to construct the graph.
    #[cfg(feature = "dot")]
    pub fn from_dot_string<B>(
        data: &str,
        builder: &mut B,
    ) -> Result<Self, dot::parser::ParseError<B>>
    where
        B: dot::parser::GraphBuilder<GraphImpl = G>,
    {
        dot::parser::parse_dot_into_graph(data, builder)
    }
}

// This trait impl just delegates to inherent methods of the same name.  The
// methods are inherent instead of trait methods to allow them to be called on
// Graphs without needing to import the traits, and to make the generated
// documentation easier to navigate.
impl<G: GraphImpl> GraphImpl for Graph<G> {
    type NodeId = NodeId<G>;
    type EdgeId = EdgeId<G>;
    type NodeData = G::NodeData;
    type EdgeData = G::EdgeData;
    type Directedness = G::Directedness;
    type EdgeMultiplicity = G::EdgeMultiplicity;

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn num_nodes(&self) -> usize {
        self.num_nodes()
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b NodeId<G>,
    ) -> impl Iterator<Item = EdgeId<G>> + 'a {
        self.edges_from(from)
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b NodeId<G>,
    ) -> impl Iterator<Item = EdgeId<G>> + 'a {
        self.edges_into(into)
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b NodeId<G>,
        into: &'b NodeId<G>,
    ) -> impl Iterator<Item = EdgeId<G>> + 'a {
        self.edges_from_into(from, into)
    }

    fn has_edge_from(&self, from: &Self::NodeId) -> bool {
        self.has_edge_from(from)
    }

    fn has_edge_into(&self, into: &Self::NodeId) -> bool {
        self.has_edge_into(into)
    }

    fn has_edge_from_into(&self, from: &Self::NodeId, into: &Self::NodeId) -> bool {
        self.has_edge_from_into(from, into)
    }

    fn num_edges(&self) -> usize {
        self.num_edges()
    }

    fn num_edges_into(&self, into: &Self::NodeId) -> usize {
        self.num_edges_into(into)
    }

    fn num_edges_from(&self, from: &Self::NodeId) -> usize {
        self.num_edges_from(from)
    }

    fn num_edges_from_into(&self, from: &Self::NodeId, into: &Self::NodeId) -> usize {
        self.num_edges_from_into(from, into)
    }

    fn directedness(&self) -> G::Directedness {
        self.directedness()
    }

    fn edge_multiplicity(&self) -> G::EdgeMultiplicity {
        self.edge_multiplicity()
    }

    fn node_ids(&self) -> impl Iterator<Item = NodeId<G>> {
        self.node_ids()
    }

    fn node_data<'a>(&'a self, id: &NodeId<G>) -> &'a G::NodeData {
        self.node_data(id)
    }

    fn edge_data<'a>(&'a self, id: &EdgeId<G>) -> &'a G::EdgeData {
        self.edge_data(id)
    }

    fn edge_ids(&self) -> impl Iterator<Item = EdgeId<G>> + '_ {
        self.edge_ids()
    }
}

// See comment on GraphImpl impl above.
impl<G: GraphImplMut> GraphImplMut for Graph<G> {
    fn new(directedness: Self::Directedness, edge_multiplicity: Self::EdgeMultiplicity) -> Self
    where
        Self: Sized,
    {
        Self::new(directedness, edge_multiplicity)
    }

    fn clear(&mut self) {
        self.clear();
    }

    fn node_data_mut<'a>(&'a mut self, id: &'a Self::NodeId) -> &'a mut Self::NodeData {
        self.node_data_mut(id)
    }

    fn edge_data_mut<'a>(&'a mut self, id: &'a Self::EdgeId) -> &'a mut Self::EdgeData {
        self.edge_data_mut(id)
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        self.add_node(data)
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData {
        self.remove_node(id)
    }

    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> AddEdgeResult<Self::EdgeId, Self::EdgeData> {
        self.add_edge(from, into, data)
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData {
        self.remove_edge(id)
    }

    fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.reserve(additional_nodes, additional_edges);
    }

    fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.reserve_exact(additional_nodes, additional_edges);
    }

    fn compact_with(
        &mut self,
        node_id_callback: impl FnMut(&'_ Self::NodeId, &'_ Self::NodeId),
        edge_id_callback: impl FnMut(&'_ Self::EdgeId, &'_ Self::EdgeId),
    ) {
        self.compact_with(node_id_callback, edge_id_callback);
    }

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit();
    }
}

impl<G: GraphImpl> From<G> for Graph<G>
where
    G: GraphImpl,
{
    fn from(value: G) -> Self {
        Graph {
            inner: Box::new(value),
        }
    }
}

impl<G: GraphImpl> Default for Graph<G>
where
    G: Default,
{
    fn default() -> Self {
        Self {
            inner: Box::new(G::default()),
        }
    }
}

impl<G: GraphImplMut> Clone for Graph<G>
where
    G::NodeData: Clone,
    G::EdgeData: Clone,
{
    fn clone(&self) -> Self {
        GraphCopier::new(self).clone_nodes().clone_edges().copy()
    }
}

impl<G: GraphImpl> Debug for Graph<G>
where
    G::NodeData: Debug,
    G::EdgeData: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "Graph")
    }
}
