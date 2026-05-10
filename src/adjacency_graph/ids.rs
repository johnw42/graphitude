use std::fmt::Debug;

use derivative::Derivative;

use crate::{
    Directedness, GraphElementId, Storage,
    adjacency_graph::edge_container::{EdgeContainer, EdgeContainerSelector},
    bag::BagKey,
    end_pair::EndPair,
};

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct AdjacencyGraphNodeId<S: Storage> {
    payload: BagKey,
    compaction_count: S::CompactionCount,
}

impl<S: Storage> AdjacencyGraphNodeId<S> {
    pub fn new(payload: BagKey, compaction_count: S::CompactionCount) -> Self {
        Self {
            payload,
            compaction_count,
        }
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.compaction_count = compaction_count;
        self
    }

    pub fn compaction_count(&self) -> S::CompactionCount {
        self.compaction_count
    }
}

impl<S: Storage> GraphElementId for AdjacencyGraphNodeId<S> {}

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct AdjacencyGraphEdgeId<E, S, D, M>
where
    S: Storage,
    D: Directedness,
    M: EdgeContainerSelector,
{
    ends: D::EndPair<BagKey>,
    index: <M::Container<E> as EdgeContainer<E>>::Index,
    compaction_count: S::CompactionCount,
}

impl<E, S, D, M> AdjacencyGraphEdgeId<E, S, D, M>
where
    S: Storage,
    D: Directedness + Default,
    M: EdgeContainerSelector,
{
    pub fn new(
        ends: D::EndPair<BagKey>,
        index: <M::Container<E> as EdgeContainer<E>>::Index,
        compaction_count: S::CompactionCount,
    ) -> Self {
        Self {
            ends,
            index,
            compaction_count,
        }
    }

    pub fn ends(&self) -> D::EndPair<BagKey> {
        self.ends.clone()
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.compaction_count = compaction_count;
        self
    }

    pub fn compaction_count(&self) -> S::CompactionCount {
        self.compaction_count
    }

    pub fn index(&self) -> <M::Container<E> as EdgeContainer<E>>::Index {
        self.index.clone()
    }
}

impl<S: Storage> AdjacencyGraphNodeId<S> {
    pub fn key(&self) -> BagKey {
        self.payload
    }
}

impl<S: Storage> Debug for AdjacencyGraphNodeId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.payload)
    }
}

impl<E, S, D, M> GraphElementId for AdjacencyGraphEdgeId<E, S, D, M>
where
    S: Storage,
    D: Directedness + Default,
    M: EdgeContainerSelector,
{
}

impl<E, S, D, M> Debug for AdjacencyGraphEdgeId<E, S, D, M>
where
    S: Storage,
    D: Directedness + Default,
    M: EdgeContainerSelector,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (from, into) = self.ends().into_values();
        write!(f, "EdgeId({:?}, {:?})", from, into)
    }
}
