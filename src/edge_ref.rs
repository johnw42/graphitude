use crate::{Graph, vertex_ref::VertexRef};

pub trait EdgeRefCore<'g, G: Graph + ?Sized> {
    fn graph(&self) -> &'g G;
    // fn data(&self) -> &'g G::EdgeData;
    // fn source(&self) -> VertexRef<'g, G>;
    // fn target(&self) -> VertexRef<'g, G>;
}

pub struct EdgeRef<'g, G: Graph + ?Sized> {
    graph: &'g G,
    id: G::EdgeId,
}
impl<'g, G> EdgeRef<'g, G>
where
    G: Graph + ?Sized,
{
    pub(crate) fn new(graph: &'g G, id: G::EdgeId) -> Self {
        Self { graph, id }
    }

    pub fn data(&self) -> &'g G::EdgeData {
        self.graph.edge_data(&self.id)
    }

    pub fn source(&self) -> VertexRef<'g, G> {
        VertexRef::new(self.graph, self.graph.edge_source(self.id.clone()))
    }

    pub fn target(&self) -> VertexRef<'g, G> {
        VertexRef::new(self.graph, self.graph.edge_target(self.id.clone()))
    }
}
