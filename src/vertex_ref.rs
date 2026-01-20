use std::ptr;

use crate::{Graph, edge_ref::EdgeRef};

pub trait VertexRefCore<'g, G: Graph + ?Sized> 
where
    G: Graph + ?Sized,
{
    // Required Methods

     fn graph(&self) -> &'g G;
     fn id(&self) -> G::VertexId;

     fn data(&self) -> &G::VertexData {
        self.graph().vertex_data(&self.id())
    }

    // Provided Methods

     fn edges_out(&self) -> impl Iterator<Item = EdgeRef<'g, G>> {
        self.graph()
            .edges_out(&self.id())
            .map(|eid| self.graph().edge(&eid))
    }

     fn edges_in(&self) -> impl Iterator<Item = EdgeRef<'g, G>> {
        self.graph()
            .edges_in(&self.id())
            .map(|eid| self.graph().edge(&eid))
    }

     fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b VertexRef<'g, G>,
    ) -> impl Iterator<Item = EdgeRef<'g, G>> {
        assert!(ptr::eq(self.graph, from.graph));
        self.graph()
            .edges_between(&from.id(), &self.id())
            .map(|eid| self.graph().edge(&eid))
    }

     fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b VertexRef<'g, G>,
    ) -> impl Iterator<Item = EdgeRef<'g, G>> {
        assert!(ptr::eq(self.graph, into.graph));
        self.graph()
            .edges_between(&self.id(), &into.id())
            .map(|eid| self.graph().edge(&eid))
    }

    /// Gets an iterator over the predacessors vertices of a given vertex, i.e.
    /// those vertices reachable by incoming edges.
    fn predacessors(&self) -> impl Iterator<Item = Self> + '_ {
        self.graph()
            .predacessors(&self.id())
            .map(|id| self.graph().vertex_ref(&id))
    }

    /// Gets an iterator over the successor vertices of a given vertex, i.e.
    /// those vertices reachable by outgoing edges.
    fn successors(&self) -> impl Iterator<Item = Self> + '_ {
        self.graph()
            .successors(&self.id())
            .map(|id| self.graph().vertex_ref(&id))
    }

    fn bfs(&self) -> impl Iterator<Item = Self> + '_ {
        self.graph().bfs(&self.id()).map(|id| self.graph().vertex_ref(&id))
    }

    fn dfs(&self) -> impl Iterator<Item = Self> + '_ {
        self.graph().dfs(&self.id()).map(|id| self.graph().vertex_ref(&id))
    }
}

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
}

impl<'g, G> VertexRefCore<'g, G> for VertexRef<'g, G>
where
    G: Graph + ?Sized,
{  
    fn graph(&self) -> &'g G {
        self.graph
    }
    
    fn id(&self) -> G::VertexId {
        self.id.clone()
    }
}

pub struct VertexRefMut<'g, G: Graph + GraphMut + ?Sized> {
    pub graph: &'g mut G,
    pub id: G::VertexId,
}

impl<'g, G> VertexRefMut<'g, G>
where
    G: Graph + GraphMut + ?Sized,
{
    pub(crate) fn new(graph: &'g mut G, id: G::VertexId) -> Self {
        Self { graph, id }
    }

    pub fn graph_mut(self) -> &'g mut G {
        self.graph
    }

    pub fn data_mut(&mut self) -> &mut G::VertexData {
        self.graph.vertex_data_mut(&self.id)
    }
}

impl<'g, G> VertexRefCore<'g, G> for VertexRefMut<'g, G>
where
    G: Graph + GraphMut + ?Sized,
{
    fn graph(&self) -> &'g G {
        self.graph
    }
    
    fn id(&self) -> G::VertexId {
        self.id.clone()
    }
}

