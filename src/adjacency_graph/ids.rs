use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use derivative::Derivative;

use crate::{Directedness, Storage, graph_id::GraphId, id_vec::IdVecKey, pairs::Pair};

#[derive(Derivative)]
#[derivative(
    Clone(bound = "T: Clone"),
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
pub struct NodeIdOrEdgeId<S: Storage, T> {
    payload: T,
    pub compaction_count: S::CompactionCount,
    pub graph_id: GraphId,
}

impl<S: Storage, T> NodeIdOrEdgeId<S, T> {
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

pub type NodeId<S> = NodeIdOrEdgeId<S, IdVecKey>;

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct EdgeId<S: Storage, D: Directedness> {
    inner: NodeIdOrEdgeId<S, D::Pair<IdVecKey>>,
    _directedness: PhantomData<D>,
}

impl<S: Storage, D: Directedness> EdgeId<S, D> {
    pub fn new(
        payload: D::Pair<IdVecKey>,
        graph_id: GraphId,
        compaction_count: S::CompactionCount,
    ) -> Self {
        Self {
            inner: NodeIdOrEdgeId::new(payload, graph_id, compaction_count),
            _directedness: PhantomData,
        }
    }

    pub fn keys(&self) -> D::Pair<IdVecKey> {
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

impl<S: Storage> crate::graph::NodeId for NodeId<S> {}

impl<S: Storage> NodeId<S> {
    pub fn key(&self) -> IdVecKey {
        self.payload
    }
}

impl<S: Storage> Debug for NodeId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.payload)
    }
}

impl<S: Storage, D: Directedness> crate::graph::EdgeId for EdgeId<S, D> {
    type NodeId = NodeId<S>;
    type Directedness = D;

    fn ends(&self) -> D::Pair<NodeId<S>> {
        let (from_key, to_key) = self.inner.payload.clone().into();
        let from = NodeId::new(from_key, self.inner.graph_id, self.inner.compaction_count);
        let to = NodeId::new(to_key, self.inner.graph_id, self.inner.compaction_count);
        (from, to).into()
    }

    fn source(&self) -> NodeId<S> {
        NodeId::new(
            self.inner.payload.first().clone(),
            self.inner.graph_id,
            self.inner.compaction_count,
        )
    }

    fn target(&self) -> NodeId<S> {
        NodeId::new(
            self.inner.payload.second().clone(),
            self.inner.graph_id,
            self.inner.compaction_count,
        )
    }
}

impl<S: Storage, D: Directedness> Debug for EdgeId<S, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId{:?}", self.inner.payload)
    }
}
