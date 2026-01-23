use crate::{
    Graph, GraphMut, adjacency_matrix::{AdjacencyMatrix, AsymmetricAdjacencyMatrix}, graph::Directed, id_vec::{IdVec, IdVecIndex}
};

pub struct AdjacencyGraph<V, E> {
    vertices: IdVec<V>,
    adjacency: AsymmetricAdjacencyMatrix<IdVecIndex, E>,
}

impl<V, E> AdjacencyGraph<V, E> {
    pub fn new() -> Self {
        Self {
            vertices: IdVec::new(),
            adjacency: AsymmetricAdjacencyMatrix::new(),
        }
    }
}

impl<V, E> Graph for AdjacencyGraph<V, E> {
    type EdgeData = E;
    type EdgeId = (Self::VertexId, Self::VertexId);
    type VertexData = V;
    type VertexId = IdVecIndex;
    type Directedness = Directed;

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

impl<V,E> GraphMut for AdjacencyGraph<V, E> {
    fn add_vertex(&mut self, data: Self::VertexData) -> Self::VertexId {
        self.vertices.insert(data)
    }

    fn add_or_replace_edge(
        &mut self,
        from: &Self::VertexId,
        into: &Self::VertexId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let old_data = self.adjacency.insert(from.clone(), into.clone(), data);
        ((from.clone(), into.clone()), old_data)
    }

    fn remove_vertex(&mut self, id: &Self::VertexId) -> Self::VertexData {
        for into in self.adjacency.edges_from(id).map(|(to, _)| to).collect::<Vec<_>>() {
            self.adjacency.remove(id, &into);
        }
        self.vertices.remove(*id)
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Option<Self::EdgeData> {
        self.adjacency.remove(&id.0, &id.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::{tests::TestDataBuilder, *};
    use super::*;

    impl TestDataBuilder for AdjacencyGraph<i32, String> {
        type Graph = Self;

        fn new_graph() -> Self::Graph {
            Self::new()
        }

        fn new_edge_data(i: usize) -> String {
            format!("e{}", i)
        }

        fn new_vertex_data(i: usize) -> i32 {
            i as i32
        }
    }

    graph_tests!(AdjacencyGraph<i32, String>);
    graph_test_copy_from_with!(
        AdjacencyGraph<i32, String>,
        |data| data * 2,
        |data: &String| format!("{}-copied", data));
}
