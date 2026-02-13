use std::{fmt::Debug, hash::Hash};

/// Marker type representing single edges (no multiple edges between same nodes).
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SingleEdge;

/// Marker type representing multiple edges (multiple edges allowed between same nodes).
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MultipleEdges;

/// Enum representing the edge multiplicity of a graph.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeMultiplicity {
    SingleEdge,
    MultipleEdges,
}

/// Trait defining the edge multiplicity behavior of graphs.
///
/// This trait is implemented by [`SingleEdge`] and [`MultipleEdges`] marker
/// types to provide compile-time specialization of graph behavior based on edge
/// multiplicity. It is also implemented by the [`EdgeMultiplicity`] enum for
/// runtime configuration.
pub trait EdgeMultiplicityTrait:
    Copy + Clone + Debug + PartialEq + Eq + Hash + PartialOrd + Ord
{
    fn allows_parallel_edges(&self) -> bool;
}

impl EdgeMultiplicityTrait for SingleEdge {
    fn allows_parallel_edges(&self) -> bool {
        false
    }
}

impl EdgeMultiplicityTrait for MultipleEdges {
    fn allows_parallel_edges(&self) -> bool {
        true
    }
}

impl EdgeMultiplicityTrait for EdgeMultiplicity {
    fn allows_parallel_edges(&self) -> bool {
        match self {
            EdgeMultiplicity::SingleEdge => false,
            EdgeMultiplicity::MultipleEdges => true,
        }
    }
}
