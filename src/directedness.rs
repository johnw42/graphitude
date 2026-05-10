use std::{fmt::Debug, hash::Hash};

use crate::{
    end_pair::{EndPair, SortedPair},
    util::sort_pair_if,
};

/// Trait defining the directedness behavior of graph edges.
///
/// This trait is implemented by [`Directed`] and [`Undirected`] marker types to
/// provide compile-time specialization of graph behavior, as well as by the
/// [`Directedness`] enum for dynamic directedness.
pub trait Directedness:
    Clone + Copy + Debug + Default + PartialEq + Eq + Hash + PartialOrd + Ord + Send + Sync
{
    type EndPair<T>: EndPair<T>
    where
        T: Clone + Eq + Hash + Ord + Send + Sync;

    const IS_DIRECTED: bool;

    fn sort_pair<T: Ord>(pair: (T, T)) -> (T, T) {
        sort_pair_if(!Self::IS_DIRECTED, pair)
    }

    fn make_pair<T>(left: T, right: T) -> Self::EndPair<T>
    where
        T: Clone + Eq + Hash + Ord + Send + Sync,
    {
        Self::EndPair::from((left, right))
    }
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Directed;

impl Directedness for Directed {
    type EndPair<T>
        = (T, T)
    where
        T: Clone + Eq + Hash + Ord + Send + Sync;

    const IS_DIRECTED: bool = true;
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Undirected;

impl Directedness for Undirected {
    type EndPair<T>
        = SortedPair<T>
    where
        T: Clone + Eq + Hash + Ord + Send + Sync;

    const IS_DIRECTED: bool = false;
}
