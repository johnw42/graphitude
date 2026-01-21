use std::ptr;

use crate::{
    Graph,
    edge_ref::EdgeRef,
};

#[derive(Clone)]
pub struct VertexRef<'g, G: Graph + ?Sized> {
    graph: &'g G,
    id: G::VertexId,
}

impl<'g, G> VertexRef<'g, G>
where
    G: Graph + ?Sized,
{
    pub(crate) fn new(graph: &'g G, id: G::VertexId) -> Self {
        Self { graph, id }
    }

    fn graph(&self) -> &'g G {
        self.graph
    }

    fn id(&self) -> G::VertexId {
        self.id.clone()
    }

    fn data(&self) -> &'g G::VertexData {
        self.graph().vertex_data(&self.id())
    }

    fn edges_out(&self) -> impl Iterator<Item = EdgeRef<'g, G>> + 'g {
        let graph = self.graph();
        graph.edges_out(self.id()).map(move |eid| graph.edge(eid))
    }

    fn edges_in(&self) -> impl Iterator<Item = EdgeRef<'g, G>> + 'g {
        let graph = self.graph();
        graph.edges_in(self.id()).map(move |eid| graph.edge(eid))
    }

    fn edges_from(&self, from: &VertexRef<'g, G>) -> impl Iterator<Item = EdgeRef<'g, G>> + 'g {
        assert!(ptr::eq(self.graph(), from.graph()));
        let graph = self.graph();
        graph
            .edges_between(from.id(), self.id())
            .map(move |eid| graph.edge(eid))
    }

    fn edges_into(&self, into: &VertexRef<'g, G>) -> impl Iterator<Item = EdgeRef<'g, G>> + 'g {
        assert!(ptr::eq(self.graph(), into.graph()));
        let graph = self.graph();
        graph
            .edges_between(self.id(), into.id())
            .map(move |eid| graph.edge(eid))
    }

    /// Gets an iterator over the predacessors vertices of a given vertex, i.e.
    /// those vertices reachable by incoming edges.
    fn predacessors(&self) -> impl Iterator<Item = Self> {
        self.graph()
            .predacessors(self.id())
            .map(|id| self.graph().vertex(id))
    }

    /// Gets an iterator over the successor vertices of a given vertex, i.e.
    /// those vertices reachable by outgoing edges.
    fn successors(&self) -> impl Iterator<Item = Self> {
        self.graph()
            .successors(self.id())
            .map(|id| self.graph().vertex(id))
    }

    fn bfs(&self) -> impl Iterator<Item = Self> {
        self.graph()
            .bfs(self.id())
            .map(|id| self.graph().vertex(id))
    }

    fn dfs(&self) -> impl Iterator<Item = Self> {
        self.graph()
            .dfs(self.id())
            .map(|id| self.graph().vertex(id))
    }
}
