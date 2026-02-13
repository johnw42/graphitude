use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

/// Node and edge ID types for adjacency graphs.
pub use self::ids::{EdgeId, NodeId};
use crate::{
    AdjacencyMatrix,
    adjacency_matrix::{
        AdjacencyMatrixSelector, CompactionCount as _, HashStorage, SelectMatrix, Storage,
    },
    automap::{Automap, trait_def::AutomapIndexing},
    debug::format_debug,
    directedness::DirectednessTrait,
    graph_id::GraphId,
    prelude::*,
};

#[cfg(not(feature = "bitvec"))]
use crate::automap::indexed::{IndexedAutomap, IndexedAutomapKey};
#[cfg(feature = "bitvec")]
use crate::automap::{OffsetAutomap, OffsetAutomapKey};

// Use OffsetAutomap when bitvec feature is enabled, otherwise use IndexedAutomap
#[cfg(feature = "bitvec")]
type NodeVec<N> = OffsetAutomap<N>;
#[cfg(not(feature = "bitvec"))]
type NodeVec<N> = IndexedAutomap<N>;

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
    D: DirectednessTrait + Default,
    S: Storage,
    (D, S): AdjacencyMatrixSelector<usize, E>,
{
    nodes: NodeVec<N>,
    adjacency: <(D, S) as AdjacencyMatrixSelector<usize, E>>::Matrix,
    directedness: PhantomData<D>,
    id: GraphId,
    compaction_count: S::CompactionCount,
}

type NodeIdCallback<'a, N, E, D, S> = dyn for<'b> FnMut(
        &'b <AdjacencyGraph<N, E, D, S> as Graph>::NodeId,
        &'b <AdjacencyGraph<N, E, D, S> as Graph>::NodeId,
    ) + 'a;

type EdgeIdCallback<'a, N, E, D, S> = dyn for<'b> FnMut(
        &'b <AdjacencyGraph<N, E, D, S> as Graph>::EdgeId,
        &'b <AdjacencyGraph<N, E, D, S> as Graph>::EdgeId,
    ) + 'a;

impl<N, E, D, S> AdjacencyGraph<N, E, D, S>
where
    D: DirectednessTrait + Default,
    S: Storage,
    (D, S): AdjacencyMatrixSelector<usize, E>,
{
    /// Creates a `NodeId` for the given `AutomapKey`.
    fn node_id(&self, key: OffsetAutomapKey) -> NodeId<S> {
        NodeId::new(key, self.id, self.compaction_count)
    }

    /// Creates an `EdgeId` for the given `AutomapKey` pair.
    fn edge_id(&self, from: OffsetAutomapKey, into: OffsetAutomapKey) -> EdgeId<S, D> {
        EdgeId::new(
            D::default().make_pair(from, into),
            self.id,
            self.compaction_count,
        )
    }

    /// Internal function to perform compaction or shrinking of the graph.
    /// This function handles updating node and edge IDs via the provided callbacks.
    fn do_compact<F>(
        &mut self,
        mut compact_fn: F,
        node_id_callback: &mut NodeIdCallback<'_, N, E, D, S>,
        edge_id_callback: &mut EdgeIdCallback<'_, N, E, D, S>,
    ) where
        F: FnMut(&mut NodeVec<N>, &mut dyn FnMut(OffsetAutomapKey, Option<OffsetAutomapKey>)),
    {
        let old_compaction_count = self.compaction_count;
        let new_compaction_count = self.compaction_count.increment();

        // Compact nodes and build ID mapping.
        let mut automap_map: HashMap<OffsetAutomapKey, OffsetAutomapKey> =
            HashMap::with_capacity(self.nodes.len());
        let old_indexing = self.nodes.indexing();
        compact_fn(&mut self.nodes, &mut |old_key, new_key_opt| {
            if let Some(new_key) = new_key_opt {
                automap_map.insert(old_key, new_key);
            }
        });
        let new_indexing = self.nodes.indexing();

        // Call node_id_callback for each node ID mapping
        for (&old_index, &new_index) in &automap_map {
            let old_node_id = self
                .node_id(old_index)
                .with_compaction_count(old_compaction_count);
            let new_node_id = self
                .node_id(new_index)
                .with_compaction_count(new_compaction_count);
            node_id_callback(&old_node_id, &new_node_id);
        }

        // Update adjacency matrix with new node indices.
        if !automap_map.is_empty() {
            let mut old_adjacency = self.adjacency.clone_empty();
            std::mem::swap(&mut self.adjacency, &mut old_adjacency);
            self.adjacency.reserve(self.nodes.len());
            for (old_from_index, old_into_index, data) in old_adjacency.into_iter() {
                let old_from = old_indexing.index_to_key(old_from_index);
                let old_into = old_indexing.index_to_key(old_into_index);

                // Skip edges where either endpoint was removed
                let Some(&new_from) = automap_map.get(&old_from) else {
                    continue;
                };
                let Some(&new_into) = automap_map.get(&old_into) else {
                    continue;
                };

                self.adjacency.insert(
                    new_indexing.key_to_index(new_from),
                    new_indexing.key_to_index(new_into),
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
    D: DirectednessTrait + Default,
    S: Storage,
    (D, S): AdjacencyMatrixSelector<usize, E>,
{
    type EdgeData = E;
    type EdgeId = EdgeId<S, D>;
    type NodeData = N;
    type NodeId = NodeId<S>;
    type Directedness = D;
    type EdgeMultiplicity = SingleEdge;

    fn directedness(&self) -> Self::Directedness {
        D::default()
    }

    fn edge_multiplicity(&self) -> Self::EdgeMultiplicity {
        SingleEdge
    }

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        self.assert_valid_node_id(id);
        self.nodes.get(id.key()).expect("no such node")
    }

    fn node_ids(&self) -> impl Iterator<Item = <Self as Graph>::NodeId> {
        self.nodes.iter_keys().map(|key| self.node_id(key))
    }

    fn edge_data(&self, eid: &Self::EdgeId) -> &Self::EdgeData {
        self.assert_valid_edge_id(eid);
        let (from, to) = eid.keys().into_values();
        self.adjacency
            .get(
                self.nodes.indexing().key_to_index(from),
                self.nodes.indexing().key_to_index(to),
            )
            .expect("no such edge")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency.iter().map(|(from, into, _)| {
            self.edge_id(
                self.nodes.indexing().index_to_key(from),
                self.nodes.indexing().index_to_key(into),
            )
        })
    }

    fn num_edges(&self) -> usize {
        self.adjacency.len()
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(from);
        self.assert_valid_node_id(into);
        let indexing = self.nodes.indexing();
        self.adjacency
            .entry_at(
                indexing.key_to_index(from.key()),
                indexing.key_to_index(into.key()),
            )
            .into_iter()
            .map(move |(indices, _)| {
                let ends = indices.into_values();
                self.edge_id(indexing.index_to_key(ends.0), indexing.index_to_key(ends.1))
            })
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(into);
        let into_key = into.key();
        self.adjacency
            .entries_in_col(self.nodes.indexing().key_to_index(into.key()))
            .map(move |(from, _)| self.edge_id(self.nodes.indexing().index_to_key(from), into_key))
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(from);
        let from_key = from.key();
        self.adjacency
            .entries_in_row(self.nodes.indexing().key_to_index(from.key()))
            .map(move |(into, _)| self.edge_id(from_key, self.nodes.indexing().index_to_key(into)))
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
        let (source, target) = id.keys().into_values();
        let indexing = self.nodes.indexing();
        if self
            .adjacency
            .get(indexing.key_to_index(source), indexing.key_to_index(target))
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

impl<N, E, D, S> Default for AdjacencyGraph<N, E, D, S>
where
    D: DirectednessTrait + Default,
    S: Storage,
    (D, S): AdjacencyMatrixSelector<usize, E>,
{
    fn default() -> Self {
        Self {
            nodes: NodeVec::default(),
            adjacency: SelectMatrix::<D, S, usize, E>::new(),
            directedness: PhantomData,
            compaction_count: S::CompactionCount::default(),
            id: GraphId::new(),
        }
    }
}

impl<N, E, D, S> GraphMut for AdjacencyGraph<N, E, D, S>
where
    D: DirectednessTrait + Default,
    S: Storage,
    (D, S): AdjacencyMatrixSelector<usize, E>,
{
    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let index = self.nodes.insert(data);
        self.node_id(index)
    }

    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> AddEdgeResult<Self::EdgeId, Self::EdgeData> {
        match self.adjacency.insert(
            self.nodes.indexing().key_to_index(from.key()),
            self.nodes.indexing().key_to_index(into.key()),
            data,
        ) {
            Some(data) => AddEdgeResult::Updated(data),
            None => AddEdgeResult::Added(self.edge_id(from.key(), into.key())),
        }
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData {
        let row_col = self.nodes.indexing().key_to_index(id.key());
        self.adjacency.clear_row_and_column(row_col, row_col);
        self.nodes.remove(id.key()).expect("invalid node ID")
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData {
        let (source, target) = id.keys().into_values();
        let indexing = self.nodes.indexing();
        self.adjacency
            .remove(indexing.key_to_index(source), indexing.key_to_index(target))
            .expect("Invalid edge ID")
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.adjacency.clear();
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
    D: DirectednessTrait + Default,
    S: Storage,
    (D, S): AdjacencyMatrixSelector<usize, E>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "AdjacencyGraph")
    }
}
