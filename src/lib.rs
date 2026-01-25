pub mod adjacency_graph;
pub mod adjacency_matrix;
pub mod debug;
pub mod directedness;
pub mod edge_ref;
pub mod graph;
pub mod linked_graph;
pub mod object_graph;
pub mod search;
pub mod tests;
pub mod vertex_ref;

mod id_vec;
mod util;
mod symmetric_maxtrix_indexing;

pub use adjacency_matrix::{
    AdjacencyMatrix,
    bitvec::{asymmetric::AsymmetricBitvecAdjacencyMatrix, symmetric::SymmetricBitvecAdjacencyMatrix},
    hash::{asymmetric::AsymmetricHashAdjacencyMatrix, symmetric::SymmetricHashAdjacencyMatrix},
};
pub use directedness::{Directed, Directedness, Undirected};
pub use graph::{Graph, GraphMut, GraphDirected, GraphUndirected};
