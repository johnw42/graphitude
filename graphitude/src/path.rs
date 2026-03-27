use std::{cmp::Ordering, iter::once};

use derivative::Derivative;

use crate::{Directedness, EdgeIdImpl, GraphImpl};

/// A helper struct returned by [`Path::nodes_with_edges`] that contains a node
/// along with its incoming and outgoing edges in the path.  The edges are
/// optional because the first node in the path has no incoming edge and the
/// last node has no outgoing edge.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Debug(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = "")
)]
pub struct NodeWithEdges<G: GraphImpl + ?Sized> {
    pub node: G::NodeId,
    pub edge_in: Option<G::EdgeId>,
    pub edge_out: Option<G::EdgeId>,
}

/// A path in a graph, represented as a sequence of nodes and the edges that
/// connect them.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    Debug(bound = "")
)]
pub struct Path<G: GraphImpl + ?Sized> {
    edges: Vec<G::EdgeId>,
    nodes: Vec<G::NodeId>,
    directedness: G::Directedness,
}

impl<G> Path<G>
where
    G: GraphImpl + ?Sized,
{
    /// Creates a new path starting at the given node.
    pub(crate) fn new(start: G::NodeId, directedness: G::Directedness) -> Self {
        Self {
            edges: Vec::new(),
            nodes: vec![start],
            directedness,
        }
    }

    /// Returns the first node in the path.
    pub fn first_node(&self) -> G::NodeId {
        self.nodes.first().expect("Path has no nodes").clone()
    }

    /// Returns the last node in the path.
    pub fn last_node(&self) -> G::NodeId {
        self.nodes.last().expect("Path has no nodes").clone()
    }

    /// Returns an iterator over the edges in the path.
    pub fn edges(&self) -> impl Iterator<Item = G::EdgeId> + '_ {
        self.edges.iter().cloned()
    }

    /// Returns an iterator over the nodes in the path.
    pub fn nodes(&self) -> impl Iterator<Item = G::NodeId> + '_ {
        self.nodes.iter().cloned()
    }

    /// Returns an iterator over the nodes in the path along with the edges
    /// connecting them. The first node will have no incoming edge and the last
    /// node will have no outgoing edge.  Other nodes will have both an incoming
    /// and outgoing edge.
    pub fn nodes_with_edges(&self) -> impl Iterator<Item = NodeWithEdges<G>> + '_ {
        let incoming = once(None).chain(self.edges.iter().cloned().map(Some));
        let outgoing = self.edges.iter().cloned().map(Some).chain(once(None));
        let nodes = self.nodes.iter().cloned();
        incoming
            .zip(outgoing)
            .zip(nodes)
            .map(|((edge_in, edge_out), node)| {
                if let Some(ref e) = edge_in {
                    debug_assert!(e.has_end(&node));
                }
                if let Some(ref e) = edge_out {
                    debug_assert!(e.has_end(&node));
                }
                NodeWithEdges {
                    node,
                    edge_in,
                    edge_out,
                }
            })
    }

    /// Adds an edge to the end of the path, extending it to the edge's target
    /// node. Panics if the edge does not connect the current last node of the
    /// path to its target node.
    ///
    /// Prefer [`Self::push_with_node`] for slightly better error checking.
    pub fn push(&mut self, edge_id: G::EdgeId) {
        let next_node = edge_id.other_end(&self.last_node());
        self.push_with_node(edge_id, next_node);
    }

    /// Adds an edge and its target node to the end of the path. Panics if
    /// the edge does not connect the current last node of the path to its
    /// target node.
    pub fn push_with_node(&mut self, edge_id: G::EdgeId, node_id: G::NodeId) {
        let last = self.last_node();

        if self.directedness.is_directed() {
            assert!(
                edge_id.left() == last && edge_id.right() == node_id,
                "Edge does not connect last node to provided node"
            );
        } else {
            assert!(
                edge_id.has_ends(&last, &node_id),
                "Edge does not connect last node to provided node"
            );
        }

        self.edges.push(edge_id);
        self.nodes.push(node_id);
    }
}

impl<G: GraphImpl + ?Sized> PartialOrd for Path<G> {
    /// A path is "less than" another path if its last node matches the
    /// other's first node, and "greater than" if its first node matches the
    /// other's last node. If neither condition is met and the paths are not
    /// equal, they are considered unordered.
    fn partial_cmp(&self, other: &Path<G>) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.last_node() == other.first_node() {
            Some(Ordering::Less)
        } else if self.first_node() == other.last_node() {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl<G> Extend<G::EdgeId> for Path<G>
where
    G: GraphImpl + ?Sized,
{
    fn extend<T: IntoIterator<Item = G::EdgeId>>(&mut self, iter: T) {
        for edge_id in iter {
            self.push(edge_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use test_suite_macro::test_suite_macro;

    use crate::{
        AdjacencyGraph, Directed, GraphImplMut, Undirected,
        path::{NodeWithEdges, Path},
    };

    struct PathTests<G>(PhantomData<G>);

    #[test_suite_macro(path_tests)]
    impl<G> PathTests<G>
    where
        G: GraphImplMut<NodeData = &'static str, EdgeData = &'static str> + Default,
    {
        #[test]
        fn test_new_path() {
            let mut graph = G::default();
            let n1 = graph.add_node("n1");
            let path = Path::<G>::new(n1.clone(), graph.directedness());
            assert_eq!(path.first_node(), n1);
            assert_eq!(path.last_node(), n1);
        }

        #[test]
        fn test_add_edge() {
            let mut graph = G::default();
            let n1 = graph.add_node("n1");
            let n2 = graph.add_node("n2");
            let e1 = graph.add_edge(&n1, &n2, "e1").edge_id();

            let mut path = Path::<G>::new(n1, graph.directedness());
            path.push(e1);

            assert_eq!(path.last_node(), n2);
            assert_eq!(path.edges().count(), 1);
            assert_eq!(path.nodes().count(), 2);
        }

        #[test]
        fn test_nodes_with_edges() {
            let mut graph = G::default();
            let n1 = graph.add_node("n1");
            let n2 = graph.add_node("n2");
            let n3 = graph.add_node("n3");
            let e1 = graph.add_edge(&n1, &n2, "e1").edge_id();
            let e2 = graph.add_edge(&n2, &n3, "e2").edge_id();
            let mut path = Path::<G>::new(n1.clone(), graph.directedness());
            path.push(e1.clone());
            path.push(e2.clone());
            let mut iter = path.nodes_with_edges();
            assert_eq!(
                iter.next(),
                Some(NodeWithEdges {
                    node: n1.clone(),
                    edge_in: None,
                    edge_out: Some(e1.clone())
                })
            );
            assert_eq!(
                iter.next(),
                Some(NodeWithEdges {
                    node: n2.clone(),
                    edge_in: Some(e1.clone()),
                    edge_out: Some(e2.clone())
                })
            );
            assert_eq!(
                iter.next(),
                Some(NodeWithEdges {
                    node: n3.clone(),
                    edge_in: Some(e2.clone()),
                    edge_out: None
                })
            );
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn test_extend() {
            let mut graph = G::default();
            let n1 = graph.add_node("n1");
            let n2 = graph.add_node("n2");
            let n3 = graph.add_node("n3");
            let e1 = graph.add_edge(&n1, &n2, "e12").edge_id();
            let e2 = graph.add_edge(&n2, &n3, "e23").edge_id();

            let mut path = Path::<G>::new(n1, graph.directedness());
            path.extend(vec![e1, e2]);

            assert_eq!(path.nodes().count(), 3);
            assert_eq!(path.edges().count(), 2);
        }

        #[test]
        fn test_debug() {
            let mut graph = G::default();
            let n1 = graph.add_node("n1");
            let n2 = graph.add_node("n2");
            let n3 = graph.add_node("n3");
            let e1 = graph.add_edge(&n1, &n2, "e12").edge_id();
            let e2 = graph.add_edge(&n2, &n3, "e23").edge_id();
            let e3 = graph.add_edge(&n3, &n1, "e31").edge_id();

            let mut path = Path::<G>::new(n1, graph.directedness());
            path.extend(vec![e1, e2, e3]);
            let debug_str = format!("{:?}", path);
            assert!(debug_str.contains("Path"));
            assert!(debug_str.contains("nodes"));
            assert!(debug_str.contains("edges"));
        }
    }

    path_tests!(directed: PathTests<AdjacencyGraph<&'static str, &'static str, Directed>>);
    path_tests!(undirected: PathTests<AdjacencyGraph<&'static str, &'static str, Undirected>>);
}
