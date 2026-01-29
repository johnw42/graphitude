/// Sorts a pair of values into nondescending order.
pub fn sort_pair<K: Ord>(a: K, b: K) -> (K, K) {
    if a <= b { (a, b) } else { (b, a) }
}

pub fn sort_pair_if<K: Ord>(a: K, b: K, should_sort: bool) -> (K, K) {
    if should_sort { sort_pair(a, b) } else { (a, b) }
}
