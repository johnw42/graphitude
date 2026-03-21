use std::{
    fmt::{Debug, Formatter},
    panic::UnwindSafe,
};

use derivative::Derivative;

/// Sorts a pair of values into nondescending order.
pub fn sort_pair<K: Ord>((a, b): (K, K)) -> (K, K) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Sorts a pair of values into nondescending order if `should_sort` is true, otherwise returns them in the original order.
pub fn sort_pair_if<K: Ord>(should_sort: bool, pair: (K, K)) -> (K, K) {
    if should_sort { sort_pair(pair) } else { pair }
}

/// A wrapper type that implements `Debug` by delegating to a closure.
///
/// This allows creating ad-hoc `Debug` implementations without defining new types.
pub struct FormatDebugWith<F>(pub F);

impl<F> Debug for FormatDebugWith<F>
where
    F: Fn(&mut Formatter<'_>) -> std::fmt::Result,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        (self.0)(f)
    }
}

/// A wrapper type that implements `Debug` by writing a pre-formatted string.
///
/// This is useful when you have a string representation ready and want to present it as `Debug` output.
pub struct FormatDebugAs(pub String);

impl Debug for FormatDebugAs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Result type for [`other_value`] indicating which of two values is "other" than a target value.
pub enum OtherValue<T> {
    /// The first value is the "other" one (target matched the second value).
    First(T),
    /// The second value is the "other" one (target matched the first value).
    Second(T),
    /// Both values are the same as the target (all three are equal).
    Both(T),
}

impl<T> OtherValue<T> {
    /// Extracts the inner value regardless of which variant it is. This is
    /// useful when you just want the "other" value without caring about which
    /// one it was.
    pub fn into_inner(self) -> T {
        match self {
            OtherValue::First(value) | OtherValue::Second(value) | OtherValue::Both(value) => value,
        }
    }
}

/// Given two values and a target value, determines which of the two values is
/// the "other" one (i.e., not equal to the target).
///
/// # Panics
///
/// Panics if the target doesn't match either of the two values.
pub fn other_value<T: Eq>(first: T, second: T, target: &T) -> OtherValue<T> {
    if first == *target {
        if second == *target {
            OtherValue::Both(second)
        } else {
            OtherValue::Second(second)
        }
    } else if second == *target {
        OtherValue::First(first)
    } else {
        panic!("Neither value matches the target");
    }
}

pub fn other_value_ref<'a, 'b, T: Eq>(
    first: &'a T,
    second: &'a T,
    target: &'b T,
) -> OtherValue<&'a T> {
    if first == target {
        if second == target {
            OtherValue::Both(second)
        } else {
            OtherValue::Second(second)
        }
    } else if second == target {
        OtherValue::First(first)
    } else {
        panic!("Neither value matches the target");
    }
}

/// A pointer that cannot be dereferenced, used for identity purposes without allowing access to the underlying value.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    Hash(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct NonDereferenceable<T: ?Sized>(*const T);

impl<T: ?Sized> UnwindSafe for NonDereferenceable<T> {}

impl<T: ?Sized> Debug for NonDereferenceable<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T: ?Sized> From<*const T> for NonDereferenceable<T> {
    fn from(ptr: *const T) -> Self {
        NonDereferenceable(ptr)
    }
}

impl<T: ?Sized> From<&T> for NonDereferenceable<T> {
    fn from(ptr: &T) -> Self {
        NonDereferenceable(ptr as *const T)
    }
}
