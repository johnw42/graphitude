use std::{cmp::Ordering, fmt::Debug, hash::Hash, iter::once};

use crate::{EdgeId, Graph};

/// A path in a graph, represented as a sequence of nodes and the edges that
/// connect them.
pub struct Path<G: Graph> {
    edges: Vec<G::EdgeId>,
    nodes: Vec<G::NodeId>,
}

impl<G: Graph> Path<G> {
    /// Creates a new path starting at the given node.
    pub fn new(start: G::NodeId) -> Self {
        Self {
            edges: Vec::new(),
            nodes: vec![start],
        }
    }

    pub fn from_edges(start: G::NodeId, edges: impl IntoIterator<Item = G::EdgeId>) -> Self {
        let mut path = Self::new(start);
        path.extend(edges);
        path
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
    /// connecting them. Each item is a tuple of the for `(incoming_edge,
    /// node, outgoing_edge)`; the edges are optional but will always be
    /// `Some` except for the first node (no incoming edge) and the last
    /// node (no outgoing edge).
    pub fn nodes_with_edges(
        &self,
    ) -> impl Iterator<Item = (Option<G::EdgeId>, G::NodeId, Option<G::EdgeId>)> + '_ {
        let incoming = once(None).chain(self.edges.iter().cloned().map(Some));
        let outgoing = self.edges.iter().cloned().map(Some).chain(once(None));
        let nodes = self.nodes.iter().cloned();
        incoming
            .zip(outgoing)
            .zip(nodes)
            .map(|((in_edge, out_edge), node)| {
                if let Some(ref e) = in_edge {
                    debug_assert!(e.target() == node);
                }
                if let Some(ref e) = out_edge {
                    debug_assert!(e.source() == node);
                }
                (in_edge, node, out_edge)
            })
    }

    /// Adds an edge to the end of the path, extending it to the edge's target
    /// node. Panics if the edge's source node does not match the current
    /// last node of the path.
    pub fn add_edge(&mut self, edge_id: G::EdgeId) {
        assert_eq!(edge_id.source(), self.last_node());
        let target = edge_id.target();
        self.edges.push(edge_id);
        self.nodes.push(target);
    }

    /// Adds an edge and its target node to the end of the path. Panics if
    /// the edge's source node does not match the current last node of the
    /// path, or if the provided node does not match the edge's target node.
    pub fn add_edge_and_node(&mut self, edge_id: G::EdgeId, node_id: G::NodeId) {
        assert_eq!(edge_id.source(), self.last_node());
        assert_eq!(
            edge_id.target(),
            node_id,
            "Edge target does not match provided node"
        );
        self.edges.push(edge_id);
        self.nodes.push(node_id);
    }

    /// Extends the path by appending all edges from another path. Panics if
    /// the first node of the other path does not match the current last
    /// node of this path.
    pub fn extend_with(&mut self, other: &Path<G>) {
        for edge_id in other.edges() {
            self.add_edge(edge_id);
        }
    }
}

impl<G> Clone for Path<G>
where
    G: Graph,
{
    fn clone(&self) -> Self {
        Self {
            edges: self.edges.clone(),
            nodes: self.nodes.clone(),
        }
    }
}

impl<G> PartialEq for Path<G>
where
    G: Graph,
{
    fn eq(&self, other: &Self) -> bool {
        self.edges == other.edges && self.nodes == other.nodes
    }
}

impl<G> Eq for Path<G> where G: Graph {}

impl<G> Hash for Path<G>
where
    G: Graph,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.edges.hash(state);
        self.nodes.hash(state);
    }
}

impl<G> PartialOrd for Path<G>
where
    G: Graph,
{
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
    G: Graph,
{
    fn extend<T: IntoIterator<Item = G::EdgeId>>(&mut self, iter: T) {
        for edge_id in iter {
            self.add_edge(edge_id);
        }
    }
}

impl<G> Debug for Path<G>
where
    G: Graph,
    G::NodeId: Debug + Clone,
    G::EdgeId: Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Path")
            .field("nodes", &self.nodes)
            .field("edges", &self.edges)
            .finish()
    }
}

#[cfg(test)]
#[cfg(feature = "bitvec")]
mod tests {
    use crate::GraphMut as _;
    use crate::adjacency_graph::AdjacencyGraph;

    use super::*;

    type TestGraph = AdjacencyGraph<&'static str, &'static str>;

    #[test]
    fn test_new_path() {
        let mut graph = TestGraph::new();
        let n1 = graph.add_node("n1");
        let path = Path::<TestGraph>::new(n1);
        assert_eq!(path.first_node(), n1);
        assert_eq!(path.last_node(), n1);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = TestGraph::new();
        let n1 = graph.add_node("n1");
        let n2 = graph.add_node("n2");
        let e1 = graph.add_edge(n1, n2, "e1");

        let mut path = Path::<TestGraph>::new(n1);
        path.add_edge(e1);

        assert_eq!(path.last_node(), n2);
        assert_eq!(path.edges().count(), 1);
        assert_eq!(path.nodes().count(), 2);
    }

    #[test]
    fn test_nodes_with_edges() {
        let mut graph = TestGraph::new();
        let n1 = graph.add_node("n1");
        let n2 = graph.add_node("n2");
        let n3 = graph.add_node("n3");
        let e1 = graph.add_edge(n1.clone(), n2.clone(), "e1");
        let e2 = graph.add_edge(n2.clone(), n3.clone(), "e2");
        let mut path = Path::<TestGraph>::new(n1.clone());
        path.add_edge(e1);
        path.add_edge(e2);
        let mut iter = path.nodes_with_edges();
        assert_eq!(iter.next(), Some((None, n1.clone(), Some(e1.clone()))));
        assert_eq!(
            iter.next(),
            Some((Some(e1.clone()), n2.clone(), Some(e2.clone())))
        );
        assert_eq!(iter.next(), Some((Some(e2.clone()), n3.clone(), None)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_extend_with() {
        let mut graph = TestGraph::new();
        let n1 = graph.add_node("n1");
        let n2 = graph.add_node("n2");
        let n3 = graph.add_node("n3");
        let e1 = graph.add_edge(n1, n2, "e12");
        let e2 = graph.add_edge(n2, n3, "e23");

        let mut path1 = Path::<TestGraph>::new(n1);
        path1.add_edge(e1);

        let mut path2 = Path::<TestGraph>::new(n2);
        path2.add_edge(e2);

        path1.extend_with(&path2);

        assert_eq!(path1.nodes().count(), 3);
        assert_eq!(path1.edges().count(), 2);
    }

    #[test]
    fn test_extend() {
        let mut graph = AdjacencyGraph::<&str, &str>::new();
        let n1 = graph.add_node("n1");
        let n2 = graph.add_node("n2");
        let n3 = graph.add_node("n3");
        let e1 = graph.add_edge(n1, n2, "e12");
        let e2 = graph.add_edge(n2, n3, "e23");

        let mut path = Path::<TestGraph>::new(n1);
        path.extend(vec![e1, e2]);

        assert_eq!(path.nodes().count(), 3);
        assert_eq!(path.edges().count(), 2);
    }

    #[test]
    fn test_debug() {
        let mut graph = AdjacencyGraph::<&str, &str>::new();
        let n1 = graph.add_node("n1");
        let n2 = graph.add_node("n2");
        let n3 = graph.add_node("n3");
        let e1 = graph.add_edge(n1, n2, "e12");
        let e2 = graph.add_edge(n2, n3, "e23");
        let e3 = graph.add_edge(n3, n1, "e31");

        let mut path = Path::<TestGraph>::new(n1);
        path.extend(vec![e1, e2, e3]);
        let debug_str = format!("{:?}", path);
        assert!(debug_str.contains("Path"));
        assert!(debug_str.contains("nodes"));
        assert!(debug_str.contains("edges"));
    }
}
