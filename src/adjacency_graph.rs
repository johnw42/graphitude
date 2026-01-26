#![cfg(feature = "bitvec")]
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use crate::{
    AdjacencyMatrix, Graph, GraphMut,
    adjacency_matrix::{AdjacencyMatrixSelector, HashStorage, SelectMatrix, Storage},
    debug::format_debug,
    directedness::Directedness, id_vec::{IdVec, IdVecIndex},
};

pub struct EdgeId<V, D>(V, V, PhantomData<D>);

impl<V, D> EdgeId<V, D>
where
    D: Directedness,
    V: Ord,
{
    pub fn new(from: V, into: V) -> Self {
        let (v1, v2) = D::maybe_sort(from, into);
        EdgeId(v1, v2, PhantomData)
    }
}

impl<V, D> Clone for EdgeId<V, D>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        EdgeId(self.0.clone(), self.1.clone(), PhantomData)
    }
}

impl<V, D> PartialEq for EdgeId<V, D>
where
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<V, D> Eq for EdgeId<V, D> where V: Eq {}

impl<V, D> Hash for EdgeId<V, D>
where
    V: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<V, D> Into<(V, V)> for EdgeId<V, D> {
    fn into(self) -> (V, V) {
        (self.0, self.1)
    }
}

impl<'a, V, D> Into<(&'a V, &'a V)> for &'a EdgeId<V, D> {
    fn into(self) -> (&'a V, &'a V) {
        (&self.0, &self.1)
    }
}

impl<V, D> Debug for EdgeId<V, D>
where
    V: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?}, {:?})", self.0, self.1)
    }
}

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
    type EdgeId = EdgeId<Self::VertexId, Self::Directedness>;
    type VertexData = V;
    type VertexId = IdVecIndex;
    type Directedness = D;

    fn vertex_data(&self, id: Self::VertexId) -> &Self::VertexData {
        &self.vertices[id]
    }

    fn vertex_ids(&self) -> impl Iterator<Item = <Self as Graph>::VertexId> {
        self.vertices.iter_indices()
    }

    fn edge_data(&self, eid: Self::EdgeId) -> &Self::EdgeData {
        let (from, to) = eid.into();
        &self.adjacency.get(from, to).expect("no such edge")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .edges()
            .map(|(from, to, _)| EdgeId::<Self::VertexId, Self::Directedness>::new(from, to))
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::VertexId, Self::VertexId) {
        eid.into()
    }

    fn edges_between(
        &self,
        from: Self::VertexId,
        into: Self::VertexId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .edge_between(from, into)
            .into_iter()
            .map(|(from, into, _)| Self::EdgeId::new(from, into))
    }

    fn edges_into<'a>(&'a self, into: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_into(into)
            .map(|(from, _)| Self::EdgeId::new(from, into))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_from<'a>(&'a self, from: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_from(from)
            .map(|(to, _)| Self::EdgeId::new(from, to))
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
        from: Self::VertexId,
        into: Self::VertexId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let old_data = self.adjacency.insert(from.clone(), into.clone(), data);
        (Self::EdgeId::new(from, into), old_data)
    }

    fn remove_vertex(&mut self, id: Self::VertexId) -> Self::VertexData {
        for into in self
            .adjacency
            .edges_from(id)
            .map(|(to, _)| to)
            .collect::<Vec<_>>()
        {
            self.adjacency.remove(id, into);
        }
        self.vertices.remove(id)
    }

    fn remove_edge(&mut self, id: Self::EdgeId) -> Option<Self::EdgeData> {
        self.adjacency.remove(id.0, id.1)
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

    mod directed_bitvec {
        use super::*;
        use crate::{
            adjacency_matrix::BitvecStorage, directedness::Directed, graph_test_copy_from_with,
            graph_tests,
        };

        graph_tests!(AdjacencyGraph<i32, String, Directed, BitvecStorage>);
        graph_test_copy_from_with!(
        AdjacencyGraph<i32, String, Directed, BitvecStorage>,
        |data| data * 2,
        |data: &String| format!("{}-copied", data));
    }

    mod undirected_bitvec {
        use super::*;
        use crate::{
            adjacency_matrix::BitvecStorage, directedness::Undirected, graph_test_copy_from_with,
            graph_tests,
        };

        graph_tests!(AdjacencyGraph<i32, String, Undirected, BitvecStorage>);
        graph_test_copy_from_with!(
        AdjacencyGraph<i32, String, Undirected, BitvecStorage>,
        |data| data * 2,
        |data: &String| format!("{}-copied", data));
    }

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
