#![cfg(feature = "bitvec")]
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use crate::{
    AdjacencyMatrix, Directed, Graph, GraphMut,
    adjacency_matrix::{AdjacencyMatrixSelector, HashStorage, SelectMatrix, Storage},
    debug::format_debug,
    directedness::Directedness,
    id_vec::{IdVec, IdVecIndex},
};

pub struct EdgeId<N, D>(N, N, PhantomData<D>);

impl<N, D> EdgeId<N, D>
where
    D: Directedness,
    N: Ord,
{
    pub fn new(from: N, into: N) -> Self {
        let (n1, n2) = D::maybe_sort(from, into);
        EdgeId(n1, n2, PhantomData)
    }
}

impl<N, D> Clone for EdgeId<N, D>
where
    N: Copy,
{
    fn clone(&self) -> Self {
        EdgeId(self.0, self.1, PhantomData)
    }
}

impl<N, D> Copy for EdgeId<N, D> where N: Copy {}

impl<N, D> PartialEq for EdgeId<N, D>
where
    N: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }
}

impl<N, D> Eq for EdgeId<N, D> where N: Eq {}

impl<N, D> Hash for EdgeId<N, D>
where
    N: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

impl<N, D> Into<(N, N)> for EdgeId<N, D> {
    fn into(self) -> (N, N) {
        (self.0, self.1)
    }
}

impl<'a, N, D> Into<(&'a N, &'a N)> for &'a EdgeId<N, D> {
    fn into(self) -> (&'a N, &'a N) {
        (&self.0, &self.1)
    }
}

impl<N, D> Debug for EdgeId<N, D>
where
    N: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?}, {:?})", self.0, self.1)
    }
}

pub struct AdjacencyGraph<N, E, D = Directed, S = HashStorage>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
    nodes: IdVec<N>,
    adjacency: <(D::Symmetry, S) as AdjacencyMatrixSelector<IdVecIndex, E>>::Matrix,
    directedness: PhantomData<D>,
}

impl<N, E, D, S> AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
    pub fn new() -> Self {
        Self {
            nodes: IdVec::new(),
            adjacency: SelectMatrix::<D::Symmetry, S, IdVecIndex, E>::new(),
            directedness: PhantomData,
        }
    }
}

impl<N, E, D, S> Graph for AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
    type EdgeData = E;
    type EdgeId = EdgeId<Self::NodeId, Self::Directedness>;
    type NodeData = N;
    type NodeId = IdVecIndex;
    type Directedness = D;

    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData {
        &self.nodes[id]
    }

    fn node_ids(&self) -> impl Iterator<Item = <Self as Graph>::NodeId> {
        self.nodes.iter_indices()
    }

    fn edge_data(&self, eid: Self::EdgeId) -> &Self::EdgeData {
        let (from, to) = eid.into();
        &self.adjacency.get(from, to).expect("no such edge")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .edges()
            .map(|(from, to, _)| EdgeId::<Self::NodeId, Self::Directedness>::new(from, to))
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::NodeId, Self::NodeId) {
        eid.into()
    }

    fn edges_between(
        &self,
        from: Self::NodeId,
        into: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .edge_between(from, into)
            .into_iter()
            .map(|(from, into, _)| Self::EdgeId::new(from, into))
    }

    fn edges_into<'a>(&'a self, into: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_into(into)
            .map(|(from, _)| Self::EdgeId::new(from, into))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_from<'a>(&'a self, from: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_from(from)
            .map(|(to, _)| Self::EdgeId::new(from, to))
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<N, E, D, S> GraphMut for AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecIndex, E>,
{
    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        self.nodes.insert(data)
    }

    fn add_or_replace_edge(
        &mut self,
        from: Self::NodeId,
        into: Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let old_data = self.adjacency.insert(from.clone(), into.clone(), data);
        (Self::EdgeId::new(from, into), old_data)
    }

    fn remove_node(&mut self, id: Self::NodeId) -> Self::NodeData {
        for into in self
            .adjacency
            .edges_from(id)
            .map(|(to, _)| to)
            .collect::<Vec<_>>()
        {
            self.adjacency.remove(id, into);
        }
        self.nodes.remove(id)
    }

    fn remove_edge(&mut self, id: Self::EdgeId) -> Self::EdgeData {
        self.adjacency.remove(id.0, id.1).expect("Invalid edge ID")
    }
}

impl<N, E, D, S> Debug for AdjacencyGraph<N, E, D, S>
where
    N: Debug,
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

        fn new_node_data(i: usize) -> i32 {
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
