#![cfg(feature = "bitvec")]
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use crate::{
    AdjacencyMatrix, Directed, Graph, GraphMut,
    adjacency_matrix::{AdjacencyMatrixSelector, HashStorage, SelectMatrix, Storage},
    debug::format_debug,
    directedness::Directedness,
    graph_id::GraphId,
    id_vec::{IdVec, IdVecKey},
    util::maybe_sort_pair,
};

#[derive(Clone, Copy)]
pub struct NodeId {
    index: IdVecKey,
    #[cfg(not(feature = "unchecked"))]
    graph_id: GraphId,
}

impl Into<IdVecKey> for NodeId {
    fn into(self) -> IdVecKey {
        self.index
    }
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(not(feature = "unchecked"))]
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
    from: IdVecKey,
    into: IdVecKey,
    #[cfg(not(feature = "unchecked"))]
    graph_id: GraphId,
}

impl PartialEq for EdgeId {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(not(feature = "unchecked"))]
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

impl Into<(IdVecKey, IdVecKey)> for EdgeId {
    fn into(self) -> (IdVecKey, IdVecKey) {
        (self.from, self.into)
    }
}

impl<'a> Into<(&'a IdVecKey, &'a IdVecKey)> for &'a EdgeId {
    fn into(self) -> (&'a IdVecKey, &'a IdVecKey) {
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
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, E>,
{
    nodes: IdVec<N>,
    adjacency: <(D::Symmetry, S) as AdjacencyMatrixSelector<IdVecKey, E>>::Matrix,
    directedness: PhantomData<D>,
    #[cfg(not(feature = "unchecked"))]
    id: GraphId,
}

impl<N, E, D, S> AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, E>,
{
    pub fn new() -> Self {
        Self {
            nodes: IdVec::new(),
            adjacency: SelectMatrix::<D::Symmetry, S, IdVecKey, E>::new(),
            directedness: PhantomData,
            #[cfg(not(feature = "unchecked"))]
            id: GraphId::new(),
        }
    }

    fn node_id(&self, index: IdVecKey) -> NodeId {
        NodeId {
            index,
            #[cfg(not(feature = "unchecked"))]
            graph_id: self.id,
        }
    }

    fn edge_id(&self, from: IdVecKey, into: IdVecKey) -> EdgeId {
        let (i1, i2) = maybe_sort_pair(from, into, !self.is_directed());
        EdgeId {
            from: i1,
            into: i2,
            #[cfg(not(feature = "unchecked"))]
            graph_id: self.id,
        }
    }

    fn compact_nodes<F>(
        &mut self,
        mut compact_fn: F,
        mut node_id_callback: Option<impl FnMut(NodeId, NodeId)>,
        mut edge_id_callback: Option<impl FnMut(EdgeId, EdgeId)>,
    ) where
        F: FnMut(&mut IdVec<N>, Option<&mut dyn FnMut(IdVecKey, Option<IdVecKey>)>),
    {
        let mut id_vec_map: HashMap<IdVecKey, IdVecKey> = HashMap::new();

        compact_fn(
            &mut self.nodes,
            Some(&mut |old_key, new_key_opt| {
                if (node_id_callback.is_some() || edge_id_callback.is_some())
                    && let Some(new_key) = new_key_opt
                {
                    id_vec_map.insert(old_key, new_key);
                }
            }),
        );

        // Call node_id_callback for each node ID mapping
        if let Some(ref mut cb) = node_id_callback {
            for (&old_index, &new_index) in &id_vec_map {
                let old_node_id = self.node_id(old_index);
                let new_node_id = self.node_id(new_index);
                cb(old_node_id, new_node_id);
            }
        }

        // Call edge_id_callback for each edge ID mapping
        if let Some(ref mut cb) = edge_id_callback {
            for id in self.edge_ids() {
                let (from, into) = id.into();
                let new_from = id_vec_map
                    .get(&from)
                    .copied()
                    .expect("invalid from node ID");
                let new_into = id_vec_map
                    .get(&into)
                    .copied()
                    .expect("invalid into node ID");
                let new_edge_id = self.edge_id(new_from, new_into);
                cb(id, new_edge_id);
            }
        }
    }
}

impl<N, E, D, S> Graph for AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, E>,
{
    type EdgeData = E;
    type EdgeId = EdgeId;
    type NodeData = N;
    type NodeId = NodeId;
    type Directedness = D;

    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData {
        self.check_node_id(&id);
        &self.nodes.get(id.into()).expect("no such node")
    }

    fn node_ids(&self) -> impl Iterator<Item = <Self as Graph>::NodeId> {
        self.nodes.iter_keys().map(|index| self.node_id(index))
    }

    fn edge_data(&self, eid: Self::EdgeId) -> &Self::EdgeData {
        self.check_edge_id(&eid);
        let (from, to) = eid.into();
        &self.adjacency.get(from, to).expect("no such edge")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .edges()
            .map(|(from, into, _)| self.edge_id(from, into))
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::NodeId, Self::NodeId) {
        self.check_edge_id(&eid);
        (self.node_id(eid.from), self.node_id(eid.into))
    }

    fn edges_between(
        &self,
        from: Self::NodeId,
        into: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.check_node_id(&from);
        self.check_node_id(&into);
        self.adjacency
            .edge_between(from.into(), into.into())
            .into_iter()
            .map(|(from, into, _)| self.edge_id(from, into))
    }

    fn edges_into<'a>(&'a self, into: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.check_node_id(&into);
        self.adjacency
            .edges_into(into.into())
            .map(|(from, _)| self.edge_id(from, into.into()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_from<'a>(&'a self, from: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.check_node_id(&from);
        self.adjacency
            .edges_from(from.into())
            .map(|(into, _)| self.edge_id(from.into(), into))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn is_valid_node_id(&self, id: &Self::NodeId) -> bool {
        #[cfg(not(feature = "unchecked"))]
        {
            self.id == id.graph_id && self.nodes.get(id.index).is_some()
        }
        #[cfg(feature = "unchecked")]
        {
            self.node_ids().any(|nid: NodeId<N, E>| &nid == id)
        }
    }

    fn is_maybe_valid_node_id(&self, id: &Self::NodeId) -> bool {
        #[cfg(not(feature = "unchecked"))]
        {
            self.is_valid_node_id(id)
        }
        #[cfg(feature = "unchecked")]
        {
            true
        }
    }

    fn is_valid_edge_id(&self, id: &Self::EdgeId) -> bool {
        #[cfg(not(feature = "unchecked"))]
        {
            self.id == id.graph_id && self.adjacency.get(id.from, id.into).is_some()
        }
        #[cfg(feature = "unchecked")]
        {
            self.edge_ids().any(|eid: EdgeId<N, E>| &eid == id)
        }
    }

    fn is_maybe_valid_edge_id(&self, id: &Self::EdgeId) -> bool {
        #[cfg(not(feature = "unchecked"))]
        {
            self.is_valid_edge_id(id)
        }
        #[cfg(feature = "unchecked")]
        {
            true
        }
    }
}

impl<N, E, D, S> GraphMut for AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, E>,
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
        self.nodes.remove(id.into()).expect("invalid node ID")
    }

    fn remove_edge(&mut self, id: Self::EdgeId) -> Self::EdgeData {
        self.adjacency
            .remove(id.from, id.into)
            .expect("Invalid edge ID")
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.adjacency.clear();
    }

    fn add_edge(
        &mut self,
        from: Self::NodeId,
        to: Self::NodeId,
        data: Self::EdgeData,
    ) -> Self::EdgeId {
        self.add_or_replace_edge(from, to, data).0
    }

    fn reserve(&mut self, additional_nodes: usize, additional_vertices: usize) {
        self.nodes.reserve(additional_nodes);
        self.adjacency.reserve(additional_vertices);
    }

    fn reserve_exact(&mut self, additional_nodes: usize, additional_vertices: usize) {
        self.nodes.reserve_exact(additional_nodes);
        self.adjacency.reserve_exact(additional_vertices);
    }

    fn compact(&mut self) {
        self.compact_with(
            None::<fn(Self::NodeId, Self::NodeId)>,
            None::<fn(Self::EdgeId, Self::EdgeId)>,
        );
    }

    fn compact_with(
        &mut self,
        node_id_callback: Option<impl FnMut(Self::NodeId, Self::NodeId)>,
        edge_id_callback: Option<impl FnMut(Self::EdgeId, Self::EdgeId)>,
    ) {
        self.compact_nodes(
            |vec, cb| vec.compact_with(cb),
            node_id_callback,
            edge_id_callback,
        );
        self.adjacency.compact();
    }

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_with(
            None::<fn(Self::NodeId, Self::NodeId)>,
            None::<fn(Self::EdgeId, Self::EdgeId)>,
        );
    }

    fn shrink_to_fit_with(
        &mut self,
        node_id_callback: Option<impl FnMut(Self::NodeId, Self::NodeId)>,
        edge_id_callback: Option<impl FnMut(Self::EdgeId, Self::EdgeId)>,
    ) {
        self.compact_nodes(
            |vec, cb| vec.shrink_to_fit_with(cb),
            node_id_callback,
            edge_id_callback,
        );
        self.adjacency.shrink_to_fit();
    }
}

impl<N, E, D, S> Debug for AdjacencyGraph<N, E, D, S>
where
    N: Debug,
    E: Debug,
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, E>,
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
        (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, String>,
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
