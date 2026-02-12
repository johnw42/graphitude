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
pub struct EdgeEnds<T, D: DirectednessTrait> {
    data: (T, T),
    directedness: D,
}

impl<T, D> EdgeEnds<T, D>
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
            sort_pair(source, target)
        };
        Self { data, directedness }
    }

    pub fn directedness(&self) -> D {
        self.directedness
    }

    pub fn source(&self) -> &T {
        self.values().0
    }

    pub fn target(&self) -> &T {
        self.values().1
    }

    pub fn values(&self) -> (&T, &T) {
        (&self.data.0, &self.data.1)
    }

    pub fn into_source(self) -> T {
        self.into_values().0
    }

    pub fn into_target(self) -> T {
        self.into_values().1
    }

    pub fn into_values(self) -> (T, T) {
        self.data
    }

    pub fn has_both(&self, a: &T, b: &T) -> bool
    where
        T: Eq,
    {
        (self.source() == a && self.target() == b)
            || (!self.directedness().is_directed() && self.source() == b && self.target() == a)
    }

    pub fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq,
    {
        other_value(self.values(), &value)
    }

    pub fn into_other_value(self, value: T) -> OtherValue<T>
    where
        T: Eq,
    {
        other_value(self.into_values(), &value)
    }
}
