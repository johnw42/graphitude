use std::{fmt::Debug, marker::PhantomData};

use crate::{
    AdjacencyMatrix, Graph, GraphMut,
    adjacency_matrix::{AdjacencyMatrixSelector, HashStorage, SelectMatrix, Storage},
    debug::format_debug,
    directedness::Directedness,
    id_vec::{IdVec, IdVecIndex},
};

pub struct AdjacencyGraph<V, E, D, S = HashStorage>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
    vertices: IdVec<V>,
    adjacency: <(D::Symmetry, S) as AdjacencyMatrixSelector<IdVecIndex, E>>::Matrix,
    directedness: PhantomData<D>,
}

impl<V, E, D, S> AdjacencyGraph<V, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
    pub fn new() -> Self {
        Self {
            vertices: IdVec::new(),
            adjacency: SelectMatrix::<D::Symmetry, S, IdVecIndex, E>::new(),
            directedness: PhantomData,
        }
    }
}

impl<V, E, D, S> Graph for AdjacencyGraph<V, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
    type EdgeData = E;
    type EdgeId = (Self::VertexId, Self::VertexId);
    type VertexData = V;
    type VertexId = IdVecIndex;
    type Directedness = D;

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

    fn edges_between(
        &self,
        from: Self::VertexId,
        into: Self::VertexId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .edge_between(&from, &into)
            .into_iter()
            .map(|(from, into, _)| (from, into))
    }

    fn edges_into<'a>(&'a self, into: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        dbg!(
            self.adjacency
                .edges_from(&into)
                .map(|(from, _)| (into, from))
                .collect::<Vec<_>>()
        );
        self.adjacency
            .edges_into(&into)
            .map(|(from, _)| (from, into))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_from<'a>(&'a self, from: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_from(&from)
            .map(|(to, _)| (from, to))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<V, E, D, S> GraphMut for AdjacencyGraph<V, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
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
        for into in self
            .adjacency
            .edges_from(id)
            .map(|(to, _)| to)
            .collect::<Vec<_>>()
        {
            self.adjacency.remove(id, &into);
        }
        self.vertices.remove(*id)
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Option<Self::EdgeData> {
        self.adjacency.remove(&id.0, &id.1)
    }
}

impl<V, E, D, S> Debug for AdjacencyGraph<V, E, D, S>
where
    V: Debug,
    E: Debug,
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "AdjacencyGraph")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestDataBuilder;

    impl<D, S> TestDataBuilder for AdjacencyGraph<i32, String, D, S>
    where
        D: Directedness,
        S: Storage,
        (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, String>,
    {
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

    // mod directed_bitvec {
    //     use bitvec::vec::BitVec;

    //     use super::*;
    //     use crate::{directedness::Directed, graph_test_copy_from_with, graph_tests};

    //     graph_tests!(AdjacencyGraph<i32, String, Directed, BitVec>);
    //     graph_test_copy_from_with!(
    //     AdjacencyGraph<i32, String, Directed, BitVec>,
    //     |data| data * 2,
    //     |data: &String| format!("{}-copied", data));
    // }

    mod directed_hash {
        use super::*;
        use crate::{directedness::Directed, graph_test_copy_from_with, graph_tests};

        graph_tests!(AdjacencyGraph<i32, String, Directed, HashStorage>);
        graph_test_copy_from_with!(
        AdjacencyGraph<i32, String, Directed>,
        |data| data * 2,
        |data: &String| format!("{}-copied", data));
    }

    mod undirected_hash {
        use super::*;
        use crate::{directedness::Undirected, graph_test_copy_from_with, graph_tests};

        graph_tests!(AdjacencyGraph<i32, String, Undirected, HashStorage>);
        graph_test_copy_from_with!(
        AdjacencyGraph<i32, String, Undirected, HashStorage>,
        |data| data * 2,
        |data: &String| format!("{}-copied", data));
    }
}
