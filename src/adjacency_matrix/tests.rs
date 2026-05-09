#![cfg(test)]
#![doc(hidden)]

use super::super::*;
use crate::adjacency_matrix::bitvec::matrix::BitvecAdjacencyMatrix;
use crate::adjacency_matrix::hash::HashAdjacencyMatrix;
use crate::test_util::{DropCounter, DroppableValue};
use generate_test_macro::generate_test_macro;
use quickcheck::{Arbitrary, TestResult};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::adjacency_matrix::AdjacencyMatrix;

#[derive(Clone, Debug)]
struct ArbMatrix<M: AdjacencyMatrix> {
    matrix: M,
    insertions: Vec<(usize, usize, M::Value)>,
}

impl<M> ArbMatrix<M>
where
    M: AdjacencyMatrix + Clone + 'static,
    M::Value: Clone + Arbitrary,
{
    fn new(insertions: Vec<(usize, usize, M::Value)>) -> Self {
        let mut matrix = M::with_size(0);
        for (row, col, data) in &insertions {
            matrix.insert(*row, *col, data.clone());
        }
        ArbMatrix { matrix, insertions }
    }
}

impl<M> Arbitrary for ArbMatrix<M>
where
    M: AdjacencyMatrix + Clone + 'static,
    M::Value: Clone + Arbitrary,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let mut insertions = Vec::with_capacity(g.size());
        for _ in 0..g.size() {
            let row = usize::arbitrary(g) % g.size();
            let col = usize::arbitrary(g) % g.size();
            let data = M::Value::arbitrary(g);
            insertions.push((row, col, data));
        }
        ArbMatrix::new(insertions)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let insertions = self.insertions.clone();
        Box::new(insertions.shrink().map(ArbMatrix::new))
    }
}

fn compute_size<M: AdjacencyMatrix>(matrix: &M) -> usize {
    matrix
        .iter()
        .fold(0, |size, (row, col, _)| size.max(row.max(col) + 1))
}

struct AdjacencyMatrixTests<M>(M);

#[generate_test_macro(adjacency_matrix_tests)]
impl<M> AdjacencyMatrixTests<M>
where
    M: AdjacencyMatrix<Value = &'static str> + Default,
{
    #[test]
    fn test_matrix_insert_and_get() {
        let mut matrix = M::default();
        matrix.insert(0, 0, "a");
        matrix.insert(1, 0, "b");
        matrix.insert(2, 0, "c");
        matrix.insert(6, 7, "d");
        assert_eq!(matrix.get(0, 0), Some(&"a"));
        assert_eq!(matrix.get(1, 0), Some(&"b"));
        assert_eq!(matrix.get(2, 0), Some(&"c"));
        assert_eq!(matrix.get(6, 7), Some(&"d"));
        if M::Directedness::IS_DIRECTED {
            assert_eq!(matrix.get(0, 1), None);
            assert_eq!(matrix.get(0, 2), None);
            assert_eq!(matrix.get(7, 6), None);
        } else {
            assert_eq!(matrix.get(0, 1), Some(&"b"));
            assert_eq!(matrix.get(0, 2), Some(&"c"));
            assert_eq!(matrix.get(7, 6), Some(&"d"));
        }
    }

    #[test]
    fn test_insert_overwrites() {
        let mut matrix = M::default();
        assert_eq!(matrix.insert(0, 1, "first"), None);
        assert_eq!(matrix.insert(0, 1, "second"), Some("first"));
        assert_eq!(matrix.get(0, 1), Some(&"second"));
    }

    #[test]
    fn test_remove() {
        let mut matrix = M::default();
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.remove(0, 1), Some("edge"));
        assert_eq!(matrix.get(0, 1), None);
    }

    #[test]
    fn test_remove_both_directions() {
        let mut matrix = M::default();
        matrix.insert(0, 1, "");
        let removed = matrix.remove(1, 0);
        if M::Directedness::IS_DIRECTED {
            assert_eq!(removed, None);
            assert_eq!(matrix.get(0, 1), Some(&""));
        } else {
            assert_eq!(removed, Some(""));
            assert_eq!(matrix.get(0, 1), None);
        }
        assert_eq!(matrix.get(1, 0), None);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut matrix = M::default();
        assert_eq!(matrix.remove(0, 1), None);
    }

    #[test]
    fn test_entries() {
        let mut matrix = M::default();
        matrix.insert(0, 1, "a");
        matrix.insert(1, 0, "b");
        matrix.insert(2, 3, "b");
        let entries: Vec<_> = matrix.iter().collect();
        if M::Directedness::IS_DIRECTED {
            assert_eq!(entries.len(), 3);
        } else {
            assert_eq!(entries.len(), 2);
        }
    }

    #[test]
    fn test_entries_in_row() {
        let mut matrix = M::default();
        matrix.insert(0, 1, "a");
        matrix.insert(0, 2, "b");
        matrix.insert(1, 2, "c");
        let entries: Vec<_> = matrix.entries_in_row(0).collect();
        assert_eq!(entries.len(), 2);
        let entries0: Vec<_> = matrix.entries_in_row(0).collect();
        let entries1: Vec<_> = matrix.entries_in_row(1).collect();
        let entries2: Vec<_> = matrix.entries_in_row(2).collect();
        if M::Directedness::IS_DIRECTED {
            assert_eq!(entries0.len(), 2);
            assert_eq!(entries1.len(), 1);
            assert_eq!(entries2.len(), 0);
        } else {
            assert_eq!(entries0.len(), 2);
            assert_eq!(entries1.len(), 2);
            assert_eq!(entries2.len(), 2);
            assert!(entries1.iter().any(|(to, _)| *to == 0));
            assert!(entries2.iter().any(|(to, _)| *to == 0));
            assert!(entries2.iter().any(|(to, _)| *to == 1));
        }
        assert!(entries0.iter().any(|(to, _)| *to == 1));
        assert!(entries0.iter().any(|(to, _)| *to == 2));
        assert!(entries1.iter().any(|(to, _)| *to == 2));
    }

    #[test]
    fn test_entries_in_col() {
        let mut matrix = M::default();
        matrix.insert(0, 1, "a");
        matrix.insert(0, 2, "b");
        matrix.insert(1, 2, "c");
        let entries0: Vec<_> = matrix.entries_in_col(0).collect();
        let entries1: Vec<_> = matrix.entries_in_col(1).collect();
        let entries2: Vec<_> = matrix.entries_in_col(2).collect();
        if M::Directedness::IS_DIRECTED {
            assert_eq!(entries0.len(), 0);
            assert_eq!(entries1.len(), 1);
            assert_eq!(entries2.len(), 2);
        } else {
            assert_eq!(entries0.len(), 2);
            assert_eq!(entries1.len(), 2);
            assert_eq!(entries2.len(), 2);
            assert!(entries0.iter().any(|(from, _)| *from == 1));
            assert!(entries0.iter().any(|(from, _)| *from == 2));
            assert!(entries1.iter().any(|(from, _)| *from == 2));
        }
        assert!(entries1.iter().any(|(from, _)| *from == 0));
        assert!(entries2.iter().any(|(from, _)| *from == 0));
        assert!(entries2.iter().any(|(from, _)| *from == 1));
    }

    #[test]
    fn test_large_indices() {
        let mut matrix = M::default();
        matrix.insert(100, 200, "");
        assert_eq!(matrix.get(100, 200), Some(&""));
        if !M::Directedness::IS_DIRECTED {
            assert_eq!(matrix.get(200, 100), Some(&""));
        }
    }

    #[test]
    fn test_self_loop() {
        let mut matrix = M::default();
        matrix.insert(5, 5, "");
        assert_eq!(matrix.get(5, 5), Some(&""));
    }

    #[test]
    fn test_iter() {
        let mut matrix = M::default();
        matrix.insert(0, 1, "A");
        matrix.insert(2, 3, "B");
        matrix.insert(1, 0, "C");
        let entries: Vec<_> = matrix.iter().collect();
        if M::Directedness::IS_DIRECTED {
            assert_eq!(entries.len(), 3);
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 0 && col == 1 && val == &"A")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 2 && col == 3 && val == &"B")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 1 && col == 0 && val == &"C")
            );
        } else {
            assert_eq!(entries.len(), 2);
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 0 && col == 1 && val == &"C")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 2 && col == 3 && val == &"B")
            );
        }
    }

    #[test]
    fn test_into_iter() {
        let mut matrix = M::default();
        matrix.insert(0, 1, "A");
        matrix.insert(2, 3, "B");
        matrix.insert(1, 0, "C");
        let entries: Vec<_> = matrix.into_iter().collect();
        if M::Directedness::IS_DIRECTED {
            assert_eq!(entries.len(), 3);
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 0 && col == 1 && val == "A")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 2 && col == 3 && val == "B")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 1 && col == 0 && val == "C")
            );
        } else {
            assert_eq!(entries.len(), 2);
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 0 && col == 1 && val == "C")
            );
            assert!(
                entries
                    .iter()
                    .any(|&(row, col, val)| row == 2 && col == 3 && val == "B")
            );
        }
    }

    #[test]
    fn test_len() {
        let mut matrix = M::default();
        assert_eq!(matrix.len(), 0);
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.len(), 1);
        matrix.insert(0, 1, "edge");
        assert_eq!(matrix.len(), 1);
        matrix.insert(1, 0, "edge");
        assert_eq!(
            matrix.len(),
            if M::Directedness::IS_DIRECTED { 2 } else { 1 }
        );
        matrix.remove(0, 1);
        assert_eq!(
            matrix.len(),
            if M::Directedness::IS_DIRECTED { 1 } else { 0 }
        );
        matrix.clear();
        assert_eq!(matrix.len(), 0);
    }

    #[test]
    fn test_clear_row_and_column() {
        let mut matrix = M::default();

        matrix.insert(0, 2, "edge_0_2");
        matrix.insert(1, 2, "edge_1_2");
        matrix.insert(2, 3, "edge_2_3");
        matrix.insert(2, 4, "edge_2_4");
        matrix.insert(0, 1, "edge_0_1");
        matrix.insert(3, 4, "edge_3_4");

        assert_eq!(matrix.len(), 6);

        matrix.clear_row_and_column(2, 2);

        assert_eq!(matrix.len(), 2);

        // Verify edges involving node 2 are gone
        assert_eq!(matrix.get(0, 2), None);
        assert_eq!(matrix.get(2, 0), None);
        assert_eq!(matrix.get(1, 2), None);
        assert_eq!(matrix.get(2, 1), None);
        assert_eq!(matrix.get(2, 3), None);
        assert_eq!(matrix.get(3, 2), None);
        assert_eq!(matrix.get(2, 4), None);
        assert_eq!(matrix.get(4, 2), None);

        // Verify other edges remain
        assert_eq!(matrix.get(0, 1), Some(&"edge_0_1"));
        assert_eq!(matrix.get(3, 4), Some(&"edge_3_4"));
        if !M::Directedness::IS_DIRECTED {
            assert_eq!(matrix.get(1, 0), Some(&"edge_0_1"));
            assert_eq!(matrix.get(4, 3), Some(&"edge_3_4"));
        }
    }

    #[test]
    fn test_clear_row_and_column_with_different_indices() {
        let mut matrix = M::default();

        matrix.insert(0, 1, "a");
        matrix.insert(0, 2, "b");
        matrix.insert(1, 2, "c");
        matrix.insert(1, 3, "d");
        matrix.insert(2, 3, "e");
        matrix.insert(3, 4, "f");

        assert_eq!(matrix.len(), 6);

        matrix.clear_row_and_column(1, 2);

        if M::Directedness::IS_DIRECTED {
            assert_eq!(matrix.len(), 3);
            assert_eq!(matrix.get(0, 1), Some(&"a"));
            assert_eq!(matrix.get(2, 3), Some(&"e"));
        } else {
            assert_eq!(matrix.len(), 1);
            assert_eq!(matrix.get(0, 1), None);
            assert_eq!(matrix.get(2, 3), None);
        }
        assert_eq!(matrix.get(0, 2), None);
        assert_eq!(matrix.get(1, 2), None);
        assert_eq!(matrix.get(1, 3), None);
        assert_eq!(matrix.get(1, 0), None);
        assert_eq!(matrix.get(2, 0), None);
        assert_eq!(matrix.get(2, 1), None);
        assert_eq!(matrix.get(3, 1), None);
        assert_eq!(matrix.get(3, 2), None);
        assert_eq!(matrix.get(3, 4), Some(&"f"));
    }

    #[test]
    fn test_clear_row_and_column_out_of_bounds() {
        let mut matrix = M::default();

        matrix.insert(0, 0, "test");
        matrix.insert(1, 1, "test2");

        // Should not panic or affect existing entries
        matrix.clear_row_and_column(100, 200);

        assert_eq!(matrix.len(), 2);
        assert_eq!(matrix.get(0, 0), Some(&"test"));
        assert_eq!(matrix.get(1, 1), Some(&"test2"));
    }
}

struct AdjacencyMatrixDropTests<M>(M);

#[generate_test_macro(adjacency_matrix_drop_tests)]
impl<M> AdjacencyMatrixDropTests<M>
where
    M: AdjacencyMatrix<Value = DroppableValue> + Default,
{
    #[test]
    fn test_clear_row_and_column_drops_values() {
        let mut matrix: M = M::default();
        let counter = DropCounter::new();

        matrix.insert(1, 0, counter.new_value());
        matrix.insert(1, 2, counter.new_value());
        matrix.insert(0, 1, counter.new_value());
        matrix.insert(2, 1, counter.new_value());
        matrix.insert(1, 1, counter.new_value());
        matrix.insert(0, 0, counter.new_value());
        matrix.insert(2, 2, counter.new_value());

        if M::Directedness::IS_DIRECTED {
            assert_eq!(counter.drop_count(), 0);
            assert_eq!(matrix.len(), 7);
        } else {
            assert_eq!(counter.drop_count(), 2);
            assert_eq!(matrix.len(), 5);
        }

        matrix.clear_row_and_column(1, 1);
        assert_eq!(counter.drop_count(), 5);

        drop(matrix);
        assert_eq!(counter.drop_count(), 7);
    }

    #[test]
    fn test_drop_initialized_values() {
        let mut matrix = M::default();
        let counter = DropCounter::new();

        // Insert some values
        matrix.insert(0, 1, counter.new_value());
        matrix.insert(2, 3, counter.new_value());
        matrix.insert(5, 7, counter.new_value());

        // Replace one value (should drop the old one)
        matrix.insert(0, 1, counter.new_value());
        assert_eq!(counter.drop_count(), 1);

        // Remove one value (should drop it)
        matrix.remove(2, 3);
        assert_eq!(counter.drop_count(), 2);

        // Total drops: 2 (from operations) + 2 (from matrix drop) = 4
        drop(matrix);
        assert_eq!(counter.drop_count(), 4);
    }

    #[test]
    fn test_no_double_drop_after_into_iter() {
        let mut matrix = M::default();
        let counter = DropCounter::new();

        // Insert some values
        matrix.insert(0, 1, counter.new_value());
        matrix.insert(2, 3, counter.new_value());
        matrix.insert(5, 7, counter.new_value());

        assert_eq!(counter.drop_count(), 0);
        // Consume matrix with into_iter
        let collected: Vec<_> = matrix.into_iter().collect();
        assert_eq!(collected.len(), 3);

        // Values should still be alive in collected
        assert_eq!(counter.drop_count(), 0);

        // Drop the collected values
        drop(collected);

        // Now all 3 values should be dropped exactly once
        assert_eq!(counter.drop_count(), 3);

        // Matrix was consumed by into_iter, so no additional drops
        assert_eq!(counter.drop_count(), 3);
    }

    #[test]
    fn test_no_double_drop_after_clear() {
        let mut matrix = M::default();
        let counter = DropCounter::new();

        // Insert some values
        matrix.insert(0, 1, counter.new_value());
        matrix.insert(2, 3, counter.new_value());
        matrix.insert(5, 7, counter.new_value());
        assert_eq!(counter.drop_count(), 0);

        // Clear should drop all values
        matrix.clear();
        assert_eq!(counter.drop_count(), 3);

        // Add new values after clear
        matrix.insert(1, 2, counter.new_value());
        matrix.insert(3, 4, counter.new_value());

        // Still 3 drops (new values not dropped yet)
        assert_eq!(counter.drop_count(), 3);

        drop(matrix);

        // Total: 3 (from clear) + 2 (from matrix drop) = 5
        assert_eq!(counter.drop_count(), 5);
    }
}

struct AdjacencyMatrixQuickCheckTests<M>(PhantomData<M>);

#[generate_test_macro(adjacency_matrix_quickcheck_tests)]
impl<M> AdjacencyMatrixQuickCheckTests<M>
where
    M: AdjacencyMatrix<Value = u8> + Clone + 'static,
{
    #[quickcheck]
    fn prop_size_consistent(ArbMatrix { matrix, .. }: ArbMatrix<M>) -> TestResult {
        let computed_size = compute_size(&matrix);
        if matrix.size_bound() >= computed_size {
            TestResult::passed()
        } else {
            TestResult::error(format!(
                "Size bound inconsistent: expected {}, got {}",
                computed_size,
                matrix.size_bound()
            ))
        }
    }

    #[quickcheck]
    fn prop_size_consistent_after_shrink(ArbMatrix { mut matrix, .. }: ArbMatrix<M>) -> TestResult {
        let computed_size = compute_size(&matrix);
        matrix.shrink_to_fit();
        if matrix.size_bound() == computed_size {
            TestResult::passed()
        } else {
            TestResult::error(format!(
                "Size bound inconsistent: expected {}, got {}",
                computed_size,
                matrix.size_bound()
            ))
        }
    }

    #[quickcheck]
    fn prop_size_consistent_after_removal(ArbMatrix { mut matrix, .. }: ArbMatrix<M>) -> bool {
        let computed_size = compute_size(&matrix);
        for (row, col) in matrix
            .iter()
            .map(|(row, col, _)| (row, col))
            .collect::<Vec<_>>()
        {
            matrix.remove(row, col);
            if matrix.size_bound() < computed_size {
                return false;
            }
        }
        true
    }

    #[quickcheck]
    fn prop_size_consistent_after_removal_and_shrink(
        ArbMatrix {
            mut matrix,
            insertions,
        }: ArbMatrix<M>,
    ) -> TestResult {
        for (row, col, _) in insertions {
            matrix.remove(row, col);
            matrix.shrink_to_fit();
            let expected_size = compute_size(&matrix);
            if matrix.size_bound() != expected_size {
                return TestResult::error(format!(
                    "Size bound inconsistent: expected {}, got {}",
                    expected_size,
                    matrix.size_bound()
                ));
            }
        }
        TestResult::passed()
    }

    #[quickcheck]
    fn prop_get_consistent(ArbMatrix { matrix, insertions }: ArbMatrix<M>) -> bool {
        let expected_values = insertions
            .into_iter()
            .map(|(row, col, data)| (M::Directedness::sort_pair((row, col)), data))
            .collect::<HashMap<_, _>>();
        for i in 0..matrix.size_bound() {
            for j in 0..matrix.size_bound() {
                let expected = expected_values.get(&M::Directedness::sort_pair((i, j)));
                if matrix.get(i, j) != expected {
                    return false;
                }
            }
        }
        true
    }

    #[quickcheck]
    fn prop_insert_and_get_consistent(ArbMatrix { mut matrix, .. }: ArbMatrix<M>) -> bool {
        let entries: Vec<_> = matrix
            .iter()
            .map(|(row, col, data)| (row, col, *data))
            .collect();

        for (row, col, data) in entries {
            if matrix.insert(row, col, data).is_none() {
                return false;
            }
            if let Some(val) = matrix.get(row, col) {
                if *val != data {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    #[quickcheck]
    fn prop_entries_in_row_consistent(ArbMatrix { matrix, .. }: ArbMatrix<M>) -> TestResult {
        for row in 0..matrix.size_bound() {
            let expected: Vec<_> = matrix
                .iter()
                .filter_map(|(i, j, _)| (i == row).then_some(j))
                .collect();
            let entries_in_row: Vec<_> = matrix
                .entries_in_row(row)
                .flat_map(|(j, _)| {
                    if M::Directedness::IS_DIRECTED {
                        Some(j)
                    } else {
                        (j >= row).then_some(j)
                    }
                })
                .collect();

            if expected.len() != entries_in_row.len()
                || !expected.iter().all(|col| entries_in_row.contains(col))
            {
                return TestResult::error(format!(
                    "Entries in row {} inconsistent: expected {:?}, got {:?}",
                    row, expected, entries_in_row
                ));
            }
        }
        TestResult::passed()
    }

    #[quickcheck]
    fn prop_entries_in_col_consistent(ArbMatrix { matrix, .. }: ArbMatrix<M>) -> TestResult {
        for col in 0..matrix.size_bound() {
            let expected: Vec<_> = matrix
                .iter()
                .filter_map(|(i, j, _)| (j == col).then_some(i))
                .collect();
            let entries_in_col: Vec<_> = matrix
                .entries_in_col(col)
                .flat_map(|(i, _)| {
                    if M::Directedness::IS_DIRECTED {
                        Some(i)
                    } else {
                        (i <= col).then_some(i)
                    }
                })
                .collect();

            if expected.len() != entries_in_col.len()
                || !expected.iter().all(|row| entries_in_col.contains(row))
            {
                return TestResult::error(format!(
                    "Entries in column {} inconsistent: expected {:?}, got {:?}",
                    col, expected, entries_in_col
                ));
            }
        }
        TestResult::passed()
    }

    #[quickcheck]
    fn prop_clear_and_len_consistent(ArbMatrix { mut matrix, .. }: ArbMatrix<M>) -> bool {
        matrix.clear();
        matrix.iter().next().is_none() && matrix.len() == 0
    }

    #[quickcheck]
    fn prop_clear_row_and_column_consistent(
        ArbMatrix { mut matrix, .. }: ArbMatrix<M>,
        row: usize,
        col: usize,
    ) -> TestResult {
        if matrix.is_empty() {
            return TestResult::discard();
        }

        let row = row % matrix.size_bound();
        let col = col % matrix.size_bound();
        matrix.clear_row_and_column(row, col);
        (matrix.entries_in_row(row).next().is_none() && matrix.entries_in_col(col).next().is_none())
            .into()
    }
}

adjacency_matrix_tests!(directed_hash: AdjacencyMatrixTests<HashAdjacencyMatrix<&'static str, Directed>>);
adjacency_matrix_drop_tests!(directed_hash_drop: AdjacencyMatrixDropTests<HashAdjacencyMatrix<DroppableValue, Directed>>);
adjacency_matrix_quickcheck_tests!(directed_hash_quickcheck: AdjacencyMatrixQuickCheckTests<HashAdjacencyMatrix<u8, Directed>>);

adjacency_matrix_tests!(
    undirected_hash: AdjacencyMatrixTests<HashAdjacencyMatrix<&'static str, Undirected>>
);
adjacency_matrix_drop_tests!(undirected_hash_drop: AdjacencyMatrixDropTests<HashAdjacencyMatrix<DroppableValue, Undirected>>);
adjacency_matrix_quickcheck_tests!(undirected_hash_quickcheck: AdjacencyMatrixQuickCheckTests<HashAdjacencyMatrix<u8, Undirected>>);

#[cfg(feature = "bitvec")]
adjacency_matrix_tests!(
    directed_bitvec: AdjacencyMatrixTests<BitvecAdjacencyMatrix<&'static str, Directed>>
);
#[cfg(feature = "bitvec")]
adjacency_matrix_drop_tests!(directed_bitvec_drop: AdjacencyMatrixDropTests<BitvecAdjacencyMatrix<DroppableValue, Directed>>);
#[cfg(feature = "bitvec")]
adjacency_matrix_quickcheck_tests!(directed_bitvec_quickcheck: AdjacencyMatrixQuickCheckTests<BitvecAdjacencyMatrix<u8, Directed>>);

#[cfg(feature = "bitvec")]
adjacency_matrix_tests!(
    undirected_bitvec: AdjacencyMatrixTests<BitvecAdjacencyMatrix<&'static str, Undirected>>
);
#[cfg(feature = "bitvec")]
adjacency_matrix_drop_tests!(undirected_bitvec_drop: AdjacencyMatrixDropTests<BitvecAdjacencyMatrix<DroppableValue, Undirected>>);
#[cfg(feature = "bitvec")]
adjacency_matrix_quickcheck_tests!(undirected_bitvec_quickcheck: AdjacencyMatrixQuickCheckTests<BitvecAdjacencyMatrix<u8, Undirected>>);
