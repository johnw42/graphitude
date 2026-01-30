/// Sorts a pair of values into nondescending order.
pub fn sort_pair<K: Ord>(a: K, b: K) -> (K, K) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Conditionally sorts a pair of values.
///
/// If `should_sort` is true, returns the pair in nondescending order.
/// Otherwise, returns the pair unchanged as `(a, b)`.
pub fn sort_pair_if<K: Ord>(should_sort: bool, a: K, b: K) -> (K, K) {
    if should_sort { sort_pair(a, b) } else { (a, b) }
}
