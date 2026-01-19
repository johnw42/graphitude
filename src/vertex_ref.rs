#![cfg(feature = "nope")]
use crate::{Graph, GraphMut, edge_ref::EdgeRef};

pub struct VertexRef<'g, G: Graph + ?Sized> {
    pub graph: &'g G,
    pub id: G::VertexId,
}

impl<'g, G> VertexRef<'g, G>
where
    G: Graph + ?Sized,
{
    pub(crate) fn new(graph: &'g G, id: G::VertexId) -> Self {
        Self { graph, id }
    }

    pub fn graph(&self) -> &'g G {
        self.graph
    }

    pub fn data(&self) -> &G::VertexData {
        self.graph.vertex_data(&self.id)
    }

    pub fn edges_out(&self) -> impl IntoIterator<Item = EdgeRef<'g, G>> {
        self.graph
            .edges_out(&self.id)
            .into_iter()
            .map(|eid| EdgeRef::new(self.graph, eid))
    }

    pub fn edges_in(&self) -> impl IntoIterator<Item = EdgeRef<'g, G>> {
        self.graph
            .edges_in(&self.id)
            .into_iter()
            .map(|eid| EdgeRef::new(self.graph, eid))
    }
}

pub struct VertexMut<'g, G: Graph + GraphMut + ?Sized> {
    pub graph: &'g mut G,
    pub id: G::VertexId,
}

impl<'g, G> VertexMut<'g, G>
where
    G: Graph + GraphMut + ?Sized,
{
    pub(crate) fn new(graph: &'g mut G, id: G::VertexId) -> Self {
        Self { graph, id }
    }

    pub fn graph(self) -> &'g G {
        self.graph
    }

    pub fn graph_mut(self) -> &'g mut G {
        self.graph
    }

    pub fn data(&mut self) -> &G::VertexData {
        self.graph.vertex_data(&self.id)
    }

    pub fn data_mut(&mut self) -> &mut G::VertexData {
        self.graph.vertex_data_mut(&self.id)
    }
}
