use std::{fmt::Debug, hash::Hash};

use crate::edge_ends::EdgeEnds;

/// Trait defining the directedness behavior of graph edges.
///
/// This trait is implemented by [`Directed`] and [`Undirected`] marker types to
/// provide compile-time specialization of graph behavior, as well as by the
/// [`Directedness`] enum for dynamic directedness.
pub trait DirectednessTrait:
    Copy + Clone + Debug + PartialEq + Eq + Hash + PartialOrd + Ord
{
    fn is_directed(&self) -> bool;

    fn default_is_directed() -> bool
    where
        Self: Default,
    {
        Self::default().is_directed()
    }

    fn make_pair<T: Clone + Eq + Ord + Debug + Hash>(&self, from: T, into: T) -> EdgeEnds<T, Self> {
        EdgeEnds::new(from, into, *self)
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Directed;

impl DirectednessTrait for Directed {
    fn is_directed(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Undirected;

impl DirectednessTrait for Undirected {
    fn is_directed(&self) -> bool {
        false
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
