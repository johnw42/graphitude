use std::fmt::Debug;

use crate::{Directed, Graph, GraphMut, LinkedGraph};

/// A view of a graph with transformed node and edge data, suitable for debugging.
///
/// This type creates a snapshot of a graph with transformed data that can be
/// used for debug formatting. The transformation is applied once during construction,
/// and the result is stored in an internal `LinkedGraph`.
///
/// Note: This view always represents a directed graph, regardless of the source graph's
/// directedness. Undirected edges from the source will be represented as pairs of
/// directed edges in the view.
pub struct DebugGraphView<N, E> {
    inner: LinkedGraph<N, E>,
}

impl<N, E> DebugGraphView<N, E>
where
    N: Debug,
    E: Debug,
{
    /// Creates a new `DebugGraphView` by transforming the data from the source graph.
    pub fn new<G, NF, EF>(graph: &G, node_fn: NF, edge_fn: EF) -> Self
    where
        G: Graph,
        NF: Fn(&G::NodeData) -> N,
        EF: Fn(&G::EdgeData) -> E,
    {
        let mut inner = LinkedGraph::new();
        inner.copy_from_with(graph, node_fn, edge_fn);
        Self { inner }
    }
}

impl<N, E> Graph for DebugGraphView<N, E>
where
    N: Debug,
    E: Debug,
{
    type Directedness = Directed;
    type NodeData = N;
    type NodeId = <LinkedGraph<N, E> as Graph>::NodeId;
    type EdgeData = E;
    type EdgeId = <LinkedGraph<N, E> as Graph>::EdgeId;

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.inner.node_ids()
    }

    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData {
        self.inner.node_data(id)
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.inner.edge_ids()
    }

    fn edge_data(&self, id: Self::EdgeId) -> &Self::EdgeData {
        self.inner.edge_data(id)
    }

    fn edges_between(
        &self,
        from: Self::NodeId,
        to: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.inner.edges_between(from, to)
    }
}

impl<N, E> Debug for DebugGraphView<N, E>
where
    N: Debug,
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}
