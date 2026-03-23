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
    IndexedAutomap as DefaultAutomap, IndexedAutomapIndexing as DefaultAutomapIndexing,
    IndexedAutomapKey as DefaultAutomapKey,
};
#[cfg(feature = "bitvec")]
pub use offset::{
    OffsetAutomap as DefaultAutomap, OffsetAutomapIndexing as DefaultAutomapIndexing,
    OffsetAutomapKey as DefaultAutomapKey,
};
pub use trait_def::Automap;
