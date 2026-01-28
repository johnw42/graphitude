//! `Graph` and `GraphMut` are the core traits for working with graphs in this
//! library. `Graph` provides read-only access to the graph structure, while
//! `GraphMut` extends `Graph` with methods for modifying the graph.
//!
//! Mutating the data stored in nodes and edges is not provided directly
//! through these traits.  If you need to mutate the data, use interior
//! mutability (e.g., `RefCell`, `Cell`, `Mutex`, etc.) in your node and edge
//! data types.
//!
//! This module provides:
//!
//! - [`Graph`] trait: Core abstraction for graph data structures with support
//!   for nodes and edges
//! - [`GraphMut`] trait: Extension for mutable graph operations (add/remove
//!   nodes and edges)
//!
//! # Features
//!
//! - Flexible node and edge data storage through associated types
//! - Support for both directed and undirected graphs
//! - Graph traversal algorithms: DFS, BFS
//! - Path finding utilities with Dijkstra's algorithm (requires `pathfinding`
//!   feature)
//! - Queries for nodes, edges, predecessors, and successors

pub mod adjacency_graph;
pub mod adjacency_matrix;
pub mod debug;
pub mod directedness;
pub mod graph;
pub mod linked_graph;
pub mod object_graph;
pub mod path;
pub mod search;
pub mod tests;

mod graph_id;
mod id_vec;
mod symmetric_maxtrix_indexing;
mod util;

#[cfg(feature = "bitvec")]
mod euler_sum;

pub use directedness::{Directed, Directedness, Undirected};
pub use graph::{Graph, GraphDirected, GraphMut, GraphUndirected};
pub use linked_graph::LinkedGraph;
pub use tests::TestDataBuilder;

#[cfg(feature = "bitvec")]
pub use adjacency_matrix::{
    AdjacencyMatrix, Asymmetric, BitvecStorage, HashStorage, Storage, Symmetric, Symmetry,
    bitvec::{
        asymmetric::AsymmetricBitvecAdjacencyMatrix, symmetric::SymmetricBitvecAdjacencyMatrix,
    },
    hash::{asymmetric::AsymmetricHashAdjacencyMatrix, symmetric::SymmetricHashAdjacencyMatrix},
};
