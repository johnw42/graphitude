use std::{fmt::Debug, hash::Hash};

/// Marker type representing single edges (no multiple edges between same nodes).
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SingleEdge;

/// Trait defining the edge multiplicity behavior of graphs.
///
/// This trait is implemented by [`SingleEdge`] and [`MultipleEdges`] marker
/// types to provide compile-time specialization of graph behavior based on edge
/// multiplicity. It is also implemented by the [`EdgeMultiplicity`] enum for
/// runtime configuration.
pub trait EdgeMultiplicity:
    Copy + Clone + Debug + Default + PartialEq + Eq + Hash + PartialOrd + Ord + Send + Sync
{
    const ALLOWS_PARALLEL_EDGES: bool;
}

impl EdgeMultiplicity for SingleEdge {
    const ALLOWS_PARALLEL_EDGES: bool = false;
}

/// Marker type representing multiple edges (multiple edges allowed between same nodes).
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MultipleEdges;

impl EdgeMultiplicity for MultipleEdges {
    const ALLOWS_PARALLEL_EDGES: bool = true;
}
