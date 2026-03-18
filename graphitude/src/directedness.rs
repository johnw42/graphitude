use std::{fmt::Debug, hash::Hash};

use as_enum::AsEnum;
use quickcheck::Arbitrary;

use crate::{coordinate_pair::CoordinatePair, util::sort_pair_if};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, AsEnum)]
#[as_enum(arbitrary)]
pub enum Directedness {
    Directed,
    Undirected,
}

/// Trait defining the directedness behavior of graph edges.
///
/// This trait is implemented by [`Directed`] and [`Undirected`] marker types to
/// provide compile-time specialization of graph behavior, as well as by the
/// [`Directedness`] enum for dynamic directedness.
pub trait DirectednessTrait: AsEnum<Directedness> + Arbitrary {
    fn is_directed(&self) -> bool {
        matches!(self.as_enum(), Directedness::Directed)
    }

    fn sort_pair<T: Ord>(&self, pair: (T, T)) -> (T, T) {
        sort_pair_if(!self.is_directed(), pair)
    }

    fn coordinate_pair<T: Ord>(&self, (first, second): (T, T)) -> CoordinatePair<T, Self> {
        CoordinatePair::new(first, second, *self)
    }
}

impl<T> DirectednessTrait for T where T: AsEnum<Directedness> + Arbitrary {}
