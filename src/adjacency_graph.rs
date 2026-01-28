#![cfg(feature = "bitvec")]
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use crate::{
    AdjacencyMatrix, Directed, Graph, GraphMut,
    adjacency_matrix::{AdjacencyMatrixSelector, HashStorage, SelectMatrix, Storage},
    debug::format_debug,
    directedness::Directedness,
    graph_id::GraphId,
    id_vec::{IdVec, IdVecIndex},
    util::sort_pair,
};

#[derive(Clone, Copy)]
pub struct NodeId {
    index: IdVecIndex,
    #[cfg(feature = "paranoia")]
    graph_id: GraphId,
}

impl Into<IdVecIndex> for NodeId {
    fn into(self) -> IdVecIndex {
        self.index
    }
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(feature = "paranoia")]
        assert_eq!(self.graph_id, other.graph_id);
        self.index == other.index
    }
}

impl Eq for NodeId {}

impl Hash for NodeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl Ord for NodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.index)
    }
}

#[derive(Clone, Copy)]
pub struct EdgeId {
    from: IdVecIndex,
    into: IdVecIndex,
    #[cfg(feature = "paranoia")]
    graph_id: GraphId,
}

impl PartialEq for EdgeId {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(feature = "paranoia")]
        assert_eq!(self.graph_id, other.graph_id);
        self.from == other.from && self.into == other.into
    }
}

impl Eq for EdgeId {}

impl Hash for EdgeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.into.hash(state);
    }
}

impl Into<(IdVecIndex, IdVecIndex)> for EdgeId {
    fn into(self) -> (IdVecIndex, IdVecIndex) {
        (self.from, self.into)
    }
}

impl<'a> Into<(&'a IdVecIndex, &'a IdVecIndex)> for &'a EdgeId {
    fn into(self) -> (&'a IdVecIndex, &'a IdVecIndex) {
        (&self.from, &self.into)
    }
}

impl Debug for EdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?}, {:?})", self.from, self.into)
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
    #[cfg(feature = "paranoia")]
    id: GraphId,
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
            #[cfg(feature = "paranoia")]
            id: GraphId::new(),
        }
    }

    fn node(&self, id: IdVecIndex) -> &N {
        &self.nodes[id]
    }

    fn node_id(&self, index: IdVecIndex) -> NodeId {
        NodeId {
            index,
            #[cfg(feature = "paranoia")]
            graph_id: self.id,
        }
    }

    fn edge_id(&self, from: IdVecIndex, into: IdVecIndex) -> EdgeId {
        let (i1, i2) = if self.is_directed() {
            (from, into)
        } else {
            sort_pair(from, into)
        };
        EdgeId {
            from: i1,
            into: i2,
            #[cfg(feature = "paranoia")]
            graph_id: self.id,
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
    type EdgeId = EdgeId;
    type NodeData = N;
    type NodeId = NodeId;
    type Directedness = D;

    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData {
        &self.nodes[id.into()]
    }

    fn node_ids(&self) -> impl Iterator<Item = <Self as Graph>::NodeId> {
        self.nodes.iter_indices().map(|index| self.node_id(index))
    }

    fn edge_data(&self, eid: Self::EdgeId) -> &Self::EdgeData {
        let (from, to) = eid.into();
        &self.adjacency.get(from, to).expect("no such edge")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .edges()
            .map(|(from, into, _)| self.edge_id(from, into))
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::NodeId, Self::NodeId) {
        (self.node_id(eid.from), self.node_id(eid.into))
    }

    fn edges_between(
        &self,
        from: Self::NodeId,
        into: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .edge_between(from.into(), into.into())
            .into_iter()
            .map(|(from, into, _)| self.edge_id(from, into))
    }

    fn edges_into<'a>(&'a self, into: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_into(into.into())
            .map(|(from, _)| self.edge_id(from, into.into()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_from<'a>(&'a self, from: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .edges_from(from.into())
            .map(|(into, _)| self.edge_id(from.into(), into))
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
        let index = self.nodes.insert(data);
        self.node_id(index)
    }

    fn add_or_replace_edge(
        &mut self,
        from: Self::NodeId,
        into: Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let old_data = self.adjacency.insert(from.into(), into.into(), data);
        (self.edge_id(from.into(), into.into()), old_data)
    }

    fn remove_node(&mut self, id: Self::NodeId) -> Self::NodeData {
        for into in self
            .adjacency
            .edges_from(id.into())
            .map(|(to, _)| to)
            .collect::<Vec<_>>()
        {
            self.adjacency.remove(id.into(), into);
        }
        self.nodes.remove(id.into())
    }

    fn remove_edge(&mut self, id: Self::EdgeId) -> Self::EdgeData {
        self.adjacency
            .remove(id.from, id.into)
            .expect("Invalid edge ID")
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
