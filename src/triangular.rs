/// Calculates the nth triangular number.
///
/// The triangular number T(n) = 1 + 2 + ... + n = n(n+1)/2.
/// This is also equal to the number of edges in a complete undirected graph of n nodes.
pub fn triangular(n: usize) -> usize {
    (n * (n + 1)) / 2
}

/// Inverse of the triangular number function.
///
/// Returns the largest n such that `triangular(n) <= k`.
#[allow(dead_code)]
pub fn triangular_inv_floor(k: usize) -> usize {
    // n(n + 1) / 2 = k
    // n(n + 1) = 2k
    // n^2 + n - 2k = 0
    // By the quadratic formula:
    ((1 + 8 * k).isqrt() - 1) / 2
}

#[cfg(test)]
#[test]
fn test_triangular() {
    // Test some hand-picked values.
    assert_eq!(triangular(0), 0);
    assert_eq!(triangular(1), 1);
    assert_eq!(triangular(2), 3);
    assert_eq!(triangular(3), 6);
    assert_eq!(triangular(4), 10);
    assert_eq!(triangular_inv_floor(10), 4);
    assert_eq!(triangular_inv_floor(9), 3);
    assert_eq!(triangular_inv_floor(8), 3);
    assert_eq!(triangular_inv_floor(7), 3);
    assert_eq!(triangular_inv_floor(6), 3);
    assert_eq!(triangular_inv_floor(5), 2);
    assert_eq!(triangular_inv_floor(4), 2);
    assert_eq!(triangular_inv_floor(3), 2);
    assert_eq!(triangular_inv_floor(2), 1);
    assert_eq!(triangular_inv_floor(1), 1);
    assert_eq!(triangular_inv_floor(0), 0);

    // Test that triangular_inv_floor is the inverse of triangular and rounds down.
    for n in 0..1000 {
        let k1 = triangular(n);
        let k2 = triangular(n + 1);
        for k in k1..k2 {
            let n2 = triangular_inv_floor(k);
            assert_eq!(n, n2, "Failed at k={}", k);
        }
    }
}
