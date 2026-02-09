//! Stable-key vector implementations with multiple backends.
#![allow(unused)]

pub mod indexed;
#[cfg(feature = "bitvec")]
pub mod offset;
pub mod trait_def;

// Re-export commonly used types
pub use indexed::IndexedAutomap;
#[cfg(not(feature = "bitvec"))]
pub use indexed::{IndexedAutomapIndexing as AutomapIndexing, IndexedAutomapKey};
#[cfg(feature = "bitvec")]
pub use offset::{OffsetAutomap, OffsetAutomapIndexing, OffsetAutomapKey};
pub use trait_def::Automap;
