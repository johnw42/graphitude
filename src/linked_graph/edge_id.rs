use std::{fmt::Debug, hash::Hash, sync::Weak};

use derivative::Derivative;

use crate::{GraphElementId, Graph, linked_graph::GraphId};

use super::Edge;

/// Edge identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the edge data and a graph ID for safety checks.
#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
pub struct LinkedGraphEdgeId<G: Graph> {
    pub(super) ptr: Weak<Edge<G>>,
    pub(super) graph_id: GraphId,
}

// SAFETY: EdgeId is Send and Sync because it only contains a Weak pointer and
// PhantomData, and does not allow mutation of the underlying data. The EdgeId
// can only be used to access the edge data through Graph methods that ensure
// the graph is still valid, so it cannot be used after the graph has been
// dropped.
unsafe impl<G: Graph> Send for LinkedGraphEdgeId<G> {}
unsafe impl<G: Graph> Sync for LinkedGraphEdgeId<G> {}

impl<G: Graph> Debug for LinkedGraphEdgeId<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?}, {:?})", self.ptr, self.graph_id)
    }
}

impl<G: Graph> PartialEq for LinkedGraphEdgeId<G> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<G: Graph> Eq for LinkedGraphEdgeId<G> {}

impl<G: Graph> Hash for LinkedGraphEdgeId<G> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.ptr.as_ptr()).hash(state);
    }
}

impl<G: Graph> PartialOrd for LinkedGraphEdgeId<G> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<G: Graph> Ord for LinkedGraphEdgeId<G> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<G: Graph> GraphElementId for LinkedGraphEdgeId<G> {}
