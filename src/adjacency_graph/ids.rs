use std::{fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{
    DirectednessTrait, EdgeIdTrait, NodeIdTrait, Storage,
    adjacency_graph::edge_container::{EdgeContainer, EdgeContainerSelector},
    bag::BagKey,
    coordinate_pair::CoordinatePair,
};

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Hash(bound = "T: Hash"),
    // Comparing the graph_id and compaction_count is unfortunate, because
    // it changes the semantics of equality based on whether error checking
    // is enabled.  Ideally, we'd like to just assert they're equal,
    // but that would break the way hash data structures work.
    PartialEq(bound = "T: PartialEq"),
    Eq(bound = "T: Eq"),
    PartialOrd(bound = "T: Ord"),
    Ord(bound = "T: Ord")
)]
pub struct NodeIdOrEdgeId<S: Storage, T: Clone> {
    payload: T,
    pub compaction_count: S::CompactionCount,
}

impl<S: Storage, T: Clone> NodeIdOrEdgeId<S, T> {
    pub fn new(payload: T, compaction_count: S::CompactionCount) -> Self {
        Self {
            payload,
            compaction_count,
        }
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.compaction_count = compaction_count;
        self
    }
}

pub type NodeId<S> = NodeIdOrEdgeId<S, BagKey>;

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct EdgeId<E, S, D, M>
where
    S: Storage,
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    inner: NodeIdOrEdgeId<S, CoordinatePair<BagKey, D>>,
    index: <M::Container<E> as EdgeContainer<E>>::Index,
    directedness: D,
    edge_multiplicity: M,
}

impl<E, S, D, M> EdgeId<E, S, D, M>
where
    S: Storage,
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    pub fn new(
        payload: CoordinatePair<BagKey, D>,
        index: <M::Container<E> as EdgeContainer<E>>::Index,
        compaction_count: S::CompactionCount,
    ) -> Self {
        Self {
            inner: NodeIdOrEdgeId::new(payload, compaction_count),
            index,
            directedness: D::default(),
            edge_multiplicity: M::default(),
        }
    }

    pub fn keys(&self) -> CoordinatePair<BagKey, D> {
        self.inner.payload.clone()
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.inner = self.inner.with_compaction_count(compaction_count);
        self
    }

    pub fn compaction_count(&self) -> S::CompactionCount {
        self.inner.compaction_count
    }

    pub fn index(&self) -> <M::Container<E> as EdgeContainer<E>>::Index {
        self.index.clone()
    }
}

impl<S: Storage> NodeIdTrait for NodeId<S> {}

impl<S: Storage> NodeId<S> {
    pub fn key(&self) -> BagKey {
        self.payload
    }
}

impl<S: Storage> Debug for NodeId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.payload)
    }
}

impl<E, S, D, M> EdgeIdTrait for EdgeId<E, S, D, M>
where
    S: Storage,
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    type NodeId = NodeId<S>;
    type Directedness = D;

    fn directedness(&self) -> Self::Directedness {
        self.directedness
    }
}

impl<E, S, D, M> Debug for EdgeId<E, S, D, M>
where
    S: Storage,
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (from, into) = self.keys().into_values();
        write!(f, "EdgeId({:?}, {:?})", from, into)
    }
}
