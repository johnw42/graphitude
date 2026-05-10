use std::{
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Weak},
};

use crate::{Graph, linked_graph::GraphId};

use super::Node;

/// Node identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the node data and a graph ID for safety checks.
pub struct LinkedGraphNodeId<G: Graph> {
    ptr: Weak<Node<G>>,
    graph_id: GraphId,
}

impl<G: Graph> LinkedGraphNodeId<G> {
    pub(super) fn new(ptr: &Arc<Node<G>>, graph_id: GraphId) -> Self {
        Self {
            ptr: Arc::downgrade(ptr),
            graph_id,
        }
    }

    pub(super) fn as_ptr(&self) -> *const Node<G> {
        self.ptr.as_ptr()
    }

    pub(super) fn upgrade(&self, graph_id: GraphId) -> Arc<Node<G>> {
        assert_eq!(
            self.graph_id, graph_id,
            "NodeId does not belong to this graph"
        );

        self.ptr.upgrade().expect("NodeId is dangling")
    }
}

// SAFETY: See comment on EdgeId.
unsafe impl<G: Graph> Send for LinkedGraphNodeId<G> {}
unsafe impl<G: Graph> Sync for LinkedGraphNodeId<G> {}

impl<G: Graph> Debug for LinkedGraphNodeId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.ptr.as_ptr())
    }
}

impl<G: Graph> Clone for LinkedGraphNodeId<G> {
    fn clone(&self) -> Self {
        LinkedGraphNodeId {
            ptr: Weak::clone(&self.ptr),
            graph_id: self.graph_id,
        }
    }
}

impl<G: Graph> PartialEq for LinkedGraphNodeId<G> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr() && self.graph_id == other.graph_id
    }
}

impl<G: Graph> Eq for LinkedGraphNodeId<G> {}

impl<G: Graph> Hash for LinkedGraphNodeId<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.ptr.as_ptr().hash(state);
        self.graph_id.hash(state);
    }
}

impl<G: Graph> PartialOrd for LinkedGraphNodeId<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<G: Graph> Ord for LinkedGraphNodeId<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<G: Graph> crate::graph_traits::GraphElementId for LinkedGraphNodeId<G> {}
