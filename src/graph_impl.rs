use crate::{
    Graph,
    adjacency_matrix::{AdjacencyMatrix, AsymmetricAdjacencyMatrix},
    id_vec::{IdVec, IdVecIndex},
};

pub struct GraphImpl<V, E> {
    vertices: IdVec<V>,
    adjacency: AsymmetricAdjacencyMatrix<IdVecIndex, E>,
}

impl<V, E> GraphImpl<V, E> {
    pub fn new() -> Self {
        Self {
            vertices: IdVec::new(),
            adjacency: AsymmetricAdjacencyMatrix::new(),
        }
    }
}

impl<V, E> Graph for GraphImpl<V, E> {
    type EdgeData = E;
    type EdgeId = (Self::VertexId, Self::VertexId);
    type VertexData = V;
    type VertexId = IdVecIndex;

    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData {
        &self.vertices[*id]
    }

    fn vertex_ids(&self) -> impl Iterator<Item = <Self as Graph>::VertexId> {
        self.vertices.iter_indices()
    }

    fn edge_data(&self, (from, to): &Self::EdgeId) -> &Self::EdgeData {
        &self.adjacency.get(from, to).expect("no such edge")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency.edges().map(|(from, to, _)| (from, to))
    }

    fn edge_ends(&self, (from, to): Self::EdgeId) -> (Self::VertexId, Self::VertexId) {
        (from, to)
    }

    fn edges_into<'a>(&'a self, into: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_into(&into)
            .map(move |(from, _)| (from, into))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_from<'a>(&'a self, from: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_from(&from)
            .map(move |(to, _)| (from, to))
            .collect::<Vec<_>>()
            .into_iter()
    }
}
