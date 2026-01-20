use crate::{
    Graph,
    adjacency_matrix::AsymmetricAdjacencyMatrix,
    id_vec::{IdVec, IdVecIndex},
};

pub struct GraphImpl<V, E> {
    vertices: IdVec<V>,
    edges: IdVec<E>,
    adjacency: AsymmetricAdjacencyMatrix<IdVecIndex, E>,
}

impl<V, E> GraphImpl<V, E> {
    fn new() -> Self {
        Self {
            vertices: IdVec::new(),
            edges: IdVec::new(),
            adjacency: AsymmetricAdjacencyMatrix::new(),
        }
    }
}

impl<V, E> Graph for GraphImpl<V, E> {
    type EdgeData = E;
    type EdgeId = IdVecIndex;
    type VertexData = V;
    type VertexId = IdVecIndex;

    fn vertex_ids(&self) -> impl Iterator<Item = Self::VertexId> + '_ {
        todo!()
    }

    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData {
        todo!()
    }

    fn edge_data(&self, from: &Self::EdgeId) -> &Self::EdgeData {
        &self.edges[&from]
    }

    fn edge_source(&self, id: &Self::EdgeId) -> Self::VertexId {
        &self.adjacency.get(id)
    }

    fn edge_target(&self, id: &Self::EdgeId) -> Self::VertexId {
        todo!()
    }
}
