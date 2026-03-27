use std::{fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{
    Directedness, Undirected,
    util::{OtherValue, other_value, other_value_ref, sort_tuple},
};

/// A pair of values representing the two ends of an edge, with associated
/// directedness information.
///
/// The two ends are referred to as "left" and "right".  For directed edges, the
/// left end is the source and the right end is the target.  For undirected
/// edges, the left and right ends are stored in sorted order to ensure a
/// consistent representation regardless of the order they were provided in.
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
pub struct EndPair<T, D: Directedness> {
    data: (T, T),
    directedness: D,
}

impl<T, D> EndPair<T, D>
where
    D: Directedness,
{
    /// Creates a new EndPair with the given source and target values and directedness.
    /// For directed edges, the order of source and target is preserved.  For undirected
    /// edges, the values are sorted to ensure a consistent order.
    pub fn new((source, target): (T, T), directedness: D) -> Self
    where
        T: Ord,
    {
        let data = if directedness.is_directed() {
            (source, target)
        } else {
            sort_tuple((source, target))
        };
        Self { data, directedness }
    }

    /// Creates a new EndPair from the given values, assuming the values are already sorted if the edge is undirected.
    pub fn from_sorted(a: T, b: T) -> Self
    where
        D: Default,
        T: Ord,
    {
        debug_assert!(a <= b);
        Self {
            data: (a, b),
            directedness: D::default(),
        }
    }

    /// Gets the directedness of the edge.
    pub fn directedness(&self) -> D {
        self.directedness
    }

    /// Gets the left end of the edge.  For directed edges, this is the source node.
    pub fn left(&self) -> &T {
        self.values().0
    }

    /// Gets the right end of the edge.  For directed edges, this is the target node.
    pub fn right(&self) -> &T {
        self.values().1
    }

    /// Returns `(self.left(), self.right())`.
    pub fn values(&self) -> (&T, &T) {
        (&self.data.0, &self.data.1)
    }

    /// Gets the left end of the edge, consuming self.
    pub fn into_left(self) -> T {
        self.into_values().0
    }

    /// Gets the right end of the edge, consuming self.
    pub fn into_right(self) -> T {
        self.into_values().1
    }

    /// Returns `(self.into_left(), self.into_right())`, consuming self.
    pub fn into_values(self) -> (T, T) {
        self.data
    }

    /// Given one end of the edge, returns the other end.  For self loops, this returns the same node.
    /// Panics if the given node is not one of the ends of the edge.
    pub fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq,
    {
        other_value_ref(self.values(), &value)
    }

    /// Same as [`Self::other_value`] but consumes self and returns owned values instead of references.
    pub fn into_other_value(self, value: &T) -> OtherValue<T>
    where
        T: Eq,
    {
        other_value(self.into_values(), value)
    }

    /// Converts this `EndPair` into an undirected `EndPair` with the same values in sorted order.
    pub fn into_undirected(self) -> EndPair<T, Undirected>
    where
        T: Ord + Clone,
        D: Default,
    {
        EndPair::new(self.into_values(), Undirected)
    }
}

impl<T: Ord, D: Directedness + Default> From<(T, T)> for EndPair<T, D> {
    fn from(pair: (T, T)) -> Self {
        Self::new(pair, D::default())
    }
}
