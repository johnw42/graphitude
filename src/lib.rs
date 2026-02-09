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

#[cfg(feature = "bitvec")]
pub mod adjacency_graph;
#[cfg(feature = "bitvec")]
pub mod adjacency_matrix;
pub mod debug;
pub mod debug_graph_view;
pub mod directedness;
#[cfg(feature = "dot")]
pub mod dot;
pub mod edge_multiplicity;
pub mod graph;
pub mod linked_graph;
pub mod object_graph;
pub mod path;
pub mod prelude;
pub mod search;
pub mod tests;

mod graph_id;
mod id_vec;
mod pairs;
mod test_util;
mod util;

#[cfg(feature = "bitvec")]
mod triangular;

pub use directedness::{Directed, DirectednessTrait, Undirected};
pub use edge_multiplicity::{EdgeMultiplicityTrait, MultipleEdges, SingleEdge};
pub use graph::{
    AddEdgeResult, EdgeIdTrait, Graph, GraphDirected, GraphMut, GraphNew, GraphUndirected,
    NodeIdTrait,
};
pub use linked_graph::LinkedGraph;
pub use pairs::SortedPair;
pub use tests::TestDataBuilder;

#[cfg(feature = "dot")]
pub use dot::parser::{GraphBuilder, ParseError};

#[cfg(feature = "bitvec")]
pub use adjacency_matrix::{
    AdjacencyMatrix, Asymmetric, BitvecStorage, HashStorage, Storage, Symmetric, SymmetryTrait,
    bitvec::{
        asymmetric::AsymmetricBitvecAdjacencyMatrix, symmetric::SymmetricBitvecAdjacencyMatrix,
    },
    hash::{asymmetric::AsymmetricHashAdjacencyMatrix, symmetric::SymmetricHashAdjacencyMatrix},
};
