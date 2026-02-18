use std::{cmp::Ordering, fmt::Debug, hash::Hash, iter::once};

use derivative::Derivative;

use crate::{DirectednessTrait, EdgeIdTrait};

/// A path in a graph, represented as a sequence of nodes and the edges that
/// connect them.
#[derive(Derivative)]
#[derivative(
    Clone(bound = "E: Clone, E::NodeId: Clone"),
    PartialEq(bound = "E: PartialEq, E::NodeId: PartialEq"),
    Eq(bound = "E: Eq, E::NodeId: Eq"),
    Hash(bound = "E: Hash, E::NodeId: Hash"),
    Debug(bound = "E: Debug, E::NodeId: Debug")
)]
pub struct Path<E: EdgeIdTrait> {
    edges: Vec<E>,
    nodes: Vec<E::NodeId>,
}

impl<E: EdgeIdTrait> Path<E> {
    /// Creates a new path starting at the given node.
    pub fn new(start: E::NodeId) -> Self {
        Self {
            edges: Vec::new(),
            nodes: vec![start],
        }
    }

    /// Creates a new path from a sequence of edges and a starting node.  Panics
    /// if the edges do not form a valid path, the edges are empty, or if the
    /// starting node does not match the one end of the first edge.
    pub fn from_edges(start: E::NodeId, edges: impl IntoIterator<Item = E>) -> Self {
        let mut path = Path::<E>::new(start);
        path.extend(edges);
        path
    }

    /// Returns the first node in the path.
    pub fn first_node(&self) -> E::NodeId {
        self.nodes.first().expect("Path has no nodes").clone()
    }

    /// Returns the last node in the path.
    pub fn last_node(&self) -> E::NodeId {
        self.nodes.last().expect("Path has no nodes").clone()
    }

    /// Returns an iterator over the edges in the path.
    pub fn edges(&self) -> impl Iterator<Item = E> + '_ {
        self.edges.iter().cloned()
    }

    /// Returns an iterator over the nodes in the path.
    pub fn nodes(&self) -> impl Iterator<Item = E::NodeId> + '_ {
        self.nodes.iter().cloned()
    }

    /// Returns an iterator over the nodes in the path along with the edges
    /// connecting them. Each item is a tuple of the for `(incoming_edge,
    /// node, outgoing_edge)`; the edges are optional but will always be
    /// `Some` except for the first node (no incoming edge) and the last
    /// node (no outgoing edge).
    pub fn nodes_with_edges(&self) -> impl Iterator<Item = (Option<E>, E::NodeId, Option<E>)> + '_ {
        let incoming = once(None).chain(self.edges.iter().cloned().map(Some));
        let outgoing = self.edges.iter().cloned().map(Some).chain(once(None));
        let nodes = self.nodes.iter().cloned();
        incoming
            .zip(outgoing)
            .zip(nodes)
            .map(|((in_edge, out_edge), node)| {
                if let Some(ref e) = in_edge {
                    debug_assert!(e.has_end(&node));
                }
                if let Some(ref e) = out_edge {
                    debug_assert!(e.has_end(&node));
                }
                (in_edge, node, out_edge)
            })
    }

    /// Adds an edge to the end of the path, extending it to the edge's target
    /// node. Panics if the edge's source node does not match the current
    /// last node of the path.
    pub fn add_edge(&mut self, edge_id: E) {
        let next_node = edge_id.other_end(&self.last_node());
        self.edges.push(edge_id);
        self.nodes.push(next_node);
    }

    /// Adds an edge and its target node to the end of the path. Panics if
    /// the edge's source node does not match the current last node of the
    /// path, or if the provided node does not match the edge's target node.
    pub fn add_edge_and_node(&mut self, edge_id: E, node_id: E::NodeId) {
        let last = self.last_node();

        if edge_id.directedness().is_directed() {
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

    /// Extends the path by appending all edges from another path. Panics if
    /// the first node of the other path does not match the current last
    /// node of this path.
    pub fn extend_with(&mut self, other: &Path<E>) {
        for edge_id in other.edges() {
            self.add_edge(edge_id);
        }
    }
}

impl<E: EdgeIdTrait> PartialOrd for Path<E> {
    /// A path is "less than" another path if its last node matches the
    /// other's first node, and "greater than" if its first node matches the
    /// other's last node. If neither condition is met and the paths are not
    /// equal, they are considered unordered.
    fn partial_cmp(&self, other: &Path<E>) -> Option<std::cmp::Ordering> {
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

impl<E: EdgeIdTrait> Extend<E> for Path<E> {
    fn extend<T: IntoIterator<Item = E>>(&mut self, iter: T) {
        for edge_id in iter {
            self.add_edge(edge_id);
        }
    }
}

#[cfg(test)]
#[cfg(feature = "bitvec")]
mod tests {
    macro_rules! tests {
        ($mod: ident,$type: ty) => {
            mod $mod {
                use crate::adjacency_graph::AdjacencyGraph;
                use $crate::prelude::*;

                #[test]
                fn test_new_path() {
                    let mut graph = <$type>::default();
                    let n1 = graph.add_node("n1");
                    let path = graph.new_path(&n1);
                    assert_eq!(path.first_node(), n1);
                    assert_eq!(path.last_node(), n1);
                }

                #[test]
                fn test_add_edge() {
                    let mut graph = <$type>::default();
                    let n1 = graph.add_node("n1");
                    let n2 = graph.add_node("n2");
                    let e1 = graph.add_edge(&n1, &n2, "e1").unwrap();

                    let mut path = graph.new_path(&n1);
                    path.add_edge(e1);

                    assert_eq!(path.last_node(), n2);
                    assert_eq!(path.edges().count(), 1);
                    assert_eq!(path.nodes().count(), 2);
                }

                #[test]
                fn test_nodes_with_edges() {
                    let mut graph = <$type>::default();
                    let n1 = graph.add_node("n1");
                    let n2 = graph.add_node("n2");
                    let n3 = graph.add_node("n3");
                    let e1 = graph.add_edge(&n1, &n2, "e1").unwrap();
                    let e2 = graph.add_edge(&n2, &n3, "e2").unwrap();
                    let mut path = graph.new_path(&n1);
                    path.add_edge(e1.clone());
                    path.add_edge(e2.clone());
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
                    let mut graph = <$type>::default();
                    let n1 = graph.add_node("n1");
                    let n2 = graph.add_node("n2");
                    let n3 = graph.add_node("n3");
                    let e1 = graph.add_edge(&n1, &n2, "e12").unwrap();
                    let e2 = graph.add_edge(&n2, &n3, "e23").unwrap();

                    let mut path1 = graph.new_path(&n1);
                    path1.add_edge(e1);

                    let mut path2 = graph.new_path(&n2);
                    path2.add_edge(e2);

                    path1.extend_with(&path2);

                    assert_eq!(path1.nodes().count(), 3);
                    assert_eq!(path1.edges().count(), 2);
                }

                #[test]
                fn test_extend() {
                    let mut graph = <$type>::default();
                    let n1 = graph.add_node("n1");
                    let n2 = graph.add_node("n2");
                    let n3 = graph.add_node("n3");
                    let e1 = graph.add_edge(&n1, &n2, "e12").unwrap();
                    let e2 = graph.add_edge(&n2, &n3, "e23").unwrap();

                    let mut path = graph.new_path(&n1);
                    path.extend(vec![e1, e2]);

                    assert_eq!(path.nodes().count(), 3);
                    assert_eq!(path.edges().count(), 2);
                }

                #[test]
                fn test_debug() {
                    let mut graph = <$type>::default();
                    let n1 = graph.add_node("n1");
                    let n2 = graph.add_node("n2");
                    let n3 = graph.add_node("n3");
                    let e1 = graph.add_edge(&n1, &n2, "e12").unwrap();
                    let e2 = graph.add_edge(&n2, &n3, "e23").unwrap();
                    let e3 = graph.add_edge(&n3, &n1, "e31").unwrap();

                    let mut path = graph.new_path(&n1);
                    path.extend(vec![e1, e2, e3]);
                    let debug_str = format!("{:?}", path);
                    assert!(debug_str.contains("Path"));
                    assert!(debug_str.contains("nodes"));
                    assert!(debug_str.contains("edges"));
                }
            }
        };
    }

    tests!(
        directed,
        AdjacencyGraph<&'static str, &'static str, Directed>
    );
    tests!(
        undirected,
        AdjacencyGraph<&'static str, &'static str, Undirected>
    );
}
