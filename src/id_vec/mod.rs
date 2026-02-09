//! Stable-key vector implementations with multiple backends.
#![allow(unused)]

pub mod indexed;
#[cfg(feature = "bitvec")]
pub mod offset;
pub mod trait_def;

// Re-export commonly used types
pub use indexed::IndexedIdVec;
#[cfg(feature = "bitvec")]
pub use offset::{IdVecIndexing, IdVecKey, OffsetIdVec};
pub use trait_def::IdVec;
