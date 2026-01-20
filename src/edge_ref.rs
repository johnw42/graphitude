use crate::{
    Graph, GraphMut,
    vertex_ref::{VertexMut, VertexRef},
};

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
        VertexRef::new(self.graph, self.graph.edge_source(&self.id))
    }

    pub fn target(&self) -> VertexRef<'g, G> {
        VertexRef::new(self.graph, self.graph.edge_target(&self.id))
    }
}

pub struct EdgeMut<'g, G: Graph + GraphMut + ?Sized> {
    pub graph: &'g mut G,
    pub id: G::EdgeId,
}

impl<'g, G> From<EdgeMut<'g, G>> for EdgeRef<'g, G>
where
    G: GraphMut + ?Sized,
{
    fn from(edge_mut_ref: EdgeMut<'g, G>) -> Self {
        EdgeRef::new(edge_mut_ref.graph, edge_mut_ref.id)
    }
}

impl<'g, G> EdgeMut<'g, G>
where
    G: Graph + GraphMut + ?Sized,
{
    pub(crate) fn new(graph: &'g mut G, id: G::EdgeId) -> Self {
        Self { graph, id }
    }

    fn graph(self) -> &'g G {
        self.graph
    }

    fn graph_mut(self) -> &'g mut G {
        self.graph
    }

    fn data(self) -> &'g G::EdgeData {
        self.graph.edge_data(&self.id)
    }

    fn data_mut(self) -> &'g mut G::EdgeData {
        self.graph.edge_data_mut(&self.id)
    }

    fn source(self) -> VertexRef<'g, G> {
        VertexRef::new(self.graph, self.graph.edge_source(&self.id))
    }

    fn source_mut(self) -> VertexMut<'g, G> {
        VertexMut::new(self.graph, self.graph.edge_source(&self.id))
    }

    fn target(self) -> VertexRef<'g, G> {
        VertexRef::new(self.graph, self.graph.edge_target(&self.id))
    }

    fn target_mut(self) -> VertexMut<'g, G> {
        VertexMut::new(self.graph, self.graph.edge_target(&self.id))
    }
}
