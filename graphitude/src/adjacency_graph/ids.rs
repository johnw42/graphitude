use std::{fmt::Debug, hash::Hash, panic::panic_any};

use derivative::Derivative;

use crate::{
    DirectednessTrait, EdgeIdTrait, NodeIdTrait, Storage,
    adjacency_graph::edge_container::{EdgeContainer, EdgeContainerSelector},
    automap::AutomapKey,
    end_pair::EndPair,
    invalid_id::InvalidId,
};

// Comparing the graph_id and compaction_count is unfortunate, because
// it changes the semantics of equality based on whether error checking
// is enabled.  Ideally, we'd like to just assert they're equal,
// but that would break the way hash data structures work.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Validated<T, S: Storage> {
    data: T,
    compaction_count: S::CompactionCount,
}

impl<T, S: Storage> Validated<T, S> {
    pub fn new(data: T, compaction_count: S::CompactionCount) -> Self {
        Self {
            data,
            compaction_count,
        }
    }

    pub fn validate(&self, compaction_count: S::CompactionCount) -> &T {
        if self.compaction_count != compaction_count {
            panic_any(InvalidId);
        }
        &self.data
    }

    pub fn with_data(mut self, data: T) -> Self {
        self.data = data;
        self
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.compaction_count = compaction_count;
        self
    }
}

pub type InnerNodeId = AutomapKey;
impl<S: Storage> NodeIdTrait for Validated<InnerNodeId, S> {}

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct InnerEdgeId<E, D, M>
where
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    ends: EndPair<InnerNodeId, D>,
    index: <M::Container<E> as EdgeContainer<E>>::Index,
    directedness: D,
    edge_multiplicity: M,
}

impl<E, D, M> InnerEdgeId<E, D, M>
where
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    pub fn new(
        ends: EndPair<InnerNodeId, D>,
        index: <M::Container<E> as EdgeContainer<E>>::Index,
    ) -> Self {
        Self {
            ends,
            index,
            directedness: D::default(),
            edge_multiplicity: M::default(),
        }
    }

    pub fn ends(&self) -> EndPair<InnerNodeId, D> {
        self.ends.clone()
    }

    pub fn index(&self) -> <M::Container<E> as EdgeContainer<E>>::Index {
        self.index
    }
}

impl<E, D, M, S> EdgeIdTrait for Validated<InnerEdgeId<E, D, M>, S>
where
    S: Storage,
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    type NodeId = Validated<InnerNodeId, S>;
    type Directedness = D;

    fn into_ends(self) -> EndPair<Self::NodeId, Self::Directedness> {
        EndPair::new(self.left(), self.right(), self.directedness())
    }

    fn directedness(&self) -> Self::Directedness {
        self.data.directedness
    }

    fn left(&self) -> Self::NodeId {
        Validated::new(*self.data.ends.left(), self.compaction_count)
    }

    fn right(&self) -> Self::NodeId {
        Validated::new(*self.data.ends.right(), self.compaction_count)
    }
}

impl<E, D, M> Debug for InnerEdgeId<E, D, M>
where
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({:?}, {:?}, {:?})",
            self.ends.left(),
            self.ends.right(),
            self.index
        )
    }
}
