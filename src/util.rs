pub(crate) fn sort_pair<K: Ord>(a: K, b: K) -> (K, K) {
    if a <= b { (a, b) } else { (b, a) }
}


// Calculates the Euler sum of n, the sum of 1..n, also the number of edges in a
// complete graph of n vertices.
pub(crate) fn euler_sum(n: usize) -> usize {
    (n * (n + 1)) / 2
}

// Inverse of the Euler sum function, returning the largest n such that
// euler_sum(n) <= k.
pub(crate) fn euler_sum_inv_floor(k: usize) -> usize {
    // (n * (n + 1)) / 2 = k
    // n * (n + 1) = 2k
    // n^2 + n - 2k = 0
    // By the quadratic formula:
    (1 + 8*k).isqrt().wrapping_sub(1) / 2
}

#[cfg(test)]
#[test]
fn test_euler_sum() {
    // Test some hand-picked values.
    assert_eq!(euler_sum(0), 0);
    assert_eq!(euler_sum(1), 1);
    assert_eq!(euler_sum(2), 3);
    assert_eq!(euler_sum(3), 6);
    assert_eq!(euler_sum(4), 10);
    assert_eq!(euler_sum_inv_floor(10), 4);
    assert_eq!(euler_sum_inv_floor(9), 3);
    assert_eq!(euler_sum_inv_floor(8), 3);
    assert_eq!(euler_sum_inv_floor(7), 3);
    assert_eq!(euler_sum_inv_floor(6), 3);
    assert_eq!(euler_sum_inv_floor(5), 2);
    assert_eq!(euler_sum_inv_floor(4), 2);
    assert_eq!(euler_sum_inv_floor(3), 2);
    assert_eq!(euler_sum_inv_floor(2), 1);
    assert_eq!(euler_sum_inv_floor(1), 1);
    assert_eq!(euler_sum_inv_floor(0), 0);
    
    // Test that euler_sum_inv_floor is the inverse of euler_sum and rounds down.
    for n in 0..1000 {
        let k1 = euler_sum(n);
        let k2 = euler_sum(n+1);
        for k in k1..k2 {
            let n2 = euler_sum_inv_floor(k);
            assert_eq!(n, n2, "Failed at k={}", k);
        }
    }
}
