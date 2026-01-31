use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use derivative::Derivative;

use crate::{Directedness, Storage, graph_id::GraphId, id_vec::IdVecKey};

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    Eq(bound = "T: Eq"),
    Hash(bound = "T: Hash"),
    PartialOrd(bound = "T: Ord"),
    Ord(bound = "T: Ord")
)]
pub struct NodeIdOrEdgeId<S: Storage, T: Copy> {
    payload: T,
    #[derivative(PartialOrd = "ignore", Ord = "ignore", Hash = "ignore")]
    pub compaction_count: S::CompactionCount,
    #[derivative(PartialOrd = "ignore", Ord = "ignore", Hash = "ignore")]
    pub graph_id: GraphId,
}

impl<S: Storage, T: Copy + Eq> PartialEq for NodeIdOrEdgeId<S, T> {
    fn eq(&self, other: &Self) -> bool {
        assert_eq!(self.graph_id, other.graph_id);
        self.payload == other.payload
    }
}

impl<S: Storage, T: Copy> NodeIdOrEdgeId<S, T> {
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
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct EdgeId<S: Storage, D: Directedness> {
    inner: NodeIdOrEdgeId<S, (IdVecKey, IdVecKey)>,
    _directedness: PhantomData<D>,
}

impl<S: Storage, D: Directedness> EdgeId<S, D> {
    pub fn new(
        payload: (IdVecKey, IdVecKey),
        graph_id: GraphId,
        compaction_count: S::CompactionCount,
    ) -> Self {
        Self {
            inner: NodeIdOrEdgeId::new(payload, graph_id, compaction_count),
            _directedness: PhantomData,
        }
    }

    pub fn keys(&self) -> (IdVecKey, IdVecKey) {
        self.inner.payload
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

    fn source(&self) -> NodeId<S> {
        NodeId::new(
            self.inner.payload.0,
            self.inner.graph_id,
            self.inner.compaction_count,
        )
    }

    fn target(&self) -> NodeId<S> {
        NodeId::new(
            self.inner.payload.1,
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
