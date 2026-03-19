use std::{fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{
    DirectednessTrait, Undirected,
    util::{OtherValue, other_value, other_value_ref, sort_pair},
};

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
pub struct EndPair<T, D: DirectednessTrait> {
    data: (T, T),
    directedness: D,
}

impl<T, D> EndPair<T, D>
where
    D: DirectednessTrait,
{
    pub fn new(source: T, target: T, directedness: D) -> Self
    where
        T: Ord,
    {
        let data = if directedness.is_directed() {
            (source, target)
        } else {
            sort_pair((source, target))
        };
        Self { data, directedness }
    }

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

    pub fn directedness(&self) -> D {
        self.directedness
    }

    pub fn left(&self) -> &T {
        self.values().0
    }

    pub fn right(&self) -> &T {
        self.values().1
    }

    pub fn values(&self) -> (&T, &T) {
        (&self.data.0, &self.data.1)
    }

    pub fn into_left(self) -> T {
        self.into_values().0
    }

    pub fn into_right(self) -> T {
        self.into_values().1
    }

    pub fn into_values(self) -> (T, T) {
        self.data
    }

    pub fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq,
    {
        let (left, right) = self.values();
        other_value_ref(left, right, &value)
    }

    pub fn into_other_value(self, value: &T) -> OtherValue<T>
    where
        T: Eq,
    {
        let (left, right) = self.into_values();
        other_value(left, right, value)
    }

    pub fn into_sorted(self) -> EndPair<T, Undirected>
    where
        T: Ord + Clone,
        D: Default,
    {
        let (left, right) = self.into_values();
        EndPair::new(left, right, Undirected)
    }
}

impl<T: Ord, D: DirectednessTrait + Default> From<(T, T)> for EndPair<T, D> {
    fn from((a, b): (T, T)) -> Self {
        Self::new(a, b, D::default())
    }
}
