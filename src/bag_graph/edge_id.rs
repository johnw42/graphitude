use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use derivative::Derivative;

use crate::{GraphElementId, Graph, bag::BagKey};

/// Edge identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the edge data and a graph ID for safety checks.
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct BagGraphEdgeId<G: Graph> {
    pub(super) edge_key: BagKey,
    pub(super) phantom: PhantomData<G>,
}

unsafe impl<G> Send for BagGraphEdgeId<G> where G: Graph {}
unsafe impl<G> Sync for BagGraphEdgeId<G> where G: Graph {}

impl<G: Graph> Debug for BagGraphEdgeId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.edge_key)
    }
}

impl<G: Graph> PartialEq for BagGraphEdgeId<G> {
    fn eq(&self, other: &Self) -> bool {
        self.edge_key == other.edge_key
    }
}

impl<G: Graph> Eq for BagGraphEdgeId<G> {}

impl<G: Graph> Hash for BagGraphEdgeId<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.edge_key.hash(state);
    }
}

impl<G: Graph> PartialOrd for BagGraphEdgeId<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<G: Graph> Ord for BagGraphEdgeId<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.edge_key.cmp(&other.edge_key)
    }
}

impl<G: Graph> GraphElementId for BagGraphEdgeId<G> {}
