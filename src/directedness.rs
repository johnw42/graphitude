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

    type EdgesInContainer<T>: EdgesInContainer<T>;

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
    type EdgesInContainer<T> = Vec<T>;

    type EndPair<T>
        = (T, T)
    where
        T: Clone + Eq + Hash + Ord + Send + Sync;

    const IS_DIRECTED: bool = true;
}

#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Undirected;

impl Directedness for Undirected {
    type EdgesInContainer<T> = ();

    type EndPair<T>
        = SortedPair<T>
    where
        T: Clone + Eq + Hash + Ord + Send + Sync;

    const IS_DIRECTED: bool = false;
}

/// Trait defining the container type for incoming edges in a graph, which
/// varies based on the graph's directedness.  This is a `Vec` for directed
/// graphs and a unit type for undirected graphs, since undirected graphs do not
/// maintain separate incoming edge lists.
///
/// All methods have the same semantics as the corresponding `Vec` methods.
pub trait EdgesInContainer<T>: Default {
    fn as_slice(&self) -> &[T];
    fn len(&self) -> usize;
    fn push(&mut self, item: T);
    fn retain(&mut self, f: impl FnMut(&T) -> bool);
}

impl<T> EdgesInContainer<T> for Vec<T> {
    fn as_slice(&self) -> &[T] {
        self.as_slice()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn push(&mut self, item: T) {
        self.push(item);
    }

    fn retain(&mut self, f: impl FnMut(&T) -> bool) {
        self.retain(f);
    }
}

impl<T> EdgesInContainer<T> for () {
    fn as_slice(&self) -> &[T] {
        &[]
    }

    fn len(&self) -> usize {
        0
    }

    fn push(&mut self, _item: T) {
        unreachable!("Cannot push to EdgesInContainer for undirected graphs");
    }

    fn retain(&mut self, _f: impl FnMut(&T) -> bool) {}
}
