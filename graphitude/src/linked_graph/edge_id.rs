use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    rc::{Rc, Weak},
};

use derivative::Derivative;

use crate::{
    EdgeIdImpl,
    directedness::Directedness,
    linked_graph::{NodeId, graph_impl::Edge},
};

/// Edge identifier for [`LinkedGraph`](super::LinkedGraph).
///
/// Contains a weak pointer to the edge data and a graph ID for safety checks.
#[derive(Derivative)]
#[derivative(Clone(bound = "D: Clone"))]
pub struct EdgeId<N, E, D: Directedness> {
    pub(super) ptr: Weak<Edge<N, E, D>>,
    pub(super) directedness: D,
}

// SAFETY: EdgeId is Send and Sync because it only contains a Weak pointer and
// PhantomData, and does not allow mutation of the underlying data. The EdgeId
// can only be used to access the edge data through Graph methods that ensure
// the graph is still valid, so it cannot be used after the graph has been
// dropped.
unsafe impl<N, E, D: Directedness> Send for EdgeId<N, E, D> {}
unsafe impl<N, E, D: Directedness> Sync for EdgeId<N, E, D> {}

impl<N, E, D: Directedness> Debug for EdgeId<N, E, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.ptr.as_ptr())
    }
}

impl<N, E, D: Directedness> PartialEq for EdgeId<N, E, D> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<N, E, D: Directedness> Eq for EdgeId<N, E, D> {}

impl<N, E, D: Directedness> Hash for EdgeId<N, E, D> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.ptr.as_ptr() as usize).hash(state);
    }
}

impl<N, E, D: Directedness> PartialOrd for EdgeId<N, E, D> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<N, E, D: Directedness> Ord for EdgeId<N, E, D> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<N, E, D: Directedness> EdgeIdImpl for EdgeId<N, E, D> {
    type NodeId = NodeId<N, E, D>;

    fn left(&self) -> NodeId<N, E, D> {
        self.ptr
            .upgrade()
            .map(|edge| NodeId {
                ptr: Rc::downgrade(
                    &edge
                        .ends
                        .left()
                        .ptr
                        .upgrade()
                        .expect("Source node dangling"),
                ),
                directedness: PhantomData,
            })
            .expect("EdgeId is dangling")
    }

    fn right(&self) -> NodeId<N, E, D> {
        self.ptr
            .upgrade()
            .map(|edge| NodeId {
                ptr: Rc::downgrade(
                    &edge
                        .ends
                        .right()
                        .ptr
                        .upgrade()
                        .expect("Target node dangling"),
                ),
                directedness: PhantomData,
            })
            .expect("EdgeId is dangling")
    }
}
