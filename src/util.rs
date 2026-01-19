pub(crate) fn sort_pair<K: Ord>(a: K, b: K) -> (K, K) {
    if a <= b { (a, b) } else { (b, a) }
}
