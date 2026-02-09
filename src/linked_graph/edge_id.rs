use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{Arc, Weak},
};

use crate::{directedness::DirectednessTrait, graph_id::GraphId, pairs::Pair};

use super::{Edge, NodeId};

/// Edge identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the edge data and a graph ID for safety checks.
pub struct EdgeId<N, E, D: DirectednessTrait> {
    pub(super) ptr: Weak<Edge<N, E, D>>,
    pub(super) graph_id: GraphId,
    pub(super) directedness: PhantomData<D>,
}

impl<N, E, D: DirectednessTrait> Debug for EdgeId<N, E, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.ptr)
    }
}

impl<N, E, D: DirectednessTrait> Clone for EdgeId<N, E, D> {
    fn clone(&self) -> Self {
        EdgeId {
            ptr: Weak::clone(&self.ptr),
            graph_id: self.graph_id,
            directedness: PhantomData,
        }
    }
}

impl<N, E, D: DirectednessTrait> PartialEq for EdgeId<N, E, D> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<N, E, D: DirectednessTrait> Eq for EdgeId<N, E, D> {}

impl<N, E, D: DirectednessTrait> Hash for EdgeId<N, E, D> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.ptr.as_ptr() as usize).hash(state);
    }
}

impl<N, E, D: DirectednessTrait> PartialOrd for EdgeId<N, E, D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<N, E, D: DirectednessTrait> Ord for EdgeId<N, E, D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<N, E, D: DirectednessTrait> crate::graph::EdgeIdTrait for EdgeId<N, E, D> {
    type NodeId = NodeId<N, E, D>;
    type Directedness = D;

    fn source(&self) -> NodeId<N, E, D> {
        self.ptr
            .upgrade()
            .map(|edge| NodeId {
                ptr: Arc::downgrade(
                    &edge
                        .ends
                        .first()
                        .ptr
                        .upgrade()
                        .expect("Source node dangling"),
                ),
                graph_id: self.graph_id,
                directedness: PhantomData,
            })
            .expect("EdgeId is dangling")
    }

    fn target(&self) -> NodeId<N, E, D> {
        self.ptr
            .upgrade()
            .map(|edge| NodeId {
                ptr: Arc::downgrade(
                    &edge
                        .ends
                        .second()
                        .ptr
                        .upgrade()
                        .expect("Target node dangling"),
                ),
                graph_id: self.graph_id,
                directedness: PhantomData,
            })
            .expect("EdgeId is dangling")
    }
}
