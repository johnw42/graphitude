use std::{fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{
    Directed, DirectednessTrait, Undirected,
    util::{OtherValue, other_value, sort_pair},
};

/// Trait for pair types that may be ordered or unordered.
pub trait EdgeEndsTrait<T, D>: Clone + Eq + Hash + Ord
where
    D: DirectednessTrait,
{
    fn new(source: T, target: T, directedness: D) -> Self;

    fn directedness(&self) -> D;

    fn source(&self) -> &T {
        self.values().0
    }

    fn target(&self) -> &T {
        self.values().1
    }

    fn values(&self) -> (&T, &T);

    fn into_source(self) -> T {
        self.into_values().0
    }

    fn into_target(self) -> T {
        self.into_values().1
    }

    fn into_values(self) -> (T, T);

    fn has_both(&self, a: &T, b: &T) -> bool
    where
        T: Eq,
    {
        (self.source() == a && self.target() == b)
            || (!self.directedness().is_directed() && self.source() == b && self.target() == a)
    }

    fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq,
    {
        other_value(self.values(), &value)
    }

    fn into_other_value(self, value: T) -> OtherValue<T>
    where
        T: Eq,
    {
        other_value(self.into_values(), &value)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct DirectedEnds<T>(T, T);

impl<T> EdgeEndsTrait<T, Directed> for DirectedEnds<T>
where
    T: Clone + Hash + Eq + Ord,
{
    fn new(source: T, target: T, _directedness: Directed) -> Self {
        Self(source, target)
    }

    fn directedness(&self) -> Directed {
        Directed
    }

    fn values(&self) -> (&T, &T) {
        (&self.0, &self.1)
    }

    fn into_values(self) -> (T, T) {
        (self.0, self.1)
    }
}

impl<T> From<(T, T)> for DirectedEnds<T>
where
    T: Ord,
{
    fn from(pair: (T, T)) -> Self {
        Self(pair.0, pair.1)
    }
}

/// An unordered pair of values that compares and hashes equal regardless of element order.
///
/// This is useful for representing edges in undirected graphs, where (a, b) and (b, a)
/// should be considered identical.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct UndirectedEnds<T>(T, T);

impl<T: Ord> UndirectedEnds<T> {
    /// Create a new `SortedPair` from two pre-sorted values.
    pub fn from_sorted(a: T, b: T) -> Self {
        debug_assert!(a <= b, "Values are not in sorted order");
        Self(a, b)
    }
}

impl<T> EdgeEndsTrait<T, Undirected> for UndirectedEnds<T>
where
    T: Clone + Hash + Ord + Eq,
{
    fn new(a: T, b: T, _directedness: Undirected) -> Self {
        let (first, second) = sort_pair(a, b);
        Self(first, second)
    }

    fn directedness(&self) -> Undirected {
        Undirected
    }

    fn values(&self) -> (&T, &T) {
        (&self.0, &self.1)
    }

    fn into_values(self) -> (T, T) {
        (self.0, self.1)
    }
}

impl<T> From<(T, T)> for UndirectedEnds<T>
where
    T: Ord,
{
    fn from(pair: (T, T)) -> Self {
        let (first, second) = sort_pair(pair.0, pair.1);
        Self(first, second)
    }
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = "T: Clone"),
    Debug(bound = "T: Debug"),
    Hash(bound = "T: Hash"),
    PartialEq(bound = "T: PartialEq"),
    Eq(bound = "T: Eq"),
    Ord(bound = "T: Ord"),
    PartialOrd(bound = "T: PartialOrd")
)]
pub struct EdgeEnds<T, D: DirectednessTrait> {
    data: (T, T),
    directedness: D,
}

impl<T, D> EdgeEndsTrait<T, D> for EdgeEnds<T, D>
where
    D: DirectednessTrait,
    T: Clone + Ord + Eq + Debug + Hash + PartialEq + PartialOrd,
{
    fn new(source: T, target: T, directedness: D) -> Self {
        let data = if directedness.is_directed() {
            (source, target)
        } else {
            sort_pair(source, target)
        };
        Self { data, directedness }
    }

    fn directedness(&self) -> D {
        self.directedness
    }

    fn values(&self) -> (&T, &T) {
        (&self.data.0, &self.data.1)
    }

    fn into_values(self) -> (T, T) {
        self.data
    }
}
