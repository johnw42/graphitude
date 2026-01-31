use std::{fmt::Debug, hash::Hash, sync::Weak};

use crate::graph_id::GraphId;

use super::Node;

/// Node identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the node data and a graph ID for safety checks.
pub struct NodeId<N, E> {
    pub(super) ptr: Weak<Node<N, E>>,
    pub(super) graph_id: GraphId,
}

impl<N, E> Debug for NodeId<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.ptr)
    }
}

impl<N, E> Clone for NodeId<N, E> {
    fn clone(&self) -> Self {
        NodeId {
            ptr: Weak::clone(&self.ptr),
            graph_id: self.graph_id,
        }
    }
}

impl<N, E> PartialEq for NodeId<N, E> {
    fn eq(&self, other: &Self) -> bool {
        assert_eq!(self.graph_id, other.graph_id);
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<N, E> Eq for NodeId<N, E> {}

impl<N, E> Hash for NodeId<N, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.ptr.as_ptr() as usize).hash(state);
    }
}

impl<N, E> PartialOrd for NodeId<N, E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ptr.as_ptr().cmp(&other.ptr.as_ptr()))
    }
}

impl<N, E> Ord for NodeId<N, E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<N, E> crate::graph::NodeId for NodeId<N, E> {}
