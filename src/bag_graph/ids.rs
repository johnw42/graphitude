use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use derivative::Derivative;

use crate::{Graph, GraphElementId, bag::BagKey};

/// Node identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the node data and a graph ID for safety checks.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct BagGraphNodeId<G: Graph> {
    key: BagKey,
    graph: PhantomData<G>,
}

impl<G: Graph> BagGraphNodeId<G> {
    pub(super) fn new(key: BagKey) -> Self {
        Self {
            key,
            graph: PhantomData,
        }
    }

    pub(super) fn key(&self) -> BagKey {
        self.key
    }
}

// SAFETY: This is safe because the BagGraphNodeId only contains a BagKey and PhantomData.
unsafe impl<G: Graph> Send for BagGraphNodeId<G> {}
unsafe impl<G: Graph> Sync for BagGraphNodeId<G> {}

impl<G: Graph> crate::graph_traits::GraphElementId for BagGraphNodeId<G> {}

impl<G: Graph> Debug for BagGraphNodeId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.key)
    }
}

/// Edge identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the edge data and a graph ID for safety checks.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct BagGraphEdgeId<G: Graph> {
    pub(super) key: BagKey,
    pub(super) phantom: PhantomData<G>,
}

impl<G: Graph> BagGraphEdgeId<G> {
    pub(super) fn new(key: BagKey) -> Self {
        Self {
            key,
            phantom: PhantomData,
        }
    }

    pub(super) fn key(&self) -> BagKey {
        self.key
    }
}

// SAFETY: This is safe because the BagGraphEdgeId only contains a BagKey and PhantomData.
unsafe impl<G> Send for BagGraphEdgeId<G> where G: Graph {}
unsafe impl<G> Sync for BagGraphEdgeId<G> where G: Graph {}

impl<G: Graph> GraphElementId for BagGraphEdgeId<G> {}

impl<G: Graph> Debug for BagGraphEdgeId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.key)
    }
}
