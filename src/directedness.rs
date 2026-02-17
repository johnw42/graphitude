use std::{fmt::Debug, hash::Hash};

use quickcheck::Arbitrary;

use crate::coordinate_pair::CoordinatePair;

/// Trait defining the directedness behavior of graph edges.
///
/// This trait is implemented by [`Directed`] and [`Undirected`] marker types to
/// provide compile-time specialization of graph behavior, as well as by the
/// [`Directedness`] enum for dynamic directedness.
pub trait DirectednessTrait:
    Copy
    + Clone
    + Debug
    + PartialEq
    + Eq
    + Hash
    + PartialOrd
    + Ord
    + Send
    + Sync
    + TryFrom<Directed>
    + TryFrom<Undirected>
    + TryFrom<Directedness>
    + Arbitrary
{
    fn is_directed(&self) -> bool;

    fn default_is_directed() -> bool
    where
        Self: Default,
    {
        Self::default().is_directed()
    }

    fn coordinate_pair<T: Clone + Eq + Ord + Debug + Hash>(
        &self,
        (first, second): (T, T),
    ) -> CoordinatePair<T, Self> {
        CoordinatePair::new(first, second, *self)
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Directed;

impl DirectednessTrait for Directed {
    fn is_directed(&self) -> bool {
        true
    }
}

impl TryFrom<Directed> for Undirected {
    type Error = ();

    fn try_from(_: Directed) -> Result<Self, Self::Error> {
        Err(())
    }
}

impl TryFrom<Directedness> for Directed {
    type Error = ();

    fn try_from(value: Directedness) -> Result<Self, Self::Error> {
        match value {
            Directedness::Directed => Ok(Directed),
            Directedness::Undirected => Err(()),
        }
    }
}

impl Arbitrary for Directed {
    fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
        Directed
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Undirected;

impl DirectednessTrait for Undirected {
    fn is_directed(&self) -> bool {
        false
    }
}

impl TryFrom<Undirected> for Directed {
    type Error = ();

    fn try_from(_: Undirected) -> Result<Self, Self::Error> {
        Err(())
    }
}

impl TryFrom<Directedness> for Undirected {
    type Error = ();

    fn try_from(value: Directedness) -> Result<Self, Self::Error> {
        match value {
            Directedness::Directed => Err(()),
            Directedness::Undirected => Ok(Undirected),
        }
    }
}
impl Arbitrary for Undirected {
    fn arbitrary(_g: &mut quickcheck::Gen) -> Self {
        Undirected
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Directedness {
    Directed,
    Undirected,
}

impl DirectednessTrait for Directedness {
    fn is_directed(&self) -> bool {
        matches!(self, Directedness::Directed)
    }
}

impl From<Directed> for Directedness {
    fn from(_: Directed) -> Self {
        Directedness::Directed
    }
}

impl From<Undirected> for Directedness {
    fn from(_: Undirected) -> Self {
        Directedness::Undirected
    }
}

impl Arbitrary for Directedness {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        if bool::arbitrary(g) {
            Directedness::Directed
        } else {
            Directedness::Undirected
        }
    }
}
