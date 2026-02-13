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
pub trait EdgeMultiplicityTrait:
    Copy
    + Clone
    + Debug
    + PartialEq
    + Eq
    + Hash
    + PartialOrd
    + Ord
    + TryFrom<SingleEdge>
    + TryFrom<MultipleEdges>
    + TryFrom<EdgeMultiplicity>
{
    fn allows_parallel_edges(&self) -> bool;
}

impl EdgeMultiplicityTrait for SingleEdge {
    fn allows_parallel_edges(&self) -> bool {
        false
    }
}

impl TryFrom<SingleEdge> for MultipleEdges {
    type Error = ();

    fn try_from(_: SingleEdge) -> Result<Self, Self::Error> {
        Err(())
    }
}

impl TryFrom<EdgeMultiplicity> for SingleEdge {
    type Error = ();

    fn try_from(value: EdgeMultiplicity) -> Result<Self, Self::Error> {
        match value {
            EdgeMultiplicity::SingleEdge => Ok(SingleEdge),
            EdgeMultiplicity::MultipleEdges => Err(()),
        }
    }
}

/// Marker type representing multiple edges (multiple edges allowed between same nodes).
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MultipleEdges;

impl EdgeMultiplicityTrait for MultipleEdges {
    fn allows_parallel_edges(&self) -> bool {
        true
    }
}

impl TryFrom<MultipleEdges> for SingleEdge {
    type Error = ();

    fn try_from(_: MultipleEdges) -> Result<Self, Self::Error> {
        Err(())
    }
}

impl TryFrom<EdgeMultiplicity> for MultipleEdges {
    type Error = ();

    fn try_from(value: EdgeMultiplicity) -> Result<Self, Self::Error> {
        match value {
            EdgeMultiplicity::SingleEdge => Err(()),
            EdgeMultiplicity::MultipleEdges => Ok(MultipleEdges),
        }
    }
}

/// Enum representing the edge multiplicity of a graph.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum EdgeMultiplicity {
    SingleEdge,
    MultipleEdges,
}

impl EdgeMultiplicityTrait for EdgeMultiplicity {
    fn allows_parallel_edges(&self) -> bool {
        match self {
            EdgeMultiplicity::SingleEdge => false,
            EdgeMultiplicity::MultipleEdges => true,
        }
    }
}

impl From<SingleEdge> for EdgeMultiplicity {
    fn from(_: SingleEdge) -> Self {
        EdgeMultiplicity::SingleEdge
    }
}

impl From<MultipleEdges> for EdgeMultiplicity {
    fn from(_: MultipleEdges) -> Self {
        EdgeMultiplicity::MultipleEdges
    }
}
