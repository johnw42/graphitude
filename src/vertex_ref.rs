use std::ptr;

use crate::{Graph, edge_ref::EdgeRef};

pub struct VertexRef<'g, G: Graph> {
    graph: &'g G,
    id: G::VertexId,
}

impl<'g, G> VertexRef<'g, G>
where
    G: Graph,
{
    pub(crate) fn new(graph: &'g G, id: G::VertexId) -> Self {
        Self { graph, id }
    }

    /// Gets a reference to the graph this vertex belongs to.
    pub fn graph(&self) -> &'g G {
        self.graph
    }

    /// Gets the identifier of this vertex.
    pub fn id(&self) -> G::VertexId {
        self.id.clone()
    }

    /// Gets the data associated with this vertex.
    pub fn data(&self) -> &'g G::VertexData {
        self.graph().vertex_data(self.id())
    }

    /// Gets an iterator over the edges outgoing from this vertex.
    pub fn edges_from(&self) -> impl Iterator<Item = EdgeRef<'g, G>> + 'g {
        let graph = self.graph();
        graph.edges_from(self.id()).map(move |eid| graph.edge(eid))
    }

    /// Gets an iterator over the edges incoming to this vertex.
    pub fn edges_into(&self) -> impl Iterator<Item = EdgeRef<'g, G>> + 'g {
        let graph = self.graph();
        graph.edges_into(self.id()).map(move |eid| graph.edge(eid))
    }

    /// Gets an iterator over the edges between this vertex and another vertex.
    pub fn edges_to(&self, from: &VertexRef<'g, G>) -> impl Iterator<Item = EdgeRef<'g, G>> + 'g {
        assert!(ptr::eq(self.graph(), from.graph()));
        let graph = self.graph();
        graph
            .edges_between(from.id(), self.id())
            .map(move |eid| graph.edge(eid))
    }

    /// Gets an iterator over the predacessors vertices of a given vertex, i.e.
    /// those vertices reachable by incoming edges.
    pub fn predacessors(&self) -> impl Iterator<Item = Self> {
        self.graph()
            .predacessors(self.id())
            .map(|id| self.graph().vertex(id))
    }

    /// Gets an iterator over the successor vertices of a given vertex, i.e.
    /// those vertices reachable by outgoing edges.
    pub fn successors(&self) -> impl Iterator<Item = Self> {
        self.graph()
            .successors(self.id())
            .map(|id| self.graph().vertex(id))
    }

    pub fn bfs(&self) -> impl Iterator<Item = Self> {
        self.graph()
            .bfs(self.id())
            .map(|id| self.graph().vertex(id))
    }

    pub fn dfs(&self) -> impl Iterator<Item = Self> {
        self.graph()
            .dfs(self.id())
            .map(|id| self.graph().vertex(id))
    }
}

impl<'g, G> Clone for VertexRef<'g, G>
where
    G: Graph,
{
    fn clone(&self) -> Self {
        Self {
            graph: self.graph,
            id: self.id.clone(),
        }
    }
}

impl<'g, G> std::fmt::Debug for VertexRef<'g, G>
where
    G: Graph,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VertexRef({:?})", self.id)
    }
}
