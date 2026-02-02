#[cfg(not(feature = "unchecked"))]
use std::sync::atomic::{AtomicUsize, Ordering};

/// A global graph identifier counter for paranoia mode. We assume no two graphs
/// will have the same identifier, and though it is technically possible for
/// this to overflow and wrap around, it is extremely unlikely this will cause
/// problems in practice, if it occurs at all, because the only impact would be
/// that two different graphs might have the same GraphId, causing false
/// positives when checking the ownership of node/edge IDs.  
#[cfg(not(feature = "unchecked"))]
static GRAPH_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A unique identifier for a graph instance.
#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug, PartialOrd, Ord)]
pub struct GraphId(#[cfg(not(feature = "unchecked"))] usize);

impl GraphId {
    /// Create a new unique graph identifier.
    pub fn new() -> Self {
        #[cfg(feature = "unchecked")]
        {
            GraphId()
        }
        #[cfg(not(feature = "unchecked"))]
        {
            let id = GRAPH_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
            GraphId(id)
        }
    }
}
