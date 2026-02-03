use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

/// Node and edge ID types for adjacency graphs.
pub use self::ids::{EdgeId, NodeId};
use crate::{
    AdjacencyMatrix, Directed, Graph, GraphMut, SingleEdge,
    adjacency_matrix::{
        AdjacencyMatrixSelector, CompactionCount as _, HashStorage, SelectMatrix, Storage,
    },
    debug::format_debug,
    directedness::Directedness,
    graph_id::GraphId,
    id_vec::{IdVec, IdVecKey},
    pairs::Pair,
};

mod ids;

/// A graph implementation using an adjacency matrix for edge storage.
///
/// This graph stores nodes in a contiguous vector and uses an adjacency matrix
/// to track edges. The matrix implementation can be selected via the `S` (storage)
/// type parameter, supporting either hash-based or bitvec-based storage.
///
/// Multiple edges between the same pair of nodes are not supported; adding an edge
/// between two nodes that already have an edge will replace the existing edge's data.
///
/// # Type Parameters
/// * `N` - The type of data stored in nodes
/// * `E` - The type of data stored in edges
/// * `D` - The directedness ([`Directed`] or [`Undirected`](crate::Undirected))
/// * `S` - The storage type ([`HashStorage`] or [`BitvecStorage`](crate::adjacency_matrix::BitvecStorage))
pub struct AdjacencyGraph<N, E, D = Directed, S = HashStorage>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<usize, E>,
{
    nodes: IdVec<N>,
    adjacency: <(D::Symmetry, S) as AdjacencyMatrixSelector<usize, E>>::Matrix,
    directedness: PhantomData<D>,
    id: GraphId,
    compaction_count: S::CompactionCount,
}

impl<N, E, D, S> AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<usize, E>,
{
    /// Creates a `NodeId` for the given `IdVecKey`.
    fn node_id(&self, key: IdVecKey) -> NodeId<S> {
        NodeId::new(key, self.id, self.compaction_count)
    }

    /// Creates an `EdgeId` for the given `IdVecKey` pair.
    fn edge_id(&self, from: IdVecKey, into: IdVecKey) -> EdgeId<S, D> {
        EdgeId::new((from, into).into(), self.id, self.compaction_count)
    }

    /// Internal function to perform compaction or shrinking of the graph.
    /// This function handles updating node and edge IDs via the provided callbacks.
    fn do_compact<F>(
        &mut self,
        mut compact_fn: F,
        node_id_callback: &mut dyn FnMut(
            &<AdjacencyGraph<N, E, D, S> as crate::graph::Graph>::NodeId,
            &<AdjacencyGraph<N, E, D, S> as crate::graph::Graph>::NodeId,
        ),
        edge_id_callback: &mut dyn FnMut(
            &<AdjacencyGraph<N, E, D, S> as crate::graph::Graph>::EdgeId,
            &<AdjacencyGraph<N, E, D, S> as crate::graph::Graph>::EdgeId,
        ),
    ) where
        F: FnMut(&mut IdVec<N>, &mut dyn FnMut(IdVecKey, Option<IdVecKey>)),
    {
        let old_compaction_count = self.compaction_count;
        let new_compaction_count = self.compaction_count.increment();

        // Compact nodes and build ID mapping.
        let mut id_vec_map: HashMap<IdVecKey, IdVecKey> = HashMap::with_capacity(self.nodes.len());
        let old_indexing = self.nodes.indexing();
        compact_fn(&mut self.nodes, &mut |old_key, new_key_opt| {
            if let Some(new_key) = new_key_opt {
                id_vec_map.insert(old_key, new_key);
            }
        });
        let new_indexing = self.nodes.indexing();

        // Call node_id_callback for each node ID mapping
        for (&old_index, &new_index) in &id_vec_map {
            let old_node_id = self
                .node_id(old_index)
                .with_compaction_count(old_compaction_count);
            let new_node_id = self
                .node_id(new_index)
                .with_compaction_count(new_compaction_count);
            node_id_callback(&old_node_id, &new_node_id);
        }

        // Update adjacency matrix with new node indices.
        if !id_vec_map.is_empty() {
            let mut old_adjacency = self.adjacency.clone_empty();
            std::mem::swap(&mut self.adjacency, &mut old_adjacency);
            self.adjacency.reserve(self.nodes.len());
            for (old_from_index, old_into_index, data) in old_adjacency.into_iter() {
                let old_from = old_indexing.key_from_index(old_from_index);
                let old_into = old_indexing.key_from_index(old_into_index);

                // Skip edges where either endpoint was removed
                let Some(&new_from) = id_vec_map.get(&old_from) else {
                    continue;
                };
                let Some(&new_into) = id_vec_map.get(&old_into) else {
                    continue;
                };

                self.adjacency.insert(
                    new_indexing.zero_based_index(new_from),
                    new_indexing.zero_based_index(new_into),
                    data,
                );
                let old_edge_id = self
                    .edge_id(old_from, old_into)
                    .with_compaction_count(old_compaction_count);
                let new_edge_id = self
                    .edge_id(new_from, new_into)
                    .with_compaction_count(new_compaction_count);
                edge_id_callback(&old_edge_id, &new_edge_id);
            }
        }

        self.compaction_count = new_compaction_count;
    }
}

impl<N, E, D, S> Graph for AdjacencyGraph<N, E, D, S>
where
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<usize, E>,
{
    type EdgeData = E;
    type EdgeId = EdgeId<S, D>;
    type NodeData = N;
    type NodeId = NodeId<S>;
    type Directedness = D;
    type EdgeMultiplicity = SingleEdge;

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        self.assert_valid_node_id(id);
        &self.nodes.get(id.key()).expect("no such node")
    }

    fn node_ids(&self) -> impl Iterator<Item = <Self as Graph>::NodeId> {
        self.nodes.iter_keys().map(|key| self.node_id(key))
    }

    fn edge_data(&self, eid: &Self::EdgeId) -> &Self::EdgeData {
        self.assert_valid_edge_id(eid);
        let (from, to) = eid.keys().into();
        &self
            .adjacency
            .get(
                self.nodes.zero_based_index(from),
                self.nodes.zero_based_index(to),
            )
            .expect("no such edge")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency.iter().map(|(from, into, _)| {
            self.edge_id(
                self.nodes.key_from_index(from),
                self.nodes.key_from_index(into),
            )
        })
    }

    fn edges_between<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(from);
        self.assert_valid_node_id(into);
        self.adjacency
            .entry_at(
                self.nodes.zero_based_index(from.key()),
                self.nodes.zero_based_index(into.key()),
            )
            .into_iter()
            .map(|(indicies, _)| {
                self.edge_id(
                    self.nodes.key_from_index(indicies.clone().into_first()),
                    self.nodes.key_from_index(indicies.into_second()),
                )
            })
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(into);
        self.adjacency
            .entries_in_col(self.nodes.zero_based_index(into.key()))
            .map(|(from, _)| self.edge_id(self.nodes.key_from_index(from), into.key()))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(from);
        self.adjacency
            .entries_in_row(self.nodes.zero_based_index(from.key()))
            .map(|(into, _)| self.edge_id(from.key(), self.nodes.key_from_index(into)))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        if self.id != id.graph_id {
            return Err("NodeId graph ID does not match");
        }
        if self.compaction_count != id.compaction_count {
            return Err("NodeId compaction counter does not match");
        }
        if self.nodes.get(id.key()).is_none() {
            return Err("NodeId index not found in nodes");
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
            let _ = id;
            Ok(())
        }
    }

    fn check_valid_edge_id(&self, id: &Self::EdgeId) -> Result<(), &'static str> {
        if self.id != id.graph_id() {
            return Err("EdgeId graph ID does not match");
        }
        if self.compaction_count != id.compaction_count() {
            return Err("EdgeId compaction counter does not match");
        }
        if self
            .adjacency
            .get(
                self.nodes.zero_based_index(id.keys().into_first()),
                self.nodes.zero_based_index(id.keys().into_second()),
            )
            .is_none()
        {
            return Err("EdgeId not found in adjacency matrix");
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
    (D::Symmetry, S): AdjacencyMatrixSelector<usize, E>,
{
    fn new() -> Self {
        Self {
            nodes: IdVec::new(),
            adjacency: SelectMatrix::<D::Symmetry, S, usize, E>::new(),
            directedness: PhantomData,
            compaction_count: S::CompactionCount::default(),
            id: GraphId::new(),
        }
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let index = self.nodes.insert(data);
        self.node_id(index)
    }

    fn add_or_replace_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let old_data = self.adjacency.insert(
            self.nodes.zero_based_index(from.key()),
            self.nodes.zero_based_index(into.key()),
            data,
        );
        (self.edge_id(from.key(), into.key()), old_data)
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData {
        for into in self
            .adjacency
            .entries_in_row(self.nodes.zero_based_index(id.key()))
            .map(|(to, _)| to)
            .collect::<Vec<_>>()
        {
            self.adjacency
                .remove(self.nodes.zero_based_index(id.key()), into);
        }
        if self.is_directed() {
            for from in self
                .adjacency
                .entries_in_col(self.nodes.zero_based_index(id.key()))
                .map(|(to, _)| to)
                .collect::<Vec<_>>()
            {
                self.adjacency
                    .remove(from, self.nodes.zero_based_index(id.key()));
            }
        }
        self.nodes.remove(id.key()).expect("invalid node ID")
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData {
        self.adjacency
            .remove(
                self.nodes.zero_based_index(id.keys().into_first()),
                self.nodes.zero_based_index(id.keys().into_second()),
            )
            .expect("Invalid edge ID")
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.adjacency.clear();
    }

    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        to: &Self::NodeId,
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
        self.compact_with(|_, _| {}, |_, _| {});
    }

    fn compact_with(
        &mut self,
        mut node_id_callback: impl FnMut(&Self::NodeId, &Self::NodeId),
        mut edge_id_callback: impl FnMut(&Self::EdgeId, &Self::EdgeId),
    ) {
        self.do_compact(
            |vec, cb| vec.compact_with(cb),
            &mut node_id_callback,
            &mut edge_id_callback,
        );
    }

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_with(|_, _| {}, |_, _| {});
    }
    fn shrink_to_fit_with(
        &mut self,
        mut node_id_callback: impl FnMut(&Self::NodeId, &Self::NodeId),
        mut edge_id_callback: impl FnMut(&Self::EdgeId, &Self::EdgeId),
    ) {
        self.do_compact(
            |vec, cb| vec.shrink_to_fit_with(cb),
            &mut node_id_callback,
            &mut edge_id_callback,
        );
    }
}

impl<N, E, D, S> Debug for AdjacencyGraph<N, E, D, S>
where
    N: Debug,
    E: Debug,
    D: Directedness,
    S: Storage,
    (D::Symmetry, S): AdjacencyMatrixSelector<usize, E>,
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
        (D::Symmetry, S): AdjacencyMatrixSelector<usize, String>,
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
            #[cfg(not(feature = "unchecked"))]
            #[test]
            #[should_panic]
            fn test_check_node_id_panics_after_compaction() {
                let mut graph: $type = GraphMut::new();
                let n1 = graph.add_node(1);
                graph.compact();
                graph.assert_valid_node_id(&n1);
            }

            #[cfg(not(feature = "unchecked"))]
            #[test]
            #[should_panic]
            fn test_check_edge_id_panics_after_compaction() {
                let mut graph: $type = GraphMut::new();
                let n1 = graph.add_node(1);
                let n2 = graph.add_node(2);
                let e1 = graph.add_edge(&n1, &n2, "edge".to_string());
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

        type TestGraph = AdjacencyGraph<i32, String, Directed, BitvecStorage>;

        graph_tests!(TestGraph);
        graph_test_copy_from_with!(TestGraph, |data| data * 2, |data: &String| format!(
            "{}-copied",
            data
        ));
        test_compaction!(TestGraph);
    }

    mod undirected_bitvec {
        use super::*;
        use crate::{
            adjacency_matrix::BitvecStorage, directedness::Undirected, graph_test_copy_from_with,
            graph_tests,
        };

        type TestGraph = AdjacencyGraph<i32, String, Undirected, BitvecStorage>;

        graph_tests!(TestGraph);
        graph_test_copy_from_with!(TestGraph, |data| data * 2, |data: &String| format!(
            "{}-copied",
            data
        ));
        test_compaction!(TestGraph);
    }

    mod directed_hash {
        use super::*;
        use crate::{directedness::Directed, graph_test_copy_from_with, graph_tests};

        type TestGraph = AdjacencyGraph<i32, String, Directed, HashStorage>;

        graph_tests!(TestGraph);
        graph_test_copy_from_with!(TestGraph, |data| data * 2, |data: &String| format!(
            "{}-copied",
            data
        ));
    }

    mod undirected_hash {
        use super::*;
        use crate::{directedness::Undirected, graph_test_copy_from_with, graph_tests};

        type TestGraph = AdjacencyGraph<i32, String, Undirected, HashStorage>;

        graph_tests!(TestGraph);
        graph_test_copy_from_with!(TestGraph, |data| data * 2, |data: &String| format!(
            "{}-copied",
            data
        ));
    }
}
