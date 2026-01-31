use std::{fmt::Debug, hash::Hash, marker::PhantomData, sync::Weak};

use crate::graph_id::GraphId;

use super::Node;

/// Node identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the node data and a graph ID for safety checks.
pub struct NodeId<N, E, D> {
    pub(super) ptr: Weak<Node<N, E, D>>,
    pub(super) graph_id: GraphId,
    pub(super) directedness: PhantomData<D>,
}

impl<N, E, D> Debug for NodeId<N, E, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.ptr)
    }
}

impl<N, E, D> Clone for NodeId<N, E, D> {
    fn clone(&self) -> Self {
        NodeId {
            ptr: Weak::clone(&self.ptr),
            graph_id: self.graph_id,
            directedness: PhantomData,
        }
    }
}

impl<N, E, D> PartialEq for NodeId<N, E, D> {
    fn eq(&self, other: &Self) -> bool {
        assert_eq!(self.graph_id, other.graph_id);
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<N, E, D> Eq for NodeId<N, E, D> {}

impl<N, E, D> Hash for NodeId<N, E, D> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.ptr.as_ptr() as usize).hash(state);
    }
}

impl<N, E, D> PartialOrd for NodeId<N, E, D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ptr.as_ptr().cmp(&other.ptr.as_ptr()))
    }
}

impl<N, E, D> Ord for NodeId<N, E, D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<N, E, D> crate::graph::NodeId for NodeId<N, E, D> {}
