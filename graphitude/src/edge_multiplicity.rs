use std::{fmt::Debug, hash::Hash};

use as_enum::AsEnum;
use quickcheck::Arbitrary;

/// Enum representing the edge multiplicity of a graph.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, AsEnum)]
#[AsEnum(arbitrary)]
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
pub trait EdgeMultiplicityTrait: AsEnum<EdgeMultiplicity> + Arbitrary {
    fn allows_parallel_edges(&self) -> bool {
        matches!(self.as_enum(), EdgeMultiplicity::MultipleEdges)
    }
}

impl<T> EdgeMultiplicityTrait for T where T: AsEnum<EdgeMultiplicity> + Arbitrary {}
