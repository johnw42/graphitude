#[cfg(not(feature = "unchecked"))]
use std::sync::atomic::{AtomicUsize, Ordering};

/// A global graph identifier counter for paranoia mode. We assume no two graphs
/// will have the same identifier, and though it is technically possible for
/// this to overflow and wrap around, it is extremely unlikely in practice.
#[cfg(not(feature = "unchecked"))]
static GRAPH_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct GraphId(#[cfg(not(feature = "unchecked"))] usize);

impl GraphId {
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
