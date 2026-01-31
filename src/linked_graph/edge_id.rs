use std::{
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Weak},
};

use crate::{directedness::Directed, graph_id::GraphId};

use super::{Edge, NodeId};

/// Edge identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the edge data and a graph ID for safety checks.
pub struct EdgeId<N, E> {
    pub(super) ptr: Weak<Edge<N, E>>,
    pub(super) graph_id: GraphId,
}

impl<N, E> Debug for EdgeId<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.ptr)
    }
}

impl<N, E> Clone for EdgeId<N, E> {
    fn clone(&self) -> Self {
        EdgeId {
            ptr: Weak::clone(&self.ptr),
            graph_id: self.graph_id,
        }
    }
}

impl<N, E> PartialEq for EdgeId<N, E> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<N, E> Eq for EdgeId<N, E> {}

impl<N, E> Hash for EdgeId<N, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.ptr.as_ptr() as usize).hash(state);
    }
}

impl<N, E> PartialOrd for EdgeId<N, E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ptr.as_ptr().cmp(&other.ptr.as_ptr()))
    }
}

impl<N, E> Ord for EdgeId<N, E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<N: Debug, E> crate::graph::EdgeId for EdgeId<N, E> {
    type NodeId = NodeId<N, E>;
    type Directedness = Directed;

    fn source(&self) -> NodeId<N, E> {
        self.ptr
            .upgrade()
            .map(|edge| NodeId {
                ptr: Arc::downgrade(&edge.from.ptr.upgrade().expect("Source node dangling")),
                graph_id: self.graph_id,
            })
            .expect("EdgeId is dangling")
    }

    fn target(&self) -> NodeId<N, E> {
        self.ptr
            .upgrade()
            .map(|edge| NodeId {
                ptr: Arc::downgrade(&edge.into.ptr.upgrade().expect("Target node dangling")),
                graph_id: self.graph_id,
            })
            .expect("EdgeId is dangling")
    }
}
