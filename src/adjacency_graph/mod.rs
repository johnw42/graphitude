use std::{collections::HashMap, fmt::Debug};

/// Node and edge ID types for adjacency graphs.
pub use self::ids::{EdgeId, NodeId};
use crate::{
    adjacency_graph::edge_container::{EdgeContainer, EdgeContainerSelector},
    adjacency_matrix::{AdjacencyMatrix, CompactionCount as _, HashStorage, Storage},
    automap::{Automap, trait_def::AutomapIndexing},
    copier::GraphCopier,
    directedness::DirectednessTrait,
    format_debug::format_debug,
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

#[doc(hidden)]
pub mod edge_container;
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
pub struct AdjacencyGraph<N, E, D = Directed, M = SingleEdge, S = HashStorage>
where
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    nodes: NodeVec<N>,
    adjacency: S::Matrix<M::Container<E>, D>,
    num_edges: usize,
    directedness: D,
    edge_multiplicity: M,
    id: GraphId,
    compaction_count: S::CompactionCount,
}

impl<N, E, D, M, S> AdjacencyGraph<N, E, D, M, S>
where
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    /// Creates a `NodeId` for the given `AutomapKey`.
    fn node_id(&self, key: OffsetAutomapKey) -> NodeId<S> {
        NodeId::new(key, self.id, self.compaction_count)
    }

    /// Creates an `EdgeId` for the given `AutomapKey` pair.
    fn edge_id(
        &self,
        from: OffsetAutomapKey,
        into: OffsetAutomapKey,
        index: <M::Container<E> as EdgeContainer<E>>::Index,
    ) -> EdgeId<E, S, D, M> {
        EdgeId::new(
            self.directedness.coordinate_pair((from, into)),
            index,
            self.id,
            self.compaction_count,
        )
    }
}

impl<N, E, D, M, S> Graph for AdjacencyGraph<N, E, D, M, S>
where
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    type EdgeData = E;
    type EdgeId = EdgeId<E, S, D, M>;
    type NodeData = N;
    type NodeId = NodeId<S>;
    type Directedness = D;
    type EdgeMultiplicity = M;

    fn directedness(&self) -> Self::Directedness {
        self.directedness
    }

    fn edge_multiplicity(&self) -> Self::EdgeMultiplicity {
        self.edge_multiplicity
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
            .get(eid.index())
            .expect("no such edge index")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .iter()
            .flat_map(move |(from, into, container)| {
                container.iter().map(move |(index, _)| {
                    let from_key = self.nodes.indexing().index_to_key(from);
                    let into_key = self.nodes.indexing().index_to_key(into);
                    self.edge_id(from_key, into_key, index)
                })
            })
    }

    fn num_edges(&self) -> usize {
        self.num_edges
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
            .flat_map(move |(_, container)| {
                container
                    .iter()
                    .map(move |(index, _)| self.edge_id(from.key(), into.key(), index))
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
            .flat_map(move |(from, container)| {
                let from_key = self.nodes.indexing().index_to_key(from);
                container
                    .iter()
                    .map(move |(index, _)| self.edge_id(from_key, into_key, index))
            })
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.assert_valid_node_id(from);
        let from_key = from.key();
        self.adjacency
            .entries_in_row(self.nodes.indexing().key_to_index(from.key()))
            .flat_map(move |(into, container)| {
                let into_key = self.nodes.indexing().index_to_key(into);
                container
                    .iter()
                    .map(move |(index, _)| self.edge_id(from_key, into_key, index))
            })
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

impl<N, E, D, M, S> Default for AdjacencyGraph<N, E, D, M, S>
where
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn default() -> Self {
        Self::new(D::default(), M::default())
    }
}

impl<N, E, D, M, S> Clone for AdjacencyGraph<N, E, D, M, S>
where
    N: Clone,
    E: Clone,
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn clone(&self) -> Self {
        GraphCopier::new(self).clone_nodes().clone_edges().copy()
    }
}

impl<N, E, D, M, S> GraphMut for AdjacencyGraph<N, E, D, M, S>
where
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn new(directedness: D, edge_multiplicity: M) -> Self {
        Self {
            nodes: NodeVec::default(),
            adjacency: S::Matrix::default(),
            num_edges: 0,
            directedness,
            edge_multiplicity,
            compaction_count: S::CompactionCount::default(),
            id: GraphId::default(),
        }
    }

    fn node_data_mut(&mut self, id: &Self::NodeId) -> &mut Self::NodeData {
        self.assert_valid_node_id(id);
        self.nodes.get_mut(id.key()).expect("no such node")
    }

    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData {
        self.assert_valid_edge_id(id);
        let (from, to) = id.keys().into_values();
        self.adjacency
            .get_mut(
                self.nodes.indexing().key_to_index(from),
                self.nodes.indexing().key_to_index(to),
            )
            .expect("no such edge")
            .get_mut(id.index())
            .expect("no such edge index")
    }

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
        self.assert_valid_node_id(from);
        self.assert_valid_node_id(into);

        let from_index = self.nodes.indexing().key_to_index(from.key());
        let into_index = self.nodes.indexing().key_to_index(into.key());
        let old_data = self.adjacency.remove(from_index, into_index);
        let (new_data, index, replaced) = EdgeContainer::append(old_data, data);
        self.adjacency.insert(from_index, into_index, new_data);
        let edge_id = self.edge_id(from.key(), into.key(), index);
        match replaced {
            Some(replaced) => AddEdgeResult::Updated(edge_id, replaced),
            None => {
                self.num_edges += 1;
                AddEdgeResult::Added(edge_id)
            }
        }
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData {
        let row_col = self.nodes.indexing().key_to_index(id.key());
        for (_col, container) in self.adjacency.entries_in_row(row_col) {
            self.num_edges -= container.len();
        }
        if self.directedness.is_directed() {
            for (row, container) in self.adjacency.entries_in_col(row_col) {
                if row != row_col {
                    self.num_edges -= container.len();
                }
            }
        }
        self.adjacency.clear_row_and_column(row_col, row_col);
        self.nodes.remove(id.key()).expect("invalid node ID")
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData {
        let (source, target) = id.keys().into_values();
        let indexing = self.nodes.indexing();
        let container = self
            .adjacency
            .remove(indexing.key_to_index(source), indexing.key_to_index(target))
            .expect("Invalid edge ID");
        let (container, removed) = container.without(id.index());
        if let Some(container) = container {
            self.adjacency.insert(
                indexing.key_to_index(source),
                indexing.key_to_index(target),
                container,
            );
        }
        self.num_edges -= 1;
        removed.expect("Invalid edge ID")
    }

    fn clear(&mut self) {
        self.num_edges = 0;
        self.nodes.clear();
        self.adjacency.clear();
    }

    fn reserve(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.nodes.reserve(additional_nodes);
        self.adjacency.reserve(additional_edges);
    }

    fn reserve_exact(&mut self, additional_nodes: usize, additional_edges: usize) {
        self.nodes.reserve_exact(additional_nodes);
        self.adjacency.reserve_exact(additional_edges);
    }

    fn compact(&mut self) {
        self.compact_with(|_, _| {}, |_, _| {});
    }

    fn compact_with(
        &mut self,
        mut node_id_callback: impl FnMut(&Self::NodeId, &Self::NodeId),
        mut edge_id_callback: impl FnMut(&Self::EdgeId, &Self::EdgeId),
    ) {
        let old_compaction_count = self.compaction_count;
        let new_compaction_count = self.compaction_count.increment();

        // Compact nodes and build ID mapping.
        let mut automap_map: HashMap<OffsetAutomapKey, OffsetAutomapKey> =
            HashMap::with_capacity(self.nodes.len());
        let old_indexing = self.nodes.indexing();
        self.nodes.compact_with(&mut |old_key, new_key_opt| {
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
            let mut old_adjacency = S::Matrix::<M::Container<E>, D>::default();
            std::mem::swap(&mut self.adjacency, &mut old_adjacency);
            self.adjacency.reserve(self.nodes.len());
            for (old_from_index, old_into_index, container) in old_adjacency.into_iter() {
                let old_from = old_indexing.index_to_key(old_from_index);
                let old_into = old_indexing.index_to_key(old_into_index);

                // Skip edges where either endpoint was removed
                let Some(&new_from) = automap_map.get(&old_from) else {
                    continue;
                };
                let Some(&new_into) = automap_map.get(&old_into) else {
                    continue;
                };

                for (index, _) in container.iter() {
                    let old_edge_id = self
                        .edge_id(old_from, old_into, index.clone())
                        .with_compaction_count(old_compaction_count);
                    let new_edge_id = self
                        .edge_id(new_from, new_into, index)
                        .with_compaction_count(new_compaction_count);
                    edge_id_callback(&old_edge_id, &new_edge_id);
                }

                self.adjacency.insert(
                    new_indexing.key_to_index(new_from),
                    new_indexing.key_to_index(new_into),
                    container,
                );
            }
        }

        self.compaction_count = new_compaction_count;
    }

    fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
        self.adjacency.shrink_to_fit();
    }
}

impl<N, E, D, M, S> Debug for AdjacencyGraph<N, E, D, M, S>
where
    N: Debug,
    E: Debug,
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "AdjacencyGraph")
    }
}
