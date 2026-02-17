use std::fmt::{Debug, Formatter};

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
pub fn other_value<T: Eq>((a, b): (T, T), target: &T) -> OtherValue<T> {
    if a == *target {
        if b == *target {
            OtherValue::Both(b)
        } else {
            OtherValue::Second(b)
        }
    } else if b == *target {
        OtherValue::First(a)
    } else {
        panic!("Neither value matches the target");
    }
}

#[macro_export]
macro_rules! static_dynamic_enum {
    ($vis:vis trait $static_trait:ident : $trait:ident { $($member:tt)* }; $vis2:vis enum $name:ident { $($value:ident),+ $(,)?}) => {
        $vis trait $static_trait: $trait { $($member)* }

        #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
        $vis enum $name {
            $($value),+
        }

        $(
            #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
            $vis struct $value;
        )+
    };
}
