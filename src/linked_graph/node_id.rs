use std::{fmt::Debug, hash::Hash, sync::Weak};

use crate::{Graph, linked_graph::GraphId};

use super::Node;

/// Node identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the node data and a graph ID for safety checks.
pub struct NodeId<G: Graph> {
    pub(super) ptr: Weak<Node<G>>,
    pub(super) graph_id: GraphId,
}

// SAFETY: See comment on EdgeId.
unsafe impl<G: Graph> Send for NodeId<G> {}
unsafe impl<G: Graph> Sync for NodeId<G> {}

impl<G: Graph> Debug for NodeId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.ptr.as_ptr())
    }
}

impl<G: Graph> Clone for NodeId<G> {
    fn clone(&self) -> Self {
        NodeId {
            ptr: Weak::clone(&self.ptr),
            graph_id: self.graph_id,
        }
    }
}

impl<G: Graph> PartialEq for NodeId<G> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr() && self.graph_id == other.graph_id
    }
}

impl<G: Graph> Eq for NodeId<G> {}

impl<G: Graph> Hash for NodeId<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.as_ptr().hash(state);
        self.graph_id.hash(state);
    }
}

impl<G: Graph> PartialOrd for NodeId<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<G: Graph> Ord for NodeId<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<G: Graph> crate::graph_traits::NodeIdTrait for NodeId<G> {}
