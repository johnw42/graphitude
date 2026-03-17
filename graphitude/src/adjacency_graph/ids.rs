use std::{fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{
    DirectednessTrait, EdgeIdTrait, NodeIdTrait, Storage,
    adjacency_graph::edge_container::{EdgeContainer, EdgeContainerSelector},
    automap::AutomapKey,
    coordinate_pair::CoordinatePair,
    graph_id::GraphIdClone,
};

// Comparing the graph_id and compaction_count is unfortunate, because
// it changes the semantics of equality based on whether error checking
// is enabled.  Ideally, we'd like to just assert they're equal,
// but that would break the way hash data structures work.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ValidationData<S: Storage> {
    pub compaction_count: S::CompactionCount,
    pub graph_id: GraphIdClone,
}

impl<S: Storage> ValidationData<S> {
    pub fn new(graph_id: GraphIdClone, compaction_count: S::CompactionCount) -> Self {
        Self {
            compaction_count,
            graph_id,
        }
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.compaction_count = compaction_count;
        self
    }
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeId<S: Storage> {
    validation: ValidationData<S>,
    key: AutomapKey,
}

impl<S: Storage> NodeId<S> {
    pub fn new(validation: ValidationData<S>, key: AutomapKey) -> Self {
        Self { validation, key }
    }

    pub fn validation(&self) -> &ValidationData<S> {
        &self.validation
    }

    pub fn key(&self) -> AutomapKey {
        self.key
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.validation = self.validation.with_compaction_count(compaction_count);
        self
    }
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct EdgeId<E, S, D, M>
where
    S: Storage,
    D: DirectednessTrait + Default,
    M: EdgeContainerSelector,
{
    validation: ValidationData<S>,
    key: CoordinatePair<AutomapKey, D>,
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
        validation: ValidationData<S>,
        key: CoordinatePair<AutomapKey, D>,
        index: <M::Container<E> as EdgeContainer<E>>::Index,
    ) -> Self {
        Self {
            validation,
            key,
            index,
            directedness: D::default(),
            edge_multiplicity: M::default(),
        }
    }

    pub fn validation(&self) -> &ValidationData<S> {
        &self.validation
    }

    pub fn keys(&self) -> CoordinatePair<AutomapKey, D> {
        self.key.clone()
    }

    pub fn with_compaction_count(mut self, compaction_count: S::CompactionCount) -> Self {
        self.validation = self.validation.with_compaction_count(compaction_count);
        self
    }

    pub fn index(&self) -> <M::Container<E> as EdgeContainer<E>>::Index {
        self.index
    }
}

impl<S: Storage> NodeIdTrait for NodeId<S> {}

impl<S: Storage> Debug for NodeId<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.key)
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

    fn left(&self) -> NodeId<S> {
        NodeId::new(self.validation.clone(), *self.key.first())
    }

    fn right(&self) -> NodeId<S> {
        NodeId::new(self.validation.clone(), *self.key.second())
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
        write!(
            f,
            "EdgeId({:?}, {:?}, {:?})",
            from, into, self.validation.graph_id
        )
    }
}
