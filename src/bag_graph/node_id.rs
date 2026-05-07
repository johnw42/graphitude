use crate::{Graph, bag::BagKey};
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

/// Node identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the node data and a graph ID for safety checks.
pub struct NodeId<G: Graph> {
    pub(super) key: BagKey,
    pub(super) graph: PhantomData<G>,
}

// SAFETY: See comment on EdgeId.
unsafe impl<G: Graph> Send for NodeId<G> {}
unsafe impl<G: Graph> Sync for NodeId<G> {}

impl<G: Graph> Debug for NodeId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.key)
    }
}

impl<G: Graph> Clone for NodeId<G> {
    fn clone(&self) -> Self {
        NodeId {
            key: self.key,
            graph: PhantomData,
        }
    }
}

impl<G: Graph> PartialEq for NodeId<G> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<G: Graph> Eq for NodeId<G> {}

impl<G: Graph> Hash for NodeId<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<G: Graph> PartialOrd for NodeId<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<G: Graph> Ord for NodeId<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl<G: Graph> crate::graph_traits::NodeIdTrait for NodeId<G> {}
