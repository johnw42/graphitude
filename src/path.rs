use std::{cmp::Ordering, fmt::Debug, hash::Hash, iter::once};

use crate::{Graph, GraphMut as _, LinkedGraph, debug::format_debug};

/// A path in a graph, represented as a sequence of nodes and the edges that
/// connect them.
pub struct Path<'g, G: Graph> {
    graph: &'g G,
    edges: Vec<G::EdgeId>,
    nodes: Vec<G::NodeId>,
}

impl<'g, G: Graph> Path<'g, G> {
    /// Creates a new path starting at the given node.
    pub fn new(graph: &'g G, start: G::NodeId) -> Self {
        Self {
            graph,
            edges: Vec::new(),
            nodes: vec![start],
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
                    debug_assert!(self.graph.edge_target(e.clone()) == node);
                }
                if let Some(ref e) = out_edge {
                    debug_assert!(self.graph.edge_source(e.clone()) == node);
                }
                (in_edge, node, out_edge)
            })
    }

    /// Adds an edge to the end of the path, extending it to the edge's target
    /// node. Panics if the edge's source node does not match the current
    /// last node of the path.
    pub fn add_edge(&mut self, edge_id: G::EdgeId) {
        assert_eq!(self.graph.edge_source(edge_id.clone()), self.last_node());
        self.edges.push(edge_id.clone());
        self.nodes.push(
            self.graph
                .other_end(edge_id, self.last_node())
                .unwrap_or_else(|| self.last_node()),
        );
    }

    /// Extends the path by appending all edges from another path. Panics if
    /// the first node of the other path does not match the current last
    /// node of this path.
    pub fn extend_with(&mut self, other: &Path<'g, G>) {
        for edge_id in other.edges() {
            self.add_edge(edge_id);
        }
    }
}

impl<'g, G> Clone for Path<'g, G>
where
    G: Graph,
{
    fn clone(&self) -> Self {
        Self {
            graph: self.graph,
            edges: self.edges.clone(),
            nodes: self.nodes.clone(),
        }
    }
}

impl<'g, G> PartialEq for Path<'g, G>
where
    G: Graph,
{
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.graph, other.graph)
            && self.edges == other.edges
            && self.nodes == other.nodes
    }
}

impl<'g, G> Eq for Path<'g, G> where G: Graph {}

impl<'g, G> Hash for Path<'g, G>
where
    G: Graph,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.graph as *const G).hash(state);
        self.edges.hash(state);
        self.nodes.hash(state);
    }
}

impl<'g, G> PartialOrd for Path<'g, G>
where
    G: Graph,
{
    /// A path is "less than" another path if its last node matches the
    /// other's first node, and "greater than" if its first node matches the
    /// other's last node. If neither condition is met and the paths are not
    /// equal, they are considered unordered.
    fn partial_cmp(&self, other: &Path<'g, G>) -> Option<std::cmp::Ordering> {
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

impl<G> Extend<G::EdgeId> for Path<'_, G>
where
    G: Graph,
{
    fn extend<T: IntoIterator<Item = G::EdgeId>>(&mut self, iter: T) {
        for edge_id in iter {
            self.add_edge(edge_id);
        }
    }
}

impl<'g, G> Debug for Path<'g, G>
where
    G: Graph,
    G::NodeId: Debug + Clone,
    G::EdgeId: Debug + Clone,
    G::NodeData: Debug,
    G::EdgeData: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Create a temporary LinkedGraph to leverage `format_debug`.
        let mut temp_graph = LinkedGraph::<&'g G::NodeData, &'g G::EdgeData>::new();
        let mut prev_new_vid = None;
        for (eid, vid, _) in self.nodes_with_edges() {
            let v_data = self.graph.node_data(vid.clone());
            let new_vid = temp_graph.add_node(v_data);
            if let Some(eid) = eid {
                let e_data = self.graph.edge_data(eid.clone());
                temp_graph.add_edge(prev_new_vid.take().unwrap(), new_vid.clone(), e_data);
            }
            prev_new_vid = Some(new_vid);
        }
        format_debug(&temp_graph, f, "Path")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_path() {
        let mut graph = LinkedGraph::<i32, ()>::new();
        let v1 = graph.add_node(1);
        let path = Path::new(&graph, v1);
        assert_eq!(path.first_node(), v1);
        assert_eq!(path.last_node(), v1);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_node(1);
        let v2 = graph.add_node(2);
        let e1 = graph.add_edge(v1, v2, ());

        let mut path = Path::new(&graph, v1);
        path.add_edge(e1);

        assert_eq!(path.last_node(), v2);
        assert_eq!(path.edges().count(), 1);
        assert_eq!(path.nodes().count(), 2);
    }

    #[test]
    fn test_nodes_with_edges() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_node(1);
        let v2 = graph.add_node(2);
        let v3 = graph.add_node(3);
        let e1 = graph.add_edge(v1.clone(), v2.clone(), ());
        let e2 = graph.add_edge(v2.clone(), v3.clone(), ());
        let mut path = Path::new(&graph, v1.clone());
        path.add_edge(e1);
        path.add_edge(e2);
        let mut iter = path.nodes_with_edges();
        assert_eq!(iter.next(), Some((None, v1.clone(), Some(e1.clone()))));
        assert_eq!(
            iter.next(),
            Some((Some(e1.clone()), v2.clone(), Some(e2.clone())))
        );
        assert_eq!(iter.next(), Some((Some(e2.clone()), v3.clone(), None)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_extend_with() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_node("v1");
        let v2 = graph.add_node("v2");
        let v3 = graph.add_node("v3");
        let e1 = graph.add_edge(v1, v2, "e12");
        let e2 = graph.add_edge(v2, v3, "e23");

        let mut path1 = Path::new(&graph, v1);
        path1.add_edge(e1);

        let mut path2 = Path::new(&graph, v2);
        path2.add_edge(e2);

        path1.extend_with(&path2);

        assert_eq!(path1.nodes().count(), 3);
        assert_eq!(path1.edges().count(), 2);
    }

    #[test]
    fn test_extend() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_node("v1");
        let v2 = graph.add_node("v2");
        let v3 = graph.add_node("v3");
        let e1 = graph.add_edge(v1, v2, "e12");
        let e2 = graph.add_edge(v2, v3, "e23");

        let mut path = Path::new(&graph, v1);
        path.extend(vec![e1, e2]);

        assert_eq!(path.nodes().count(), 3);
        assert_eq!(path.edges().count(), 2);
    }

    #[test]
    fn test_debug() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_node("v1");
        let v2 = graph.add_node("v2");
        let v3 = graph.add_node("v3");
        let e1 = graph.add_edge(v1, v2, "e12");
        let e2 = graph.add_edge(v2, v3, "e23");
        let e3 = graph.add_edge(v3, v1, "e31");

        let mut path = Path::new(&graph, v1);
        path.extend(vec![e1, e2, e3]);
        assert_eq!(
            format!("{:?}", path),
            r#"Path { nodes: {0: "v1", 1: "v2", 2: "v3", 3: "v1"}, edges: {0 -> 1: "e12", 1 -> 2: "e23", 2 -> 3: "e31"} }"#
        );
    }
}
