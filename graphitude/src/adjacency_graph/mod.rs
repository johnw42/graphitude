#[doc(hidden)]
pub mod edge_container;
mod graph_impl;
mod ids;

pub use self::{
    graph_impl::AdjacencyGraph,
    ids::{EdgeId, NodeId},
};
