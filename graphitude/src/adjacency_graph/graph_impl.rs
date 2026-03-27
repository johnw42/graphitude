use std::collections::HashMap;

use crate::{
    adjacency_graph::{
        InnerEdgeId, InnerNodeId,
        edge_container::{EdgeContainer, EdgeContainerSelector},
    },
    adjacency_matrix::{AdjacencyMatrix, CompactionCount as _, HashStorage, Storage},
    automap::trait_def::{Automap as _, AutomapIndexing as _},
    prelude::*,
};

use super::ids::Validated;

use crate::automap::{DefaultAutomap, DefaultAutomapKey};

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
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    nodes: DefaultAutomap<N>,
    adjacency: S::Matrix<M::Container<E>, D>,
    num_edges: usize,
    directedness: D,
    edge_multiplicity: M,
    compaction_count: S::CompactionCount,
}

impl<N, E, D, M, S> AdjacencyGraph<N, E, D, M, S>
where
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    /// Creates a `NodeId` for the given `AutomapKey`.
    fn node_id(&self, key: DefaultAutomapKey) -> Validated<InnerNodeId, S> {
        Validated::new(key, self.compaction_count)
    }

    /// Creates an `EdgeId` for the given `AutomapKey` pair.
    fn edge_id(
        &self,
        from: DefaultAutomapKey,
        into: DefaultAutomapKey,
        index: <M::Container<E> as EdgeContainer<E>>::Index,
    ) -> Validated<InnerEdgeId<E, D, M>, S> {
        Validated::new(
            InnerEdgeId::new(self.directedness.end_pair((from, into)), index),
            self.compaction_count,
        )
    }
}

impl<N, E, D, M, S> GraphImpl for AdjacencyGraph<N, E, D, M, S>
where
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    type EdgeData = E;
    type EdgeId = Validated<InnerEdgeId<E, D, M>, S>;
    type NodeData = N;
    type NodeId = Validated<InnerNodeId, S>;
    type Directedness = D;
    type EdgeMultiplicity = M;

    fn directedness(&self) -> Self::Directedness {
        self.directedness
    }

    fn edge_multiplicity(&self) -> Self::EdgeMultiplicity {
        self.edge_multiplicity
    }

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        self.nodes
            .get(*id.validate(self.compaction_count))
            .expect("no such node")
    }

    fn nodes(&self) -> impl Iterator<Item = <Self as GraphImpl>::NodeId> {
        self.nodes.iter_keys().map(|key| self.node_id(key))
    }

    fn edge_data(&self, eid: &Self::EdgeId) -> &Self::EdgeData {
        let valid_eid = eid.validate(self.compaction_count);
        let (from, to) = valid_eid.ends().into_values();
        self.adjacency
            .get(
                self.nodes.indexing().key_to_index(from),
                self.nodes.indexing().key_to_index(to),
            )
            .expect("no such edge")
            .get(valid_eid.index())
            .expect("no such edge index")
    }

    fn edges(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
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
        let valid_from = from.validate(self.compaction_count);
        let valid_into = into.validate(self.compaction_count);
        let indexing = self.nodes.indexing();
        self.adjacency
            .entry_at(
                indexing.key_to_index(*valid_from),
                indexing.key_to_index(*valid_into),
            )
            .into_iter()
            .flat_map(move |(_, container)| {
                container
                    .iter()
                    .map(move |(index, _)| self.edge_id(*valid_from, *valid_into, index))
            })
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        let valid_into = into.validate(self.compaction_count);
        self.adjacency
            .entries_in_col(self.nodes.indexing().key_to_index(*valid_into))
            .flat_map(move |(from, container)| {
                let from_key = self.nodes.indexing().index_to_key(from);
                container
                    .iter()
                    .map(move |(index, _)| self.edge_id(from_key, *valid_into, index))
            })
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        let valid_from = from.validate(self.compaction_count);
        self.adjacency
            .entries_in_row(self.nodes.indexing().key_to_index(*valid_from))
            .flat_map(move |(into, container)| {
                let into_key = self.nodes.indexing().index_to_key(into);
                container
                    .iter()
                    .map(move |(index, _)| self.edge_id(*valid_from, into_key, index))
            })
    }
}

impl<N, E, D, M, S> Default for AdjacencyGraph<N, E, D, M, S>
where
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn default() -> Self {
        Self::new(D::default(), M::default())
    }
}

impl<N, E, D, M, S> GraphImplMut for AdjacencyGraph<N, E, D, M, S>
where
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn new(directedness: D, edge_multiplicity: M) -> Self {
        Self {
            nodes: DefaultAutomap::default(),
            adjacency: S::Matrix::default(),
            num_edges: 0,
            directedness,
            edge_multiplicity,
            compaction_count: S::CompactionCount::default(),
        }
    }

    fn node_data_mut(&mut self, id: &Self::NodeId) -> &mut Self::NodeData {
        let valid_id = id.validate(self.compaction_count);
        self.nodes.get_mut(*valid_id).expect("no such node")
    }

    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData {
        let valid_id = id.validate(self.compaction_count);
        let (from, to) = valid_id.ends().into_values();
        self.adjacency
            .get_mut(
                self.nodes.indexing().key_to_index(from),
                self.nodes.indexing().key_to_index(to),
            )
            .expect("no such edge")
            .get_mut(valid_id.index())
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
        let valid_from = from.validate(self.compaction_count);
        let valid_into = into.validate(self.compaction_count);
        let from_index = self.nodes.indexing().key_to_index(*valid_from);
        let into_index = self.nodes.indexing().key_to_index(*valid_into);
        let old_data = self.adjacency.remove(from_index, into_index);
        let (new_data, index, replaced) = EdgeContainer::new(old_data, data);
        self.adjacency.insert(from_index, into_index, new_data);
        let edge_id = self.edge_id(*valid_from, *valid_into, index);
        match replaced {
            Some(replaced) => AddEdgeResult::Updated(edge_id, replaced),
            None => {
                self.num_edges += 1;
                AddEdgeResult::Added(edge_id)
            }
        }
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData {
        let valid_id = id.validate(self.compaction_count);
        let row_col = self.nodes.indexing().key_to_index(*valid_id);
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
        self.nodes.remove(*valid_id).expect("invalid node ID")
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData {
        let valid_id = id.validate(self.compaction_count);
        let (source, target) = valid_id.ends().into_values();
        let indexing = self.nodes.indexing();
        let container = self
            .adjacency
            .remove(indexing.key_to_index(source), indexing.key_to_index(target))
            .expect("Invalid edge ID");
        let (container, removed) = container.without(valid_id.index());
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

    fn compact_with(
        &mut self,
        mut node_id_callback: impl FnMut(&Self::NodeId, &Self::NodeId),
        mut edge_id_callback: impl FnMut(&Self::EdgeId, &Self::EdgeId),
    ) {
        let old_compaction_count = self.compaction_count;
        let new_compaction_count = self.compaction_count.increment();

        // Compact nodes and build ID mapping.
        let mut automap_map: HashMap<DefaultAutomapKey, DefaultAutomapKey> =
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
                        .edge_id(old_from, old_into, index)
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

impl<N, E, D, M, S> Clone for AdjacencyGraph<N, E, D, M, S>
where
    N: Clone,
    E: Clone,
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn clone(&self) -> Self {
        GraphCopier::clone(self)
    }
}
