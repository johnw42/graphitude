use std::{fmt::Debug, hash::Hash};

use as_enum::AsEnum;
use quickcheck::Arbitrary;

use crate::{end_pair::EndPair, util::sort_pair_if};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, AsEnum)]
#[AsEnum(arbitrary)]
pub enum DynDirectedness {
    Directed,
    Undirected,
}

/// Trait defining the directedness behavior of graph edges.
///
/// This trait is implemented by [`Directed`] and [`Undirected`] marker types to
/// provide compile-time specialization of graph behavior, as well as by the
/// [`DynDirectedness`] enum for dynamic directedness.
pub trait Directedness: AsEnum<DynDirectedness> + Arbitrary {
    fn is_directed(&self) -> bool {
        matches!(self.as_enum(), DynDirectedness::Directed)
    }

    fn sort_pair<T: Ord>(&self, pair: (T, T)) -> (T, T) {
        sort_pair_if(!self.is_directed(), pair)
    }

    fn coordinate_pair<T: Ord>(&self, (first, second): (T, T)) -> EndPair<T, Self> {
        EndPair::new(first, second, *self)
    }
}

impl<T> Directedness for T where T: AsEnum<DynDirectedness> + Arbitrary {}
