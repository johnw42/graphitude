use std::{fmt::Debug, hash::Hash};

use as_enum::AsEnum;
use quickcheck::Arbitrary;

use crate::end_pair::EndPair;

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
    /// Returns `true` if the directedness is directed.
    fn is_directed(&self) -> bool {
        matches!(self.as_enum(), DynDirectedness::Directed)
    }

    /// Sorts a pair of values if the directedness is undirected, and returns
    /// them in the original order if directed.
    fn sort_pair<T: Ord>(&self, pair: (T, T)) -> (T, T) {
        self.end_pair(pair).into_values()
    }

    /// Creates an `EndPair` from a pair of values.
    fn end_pair<T: Ord>(&self, pair: (T, T)) -> EndPair<T, Self> {
        EndPair::new(pair, *self)
    }
}

impl<T> Directedness for T where T: AsEnum<DynDirectedness> + Arbitrary {}
