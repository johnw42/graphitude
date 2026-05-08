use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use derivative::Derivative;

use crate::{EdgeIdTrait, Graph, bag::BagKey, coordinate_pair::CoordinatePair};

use super::NodeId;

/// Edge identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the edge data and a graph ID for safety checks.
#[derive(Derivative)]
#[derivative(Clone(bound = "G::Directedness: Clone"))]
pub struct EdgeId<G: Graph> {
    pub(super) edge_key: BagKey,
    pub(super) node_keys: CoordinatePair<BagKey, G::Directedness>,
    pub(super) directedness: G::Directedness,
}

// SAFETY: EdgeId is Send and Sync because it only contains a Weak pointer and
// PhantomData, and does not allow mutation of the underlying data. The EdgeId
// can only be used to access the edge data through Graph methods that ensure
// the graph is still valid, so it cannot be used after the graph has been
// dropped.
unsafe impl<G: Graph> Send for EdgeId<G> {}
unsafe impl<G: Graph> Sync for EdgeId<G> {}

impl<G: Graph> Debug for EdgeId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.edge_key)
    }
}

impl<G: Graph> PartialEq for EdgeId<G> {
    fn eq(&self, other: &Self) -> bool {
        self.edge_key == other.edge_key
    }
}

impl<G: Graph> Eq for EdgeId<G> {}

impl<G: Graph> Hash for EdgeId<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.edge_key.hash(state);
    }
}

impl<G: Graph> PartialOrd for EdgeId<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<G: Graph> Ord for EdgeId<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.edge_key.cmp(&other.edge_key)
    }
}

impl<G: Graph> EdgeIdTrait for EdgeId<G> {
    type NodeId = NodeId<G>;
    type Directedness = G::Directedness;

    fn directedness(&self) -> Self::Directedness {
        self.directedness
    }
}
