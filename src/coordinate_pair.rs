use std::{fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{
    DirectednessTrait,
    util::{OtherValue, other_value, sort_pair},
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
pub struct CoordinatePair<T, D: DirectednessTrait> {
    data: (T, T),
    directedness: D,
}

impl<T, D> CoordinatePair<T, D>
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

    pub fn first(&self) -> &T {
        self.values().0
    }

    pub fn second(&self) -> &T {
        self.values().1
    }

    pub fn values(&self) -> (&T, &T) {
        (&self.data.0, &self.data.1)
    }

    pub fn into_first(self) -> T {
        self.into_values().0
    }

    pub fn into_second(self) -> T {
        self.into_values().1
    }

    pub fn into_values(self) -> (T, T) {
        self.data
    }

    pub fn has_both(&self, a: &T, b: &T) -> bool
    where
        T: Eq,
    {
        (self.first() == a && self.second() == b)
            || (!self.directedness().is_directed() && self.first() == b && self.second() == a)
    }

    pub fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq,
    {
        other_value(self.values(), &value)
    }

    pub fn into_other_value(self, value: &T) -> OtherValue<T>
    where
        T: Eq,
    {
        other_value(self.into_values(), value)
    }
}

impl<T: Ord, D: DirectednessTrait + Default> From<(T, T)> for CoordinatePair<T, D> {
    fn from((a, b): (T, T)) -> Self {
        Self::new(a, b, D::default())
    }
}
