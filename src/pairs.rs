use std::fmt::Debug;

use crate::util::{OtherValue, other_value, sort_pair};

/// Trait for pair types that may be ordered or unordered.
pub trait Pair<T>
where
    Self: Eq + From<(T, T)> + Into<(T, T)>,
{
    fn first(&self) -> &T;
    fn second(&self) -> &T;
    fn into_first(self) -> T;
    fn into_second(self) -> T;
    fn has_both(&self, a: &T, b: &T) -> bool
    where
        T: Eq;
    fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq + Debug;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct OrderedPair<T>(T, T);

impl<T> Pair<T> for OrderedPair<T>
where
    T: Eq + Ord,
{
    fn first(&self) -> &T {
        &self.0
    }

    fn second(&self) -> &T {
        &self.1
    }

    fn into_first(self) -> T {
        self.0
    }

    fn into_second(self) -> T {
        self.1
    }

    fn has_both(&self, a: &T, b: &T) -> bool
    where
        T: Eq,
    {
        self.0 == *a && self.1 == *b
    }

    fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq + Debug,
    {
        other_value((&self.0, &self.1), value)
    }
}

impl<T> From<(T, T)> for OrderedPair<T>
where
    T: Ord,
{
    fn from(pair: (T, T)) -> Self {
        Self(pair.0, pair.1)
    }
}

impl<T> From<OrderedPair<T>> for (T, T) {
    fn from(pair: OrderedPair<T>) -> Self {
        (pair.0, pair.1)
    }
}

impl<'a, T> From<&'a OrderedPair<T>> for (&'a T, &'a T) {
    fn from(pair: &'a OrderedPair<T>) -> Self {
        (&pair.0, &pair.1)
    }
}

/// An unordered pair of values that compares and hashes equal regardless of element order.
///
/// This is useful for representing edges in undirected graphs, where (a, b) and (b, a)
/// should be considered identical.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct SortedPair<T>(T, T);

impl<T: Ord> SortedPair<T> {
    /// Create a new `SortedPair` from two pre-sorted values.
    pub fn from_sorted(a: T, b: T) -> Self {
        debug_assert!(a <= b, "Values are not in sorted order");
        Self(a, b)
    }
}

impl<T: Ord + Eq> Pair<T> for SortedPair<T> {
    fn first(&self) -> &T {
        &self.0
    }

    fn second(&self) -> &T {
        &self.1
    }

    fn into_first(self) -> T {
        self.0
    }

    fn into_second(self) -> T {
        self.1
    }

    fn has_both(&self, a: &T, b: &T) -> bool
    where
        T: Eq,
    {
        (self.0 == *a && self.1 == *b) || (self.0 == *b && self.1 == *a)
    }

    fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq + Debug,
    {
        other_value((&self.0, &self.1), value)
    }
}

impl<T> From<(T, T)> for SortedPair<T>
where
    T: Ord,
{
    fn from(pair: (T, T)) -> Self {
        let (first, second) = sort_pair(pair.0, pair.1);
        Self(first, second)
    }
}

impl<T> From<SortedPair<T>> for (T, T) {
    fn from(pair: SortedPair<T>) -> Self {
        (pair.0, pair.1)
    }
}

impl<'a, T> From<&'a SortedPair<T>> for (&'a T, &'a T) {
    fn from(pair: &'a SortedPair<T>) -> Self {
        (&pair.0, &pair.1)
    }
}
