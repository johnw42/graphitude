use std::fmt::{Debug, Formatter};

/// Sorts a pair of values into nondescending order.
pub fn sort_pair<K: Ord>(a: K, b: K) -> (K, K) {
    if a <= b { (a, b) } else { (b, a) }
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

/// Given two values and a target value, determines which of the two values is
/// the "other" one (i.e., not equal to the target).
///
/// # Panics
///
/// Panics if the target doesn't match either of the two values.
pub fn other_value<T: Eq + Debug>((a, b): (T, T), value: T) -> OtherValue<T> {
    if a == value {
        if b == value {
            OtherValue::Both(b)
        } else {
            OtherValue::Second(b)
        }
    } else if b == value {
        OtherValue::First(a)
    } else {
        panic!("Value {:?} doesn't match either {:?} or {:?}", value, a, b);
    }
}
