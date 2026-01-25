use crate::{Graph, vertex_ref::VertexRef};

pub struct EdgeRef<'g, G: Graph> {
    graph: &'g G,
    id: G::EdgeId,
}
impl<'g, G> EdgeRef<'g, G>
where
    G: Graph,
{
    pub(crate) fn new(graph: &'g G, id: G::EdgeId) -> Self {
        Self { graph, id }
    }

    pub fn graph(&self) -> &'g G {
        self.graph
    }

    pub fn data(&self) -> &'g G::EdgeData {
        self.graph.edge_data(self.id.clone())
    }

    pub fn source(&self) -> VertexRef<'g, G> {
        VertexRef::new(self.graph, self.graph.edge_source(self.id.clone()))
    }

    pub fn target(&self) -> VertexRef<'g, G> {
        VertexRef::new(self.graph, self.graph.edge_target(self.id.clone()))
    }
}
