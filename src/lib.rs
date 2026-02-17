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

// Modules for using existing graph implementations and utilities.
pub mod adjacency_graph;
pub mod adjacency_matrix;
pub mod copier;
pub mod debug_graph_view;
pub mod directedness;
#[cfg(feature = "dot")]
pub mod dot;
pub mod edge_ends;
pub mod edge_multiplicity;
pub mod graph;
pub mod linked_graph;
pub mod object_graph;
pub mod path;
pub mod prelude;
pub mod search;

// Modules for creating new graph implementations.
pub mod format_debug;
pub mod graph_id;
pub mod tests;
#[doc(hidden)]
pub mod tracing_support;

mod automap;
mod test_util;
#[cfg(feature = "bitvec")]
mod triangular;
mod util;

pub use adjacency_graph::AdjacencyGraph;
#[cfg(feature = "bitvec")]
pub use adjacency_matrix::BitvecStorage;
pub use adjacency_matrix::{HashStorage, Storage};
pub use copier::GraphCopier;
pub use directedness::{Directed, Directedness, DirectednessTrait, Undirected};
pub use edge_multiplicity::{EdgeMultiplicity, EdgeMultiplicityTrait, MultipleEdges, SingleEdge};
pub use graph::{
    AddEdgeResult, EdgeIdTrait, Graph, GraphDirected, GraphMut, GraphUndirected, NodeIdTrait,
};
pub use linked_graph::LinkedGraph;
pub use tests::TestDataBuilder;
