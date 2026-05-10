use std::fmt::Debug;

use derivative::Derivative;

/// Node and edge ID types for adjacency graphs.
use crate::{
    adjacency_graph::{
        EdgeId, NodeId,
        edge_container::{EdgeContainer, EdgeContainerSelector},
    },
    adjacency_matrix::{AdjacencyMatrix, HashStorage, Storage},
    copier::GraphCopier,
    directedness::Directedness,
    end_pair::EndPair,
    format_debug::format_debug,
    prelude::*,
};

use crate::bag::{Bag, BagKey};

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
#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct AdjacencyGraph<N, E, D = Directed, M = SingleEdge, S = HashStorage>
where
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    nodes: Bag<N>,
    adjacency: S::Matrix<M::Container<E>, D>,
    num_edges: usize,
    compaction_count: S::CompactionCount,
}

impl<N, E, D, M, S> AdjacencyGraph<N, E, D, M, S>
where
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    /// Creates a `NodeId` for the given `BagKey`.
    fn node_id(&self, key: BagKey) -> NodeId<S> {
        NodeId::new(key, self.compaction_count)
    }

    /// Creates an `EdgeId` for the given `BagKey` pair.
    fn edge_id(
        &self,
        from: BagKey,
        into: BagKey,
        index: <M::Container<E> as EdgeContainer<E>>::Index,
    ) -> EdgeId<E, S, D, M> {
        EdgeId::new(D::make_pair(from, into), index, self.compaction_count)
    }
}

impl<N, E, D, M, S> Graph for AdjacencyGraph<N, E, D, M, S>
where
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    type EdgeData = E;
    type EdgeId = EdgeId<E, S, D, M>;
    type NodeData = N;
    type NodeId = NodeId<S>;
    type Directedness = D;
    type EdgeMultiplicity = M;

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        self.nodes.get(id.key()).expect("no such node")
    }

    fn node_ids(&self) -> impl Iterator<Item = <Self as Graph>::NodeId> {
        self.nodes.keys().map(|key| self.node_id(key))
    }

    fn edge_data(&self, eid: &Self::EdgeId) -> &Self::EdgeData {
        let (from, to) = eid.keys().into_values();
        self.adjacency
            .get(from.to_index(), to.to_index())
            .expect("no such edge")
            .get(eid.index())
            .expect("no such edge index")
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.adjacency
            .iter()
            .flat_map(move |(from, into, container)| {
                container.iter().map(move |(index, _)| {
                    let from_key = BagKey::from_index(from);
                    let into_key = BagKey::from_index(into);
                    self.edge_id(from_key, into_key, index)
                })
            })
    }

    fn num_edges(&self) -> usize {
        self.num_edges
    }

    fn edge_ends(
        &self,
        id: &Self::EdgeId,
    ) -> <Self::Directedness as Directedness>::EndPair<Self::NodeId> {
        let (from, to) = id.keys().into_values();
        (self.node_id(from), self.node_id(to)).into()
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.adjacency
            .entry_at(from.key().to_index(), into.key().to_index())
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
        let into_key = into.key();
        self.adjacency
            .entries_in_col(into.key().to_index())
            .flat_map(move |(from, container)| {
                let from_key = BagKey::from_index(from);
                container
                    .iter()
                    .map(move |(index, _)| self.edge_id(from_key, into_key, index))
            })
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        let from_key = from.key();
        self.adjacency
            .entries_in_row(from.key().to_index())
            .flat_map(move |(into, container)| {
                let into_key = BagKey::from_index(into);
                container
                    .iter()
                    .map(move |(index, _)| self.edge_id(from_key, into_key, index))
            })
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
        GraphCopier::new(self).clone_nodes().clone_edges().copy()
    }
}

impl<N, E, D, M, S> GraphMut for AdjacencyGraph<N, E, D, M, S>
where
    D: Directedness,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn node_data_mut(&mut self, id: &Self::NodeId) -> &mut Self::NodeData {
        self.nodes.get_mut(id.key()).expect("no such node")
    }

    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData {
        let (from, to) = id.keys().into_values();
        self.adjacency
            .get_mut(from.to_index(), to.to_index())
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
    ) -> (Self::EdgeId, Option<(Self::EdgeId, Self::EdgeData)>) {
        let from_index = from.key().to_index();
        let into_index = into.key().to_index();
        let old_data = self.adjacency.remove(from_index, into_index);
        let (new_data, index, replaced) = EdgeContainer::append(old_data, data);
        self.adjacency.insert(from_index, into_index, new_data);
        let edge_id = self.edge_id(from.key(), into.key(), index);
        match replaced {
            Some(replaced) => (edge_id.clone(), Some((edge_id, replaced))),
            None => {
                self.num_edges += 1;
                (edge_id, None)
            }
        }
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData {
        let row_col = id.key().to_index();
        for (_col, container) in self.adjacency.entries_in_row(row_col) {
            self.num_edges -= container.len();
        }
        if Self::Directedness::IS_DIRECTED {
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
        let container = self
            .adjacency
            .remove(source.to_index(), target.to_index())
            .expect("Invalid edge ID");
        let (container, removed) = container.without(id.index());
        if let Some(container) = container {
            self.adjacency
                .insert(source.to_index(), target.to_index(), container);
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

    fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
        self.adjacency.shrink_to_fit();
    }
}

impl<N, E, D, M, S> Debug for AdjacencyGraph<N, E, D, M, S>
where
    N: Debug,
    E: Debug,
    D: Directedness + Default,
    M: EdgeContainerSelector,
    S: Storage,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "AdjacencyGraph")
    }
}
