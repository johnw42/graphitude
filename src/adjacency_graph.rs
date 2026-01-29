#![cfg(feature = "bitvec")]
use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData};

use crate::{
    AdjacencyMatrix, Directed, Graph, GraphMut,
    adjacency_matrix::{AdjacencyMatrixSelector, HashStorage, SelectMatrix, Storage},
    debug::format_debug,
    directedness::Directedness,
    graph_id::GraphId,
    id_vec::{IdVec, IdVecKey},
    util::sort_pair_if,
};

pub struct NodeId<S: Storage> {
    index: IdVecKey,
    #[cfg(not(feature = "unchecked"))]
    compaction_counter: S::CompactionCounter,
    #[cfg(not(feature = "unchecked"))]
    graph_id: GraphId,
}

impl<S: Storage> Clone for NodeId<S> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            #[cfg(not(feature = "unchecked"))]
            compaction_counter: self.compaction_counter,
            #[cfg(not(feature = "unchecked"))]
            graph_id: self.graph_id,
        }
    }
}

impl<S: Storage> Copy for NodeId<S> {}

impl<S: Storage> Into<IdVecKey> for NodeId<S> {
    fn into(self) -> IdVecKey {
        self.index
    }
}

impl<S: Storage> PartialEq for NodeId<S> {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(not(feature = "unchecked"))]
        assert_eq!(self.graph_id, other.graph_id);
        self.index == other.index
    }
}

impl<S: Storage> Eq for NodeId<S> {}

impl<S: Storage> Hash for NodeId<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl<S: Storage> PartialOrd for NodeId<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.index.cmp(&other.index))
    }
}

impl<S: Storage> Ord for NodeId<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.index.cmp(&other.index)
    }
}

impl<S: Storage> Debug for NodeId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.index)
    }
}

pub struct EdgeId<S: Storage> {
    from: IdVecKey,
    into: IdVecKey,
    #[cfg(not(feature = "unchecked"))]
    compaction_counter: S::CompactionCounter,
    #[cfg(not(feature = "unchecked"))]
    graph_id: GraphId,
}

impl<S: Storage> Clone for EdgeId<S> {
    fn clone(&self) -> Self {
        Self {
            from: self.from,
            into: self.into,
            #[cfg(not(feature = "unchecked"))]
            compaction_counter: self.compaction_counter,
            #[cfg(not(feature = "unchecked"))]
            graph_id: self.graph_id,
        }
    }
}

impl<S: Storage> Copy for EdgeId<S> {}

impl<S: Storage> PartialEq for EdgeId<S> {
    fn eq(&self, other: &Self) -> bool {
        #[cfg(not(feature = "unchecked"))]
        assert_eq!(self.graph_id, other.graph_id);
        self.from == other.from && self.into == other.into
    }
}

impl<S: Storage> Eq for EdgeId<S> {}

impl<S: Storage> Hash for EdgeId<S> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.from.hash(state);
        self.into.hash(state);
    }
}

impl<S: Storage> Into<(IdVecKey, IdVecKey)> for EdgeId<S> {
    fn into(self) -> (IdVecKey, IdVecKey) {
        (self.from, self.into)
    }
}

impl<'a, S: Storage> Into<(&'a IdVecKey, &'a IdVecKey)> for &'a EdgeId<S> {
    fn into(self) -> (&'a IdVecKey, &'a IdVecKey) {
        (&self.from, &self.into)
    }
}

impl<S: Storage> Debug for EdgeId<S> {
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
    compaction_counter: S::CompactionCounter,
    #[cfg(not(feature = "unchecked"))]
    id: GraphId,
}

impl<N, E, D, S> AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, E>,
{
    fn node_id(&self, index: IdVecKey) -> NodeId<S> {
        NodeId {
            index,
            #[cfg(not(feature = "unchecked"))]
            compaction_counter: self.compaction_counter,
            #[cfg(not(feature = "unchecked"))]
            graph_id: self.id,
        }
    }

    fn edge_id(&self, from: IdVecKey, into: IdVecKey) -> EdgeId<S> {
        let (i1, i2) = sort_pair_if(from, into, !D::is_directed());
        EdgeId {
            from: i1,
            into: i2,
            #[cfg(not(feature = "unchecked"))]
            compaction_counter: self.compaction_counter,
            #[cfg(not(feature = "unchecked"))]
            graph_id: self.id,
        }
    }

    fn compact_nodes<F>(
        &mut self,
        mut compact_fn: F,
        mut node_id_callback: Option<impl FnMut(NodeId<S>, NodeId<S>)>,
        mut edge_id_callback: Option<impl FnMut(EdgeId<S>, EdgeId<S>)>,
    ) where
        F: FnMut(&mut IdVec<N>, Option<&mut dyn FnMut(IdVecKey, Option<IdVecKey>)>),
    {
        let mut id_vec_map: HashMap<IdVecKey, IdVecKey> = HashMap::new();

        let new_compaction_counter = S::increment_compaction_counter(self.compaction_counter);

        compact_fn(
            &mut self.nodes,
            Some(&mut |old_key, new_key_opt| {
                if (node_id_callback.is_some() || edge_id_callback.is_some())
                    && let Some(new_key) = new_key_opt
                {
                    dbg!(&old_key, &new_key);
                    id_vec_map.insert(old_key, new_key);
                }
            }),
        );

        // Call node_id_callback for each node ID mapping
        if let Some(ref mut cb) = node_id_callback {
            for (&old_index, &new_index) in &id_vec_map {
                let old_node_id = self.node_id(old_index);
                let mut new_node_id = self.node_id(new_index);
                new_node_id.compaction_counter = new_compaction_counter;
                cb(old_node_id, new_node_id);
            }
        }

        // Call edge_id_callback for each edge ID mapping
        if let Some(ref mut cb) = edge_id_callback {
            for id in self.edge_ids() {
                let (from, into) = id.into();
                // If a node index isn't in the map, it means it wasn't moved (identity mapping)
                let new_from = id_vec_map.get(&from).copied().unwrap_or(from);
                let new_into = id_vec_map.get(&into).copied().unwrap_or(into);
                let mut new_edge_id = self.edge_id(new_from, new_into);
                new_edge_id.compaction_counter = new_compaction_counter;
                cb(id, new_edge_id);
            }
        }

        self.compaction_counter = new_compaction_counter;
    }
}

impl<N, E, D, S> Graph for AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, E>,
{
    type EdgeData = E;
    type EdgeId = EdgeId<S>;
    type NodeData = N;
    type NodeId = NodeId<S>;
    type Directedness = D;

    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData {
        self.assert_valid_node_id(&id);
        &self.nodes.get(id.into()).expect("no such node")
    }

    fn node_ids(&self) -> impl Iterator<Item = <Self as Graph>::NodeId> {
        self.nodes.iter_keys().map(|index| self.node_id(index))
    }

    fn edge_data(&self, eid: Self::EdgeId) -> &Self::EdgeData {
        self.assert_valid_edge_id(&eid);
        let (from, to) = eid.into();
        &self.adjacency.get(from, to).expect("no such edge")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .entries()
            .map(|(from, into, _)| self.edge_id(from, into))
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::NodeId, Self::NodeId) {
        self.assert_valid_edge_id(&eid);
        (self.node_id(eid.from), self.node_id(eid.into))
    }

    fn edges_between(
        &self,
        from: Self::NodeId,
        into: Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.assert_valid_node_id(&from);
        self.assert_valid_node_id(&into);
        self.adjacency
            .entry_at(from.into(), into.into())
            .into_iter()
            .map(|(from, into, _)| self.edge_id(from, into))
    }

    fn edges_into<'a>(&'a self, into: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(&into);
        self.adjacency
            .entries_in_col(into.into())
            .map(|(from, _)| self.edge_id(from, into.into()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_from<'a>(&'a self, from: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(&from);
        self.adjacency
            .entries_in_row(from.into())
            .map(|(into, _)| self.edge_id(from.into(), into))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        #[cfg(not(feature = "unchecked"))]
        {
            if self.id != id.graph_id {
                return Err("NodeId graph ID does not match");
            }
            if self.compaction_counter != id.compaction_counter {
                return Err("NodeId compaction counter does not match");
            }
            if self.nodes.get(id.index).is_none() {
                return Err("NodeId index not found in nodes");
            }
        }
        #[cfg(feature = "unchecked")]
        {
            if !self.node_ids().any(|nid: NodeId<N, E>| &nid == id) {
                return Err("NodeId not found in graph");
            }
        }
        Ok(())
    }

    fn maybe_check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        #[cfg(not(feature = "unchecked"))]
        {
            self.check_valid_node_id(id)
        }
        #[cfg(feature = "unchecked")]
        {
            Ok(())
        }
    }

    fn check_valid_edge_id(&self, id: &Self::EdgeId) -> Result<(), &'static str> {
        #[cfg(not(feature = "unchecked"))]
        {
            if self.id != id.graph_id {
                return Err("EdgeId graph ID does not match");
            }
            if self.compaction_counter != id.compaction_counter {
                return Err("EdgeId compaction counter does not match");
            }
            if self.adjacency.get(id.from, id.into).is_none() {
                return Err("EdgeId not found in adjacency matrix");
            }
        }
        #[cfg(feature = "unchecked")]
        {
            if !self.edge_ids().any(|eid: EdgeId<N, E>| &eid == id) {
                return Err("EdgeId not found in graph");
            }
        }
        Ok(())
    }

    fn maybe_check_valid_edge_id(&self, _id: &Self::EdgeId) -> Result<(), &'static str> {
        #[cfg(not(feature = "unchecked"))]
        {
            self.check_valid_edge_id(_id)
        }
        #[cfg(feature = "unchecked")]
        {
            Ok(())
        }
    }
}

impl<N, E, D, S> GraphMut for AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<IdVecKey, E>,
{
    fn new() -> Self {
        Self {
            nodes: IdVec::new(),
            adjacency: SelectMatrix::<D::Symmetry, S, IdVecKey, E>::new(),
            directedness: PhantomData,
            compaction_counter: S::CompactionCounter::default(),
            #[cfg(not(feature = "unchecked"))]
            id: GraphId::new(),
        }
    }

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
            .entries_in_row(id.into())
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

    fn reserve(&mut self, additional_nodes: usize, _additional_edges: usize) {
        self.nodes.reserve(additional_nodes);
        // self.adjacency.reserve(additional_edges);
    }

    fn reserve_exact(&mut self, additional_nodes: usize, _additional_edges: usize) {
        self.nodes.reserve_exact(additional_nodes);
        // self.adjacency.reserve_exact(additional_edges);
    }

    fn compact(&mut self) {
        self.compact_with::<fn(Self::NodeId, Self::NodeId), fn(Self::EdgeId, Self::EdgeId)>(
            None, None,
        );
    }

    fn compact_with<F1, F2>(&mut self, node_id_callback: Option<F1>, edge_id_callback: Option<F2>)
    where
        F1: FnMut(Self::NodeId, Self::NodeId),
        F2: FnMut(Self::EdgeId, Self::EdgeId),
    {
        self.compact_nodes(
            |vec, cb| vec.compact_with(cb),
            node_id_callback,
            edge_id_callback,
        );
        // self.adjacency.compact();
    }

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_with::<fn(Self::NodeId, Self::NodeId), fn(Self::EdgeId, Self::EdgeId)>(
            None, None,
        );
    }

    fn shrink_to_fit_with<F1, F2>(
        &mut self,
        node_id_callback: Option<F1>,
        edge_id_callback: Option<F2>,
    ) where
        F1: FnMut(Self::NodeId, Self::NodeId),
        F2: FnMut(Self::EdgeId, Self::EdgeId),
    {
        self.compact_nodes(
            |vec, cb| vec.shrink_to_fit_with(cb),
            node_id_callback,
            edge_id_callback,
        );
        // self.adjacency.shrink_to_fit();
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

        fn new_edge_data(i: usize) -> String {
            format!("e{}", i)
        }

        fn new_node_data(i: usize) -> i32 {
            i as i32
        }
    }

    macro_rules! test_compaction {
        ($type:ty) => {
            #[test]
            #[should_panic]
            fn test_check_node_id_panics_after_compaction() {
                let mut graph: $type = GraphMut::new();
                let n1 = graph.add_node(1);
                graph.compact();
                graph.assert_valid_node_id(&n1);
            }

            #[test]
            #[should_panic]
            fn test_check_edge_id_panics_after_compaction() {
                let mut graph: $type = GraphMut::new();
                let n1 = graph.add_node(1);
                let n2 = graph.add_node(2);
                let e1 = graph.add_edge(n1, n2, "edge".to_string());
                graph.compact();
                graph.assert_valid_edge_id(&e1);
            }
        };
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
        test_compaction!(AdjacencyGraph<i32, String, Directed, BitvecStorage>);
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
        test_compaction!(AdjacencyGraph<i32, String, Undirected, BitvecStorage>);
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
