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
