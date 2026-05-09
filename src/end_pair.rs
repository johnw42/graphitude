use std::{fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::util::{OtherValue, other_value, sort_pair};

pub trait EndPair<T>:
    From<(T, T)> + Clone + Hash + PartialEq + Eq + PartialOrd + Ord + Send + Sync
{
    fn left(&self) -> &T;
    fn right(&self) -> &T;
    fn values(&self) -> (&T, &T);
    fn into_first(self) -> T;
    fn into_second(self) -> T;
    fn into_values(self) -> (T, T);

    fn has(&self, value: &T) -> bool
    where
        T: Eq,
    {
        self.left() == value || self.right() == value
    }

    fn has_both(&self, a: &T, b: &T) -> bool
    where
        T: Eq,
    {
        self.has(a) && self.has(b)
    }

    fn other_value<'a: 'b, 'b>(&'a self, value: &'b T) -> OtherValue<&'b T>
    where
        T: Eq,
    {
        other_value(self.values(), &value)
    }

    fn into_other_value(self, value: &T) -> OtherValue<T>
    where
        T: Eq,
    {
        other_value(self.into_values(), value)
    }

    fn map<U, F, EP>(self, mut f: F) -> EP
    where
        F: FnMut(&T) -> U,
        EP: EndPair<U>,
    {
        From::from((f(self.left()), f(self.right())))
    }
}

impl<T> EndPair<T> for (T, T)
where
    T: Clone + Eq + Hash + Ord + Send + Sync,
{
    fn left(&self) -> &T {
        &self.0
    }

    fn right(&self) -> &T {
        &self.1
    }

    fn values(&self) -> (&T, &T) {
        (&self.0, &self.1)
    }

    fn into_first(self) -> T {
        self.0
    }

    fn into_second(self) -> T {
        self.1
    }

    fn into_values(self) -> (T, T) {
        (self.0, self.1)
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
pub struct SortedPair<T>(T, T);

impl<T> EndPair<T> for SortedPair<T>
where
    T: Clone + Eq + Hash + Ord + Send + Sync,
{
    fn left(&self) -> &T {
        &self.0
    }

    fn right(&self) -> &T {
        &self.1
    }

    fn values(&self) -> (&T, &T) {
        (&self.0, &self.1)
    }

    fn into_first(self) -> T {
        self.0
    }

    fn into_second(self) -> T {
        self.1
    }

    fn into_values(self) -> (T, T) {
        (self.0, self.1)
    }
}

impl<T: Ord> From<(T, T)> for SortedPair<T> {
    fn from(pair: (T, T)) -> Self {
        let (left, right) = sort_pair(pair);
        Self(left, right)
    }
}
