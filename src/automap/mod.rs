//! Stable-key vector implementations with multiple backends.
#![allow(unused)]

pub mod indexed;
#[cfg(feature = "bitvec")]
pub mod offset;
pub mod trait_def;

// Re-export commonly used types
pub use indexed::IndexedAutomap;
#[cfg(not(feature = "bitvec"))]
pub use indexed::{
    IndexedAutomap as Automap, IndexedAutomapIndexing as AutomapIndexing,
    IndexedAutomapKey as AutomapKey,
};
#[cfg(feature = "bitvec")]
pub use offset::{
    OffsetAutomap as Automap, OffsetAutomapIndexing as AutomapIndexing,
    OffsetAutomapKey as AutomapKey,
};
pub use trait_def::AutomapTrait as AutomapTrait;
