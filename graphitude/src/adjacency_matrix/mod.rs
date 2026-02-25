#[cfg(feature = "bitvec")]
pub mod bitvec;

pub mod hash;
pub mod trait_def;

mod storage;
mod tests;

#[cfg(feature = "bitvec")]
pub use storage::BitvecStorage;
pub use storage::{HashStorage, Storage};
pub use trait_def::AdjacencyMatrix;

pub(crate) use storage::CompactionCount;
