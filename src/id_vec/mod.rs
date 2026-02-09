//! Stable-key vector implementations with multiple backends.
#![allow(unused)]

pub mod indexed;
#[cfg(feature = "bitvec")]
pub mod offset;
pub mod trait_def;

// Re-export commonly used types
pub use indexed::IndexedIdVec;
#[cfg(not(feature = "bitvec"))]
pub use indexed::{IdVecKey, IndexedIdVecIndexing as IdVecIndexing};
#[cfg(feature = "bitvec")]
pub use offset::{IdVecIndexing, IdVecKey, OffsetIdVec};
pub use trait_def::IdVec;
