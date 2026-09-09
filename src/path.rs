use std::iter::once;

use derive_where::derive_where;

use crate::{Graph, end_pair::EndPair as _};

#[derive_where(Clone, PartialEq, Eq, Hash, Debug)]
pub struct PathBuilder<'g, G: Graph + ?Sized> {
    #[derive_where(skip)]
    graph: &'g G,
    edges: Vec<G::EdgeId>,
    nodes: Vec<G::NodeId>,
}

impl<'g, G: Graph + ?Sized> PathBuilder<'g, G> {
    pub fn new(graph: &'g G, start: G::NodeId) -> Self {
        Self {
            graph,
            edges: Vec::new(),
            nodes: vec![start],
        }
    }

    pub fn push(&mut self, edge_id: G::EdgeId) {
        let next_node = self
            .graph
            .edge_ends(&edge_id)
            .into_other_value(
                self.nodes
                    .last()
                    .expect("Path always has at least one node"),
            )
            .into_inner();
        self.edges.push(edge_id);
        self.nodes.push(next_node);
    }

    /// Returns the last node in the path.
    pub fn last_node(&self) -> G::NodeId {
        self.nodes
            .last()
            .expect("Path always has at least one node")
            .clone()
    }

    /// Builds a path from the current state of the builder.
    pub fn build(self) -> Path<G> {
        Path {
            edges: self.edges,
            nodes: self.nodes,
        }
    }
}

/// A path in a graph, represented as a sequence of nodes and the edges that
/// connect them.
#[derive(Clone)]
#[derive_where(PartialEq, Eq, Hash, Debug; G::NodeId, G::EdgeId)]
pub struct Path<G: Graph + ?Sized> {
    edges: Vec<G::EdgeId>,
    nodes: Vec<G::NodeId>,
}

impl<G: Graph + ?Sized> Path<G> {
    /// Creates a new path from a sequence of edges and a starting node. Panics
    /// if the edges do not form a valid path, the edges are empty, or if the
    /// starting node does not match the one end of the first edge.
    pub fn from_edges<'g>(
        graph: &'g G,
        start: &'g G::NodeId,
        edges: impl IntoIterator<Item = &'g G::EdgeId>,
    ) -> Self {
        let mut path = PathBuilder::new(graph, start.clone());
        for edge in edges {
            path.push(edge.clone());
        }
        path.build()
    }

    /// Returns the first node in the path.
    pub fn first_node(&self) -> G::NodeId {
        self.nodes
            .first()
            .expect("Path always has at least one node")
            .clone()
    }

    /// Returns the last node in the path.
    pub fn last_node(&self) -> G::NodeId {
        self.nodes
            .last()
            .expect("Path always has at least one node")
            .clone()
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
            .map(|((in_edge, out_edge), node)| (in_edge, node, out_edge))
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
                fn test_add_edge() {
                    let mut graph = <$type>::default();
                    let n1 = graph.add_node("n1");
                    let n2 = graph.add_node("n2");
                    let e1 = graph.add_edge(&n1, &n2, "e1").0;

                    let mut builder = PathBuilder::new(&graph, n1);
                    builder.push(e1);
                    let path = builder.build();

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
                    let e1 = graph.add_edge(&n1, &n2, "e1").0;
                    let e2 = graph.add_edge(&n2, &n3, "e2").0;
                    let mut builder = PathBuilder::new(&graph, n1.clone());
                    builder.push(e1.clone());
                    builder.push(e2.clone());
                    let path = builder.build();
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
                fn test_debug() {
                    let mut graph = <$type>::default();
                    let n1 = graph.add_node("n1");
                    let n2 = graph.add_node("n2");
                    let n3 = graph.add_node("n3");
                    let e1 = graph.add_edge(&n1, &n2, "e12").0;
                    let e2 = graph.add_edge(&n2, &n3, "e23").0;
                    let e3 = graph.add_edge(&n3, &n1, "e31").0;

                    let mut builder = PathBuilder::new(&graph, n1.clone());
                    builder.push(e1);
                    builder.push(e2);
                    builder.push(e3);
                    let path = builder.build();
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
