use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use derivative::Derivative;

use crate::{
    DirectednessTrait, EdgeIdTrait, Storage, automap::OffsetAutomapKey,
    directedness::StaticDirectedness, edge_ends::EdgeEndsTrait as _, graph_id::GraphId,
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
    pub graph_id: GraphId,
}

impl<S: Storage, T: Clone> NodeIdOrEdgeId<S, T> {
    pub fn new(payload: T, graph_id: GraphId, compaction_count: S::CompactionCount) -> Self {
        Self {
            payload,
            compaction_count,
            graph_id,
        }
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.compaction_count = compaction_count;
        self
    }
}

pub type NodeId<S> = NodeIdOrEdgeId<S, OffsetAutomapKey>;

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct EdgeId<S: Storage, D: DirectednessTrait> {
    inner: NodeIdOrEdgeId<S, D::EdgeEnds<OffsetAutomapKey>>,
    _directedness: PhantomData<D>,
}

impl<S: Storage, D: DirectednessTrait> EdgeId<S, D> {
    pub fn new(
        payload: D::EdgeEnds<OffsetAutomapKey>,
        graph_id: GraphId,
        compaction_count: S::CompactionCount,
    ) -> Self {
        Self {
            inner: NodeIdOrEdgeId::new(payload, graph_id, compaction_count),
            _directedness: PhantomData,
        }
    }

    pub fn keys(&self) -> D::EdgeEnds<OffsetAutomapKey> {
        self.inner.payload.clone()
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.inner = self.inner.with_compaction_count(compaction_count);
        self
    }

    pub fn compaction_count(&self) -> S::CompactionCount {
        self.inner.compaction_count
    }

    pub fn graph_id(&self) -> GraphId {
        self.inner.graph_id
    }
}

impl<S: Storage> crate::graph::NodeIdTrait for NodeId<S> {}

impl<S: Storage> NodeId<S> {
    pub fn key(&self) -> OffsetAutomapKey {
        self.payload
    }
}

impl<S: Storage> Debug for NodeId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.payload)
    }
}

impl<S: Storage, D: StaticDirectedness> EdgeIdTrait for EdgeId<S, D> {
    type NodeId = NodeId<S>;
    type Directedness = D;

    fn directedness(&self) -> Self::Directedness {
        D::default()
    }

    fn ends(&self) -> D::EdgeEnds<NodeId<S>> {
        let (from_key, to_key) = self.inner.payload.values();
        let from = NodeId::new(*from_key, self.inner.graph_id, self.inner.compaction_count);
        let to = NodeId::new(*to_key, self.inner.graph_id, self.inner.compaction_count);
        self.directedness().make_pair(from, to)
    }

    fn source(&self) -> NodeId<S> {
        NodeId::new(
            *self.inner.payload.source(),
            self.inner.graph_id,
            self.inner.compaction_count,
        )
    }

    fn target(&self) -> NodeId<S> {
        NodeId::new(
            *self.inner.payload.target(),
            self.inner.graph_id,
            self.inner.compaction_count,
        )
    }
}

impl<S: Storage, D: DirectednessTrait> Debug for EdgeId<S, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId{:?}", self.inner.payload.values())
    }
}
