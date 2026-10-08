#![cfg(test)]
#![doc(hidden)]

use super::super::*;
use crate::adjacency_matrix::bitvec::matrix::BitvecAdjacencyMatrix;
use crate::adjacency_matrix::hash::HashAdjacencyMatrix;
use crate::test_util::{DropCounter, DroppableValue};
use generate_test_macro::generate_test_macro;
use iterable_assertions::assert_eq_sets;
use proptest::collection::vec;
use proptest::prelude::*;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::adjacency_matrix::AdjacencyMatrix;

fn arb_matrix<M>() -> impl Strategy<Value = M>
where
    M: AdjacencyMatrix + Clone + Debug + 'static,
    M::Value: Clone + Arbitrary,
{
    (0..64_usize).prop_flat_map(|size| {
        vec((0..size, 0..size, any::<M::Value>()), size).prop_map(|insertions| {
            let mut matrix = M::with_size(0);
            for (row, col, data) in &insertions {
                matrix.insert(*row, *col, data.clone());
            }
            matrix
        })
    })
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
    M: AdjacencyMatrix<Value = u8> + Clone + Debug + 'static,
{
    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_size_consistent(matrix: M) {
        let computed_size = compute_size(&matrix);
        prop_assert!(
            matrix.size_bound() >= computed_size,
            "Size bound inconsistent: expected {}, got {}",
            computed_size,
            matrix.size_bound(),
        );
    }

    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_size_consistent_after_shrink(mut matrix: M) {
        let computed_size = compute_size(&matrix);
        matrix.shrink_to_fit();
        prop_assert_eq!(matrix.size_bound(), computed_size);
    }

    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_size_consistent_after_removal(mut matrix: M) {
        let computed_size = compute_size(&matrix);
        for (row, col) in matrix
            .iter()
            .map(|(row, col, _)| (row, col))
            .collect::<Vec<_>>()
        {
            matrix.remove(row, col);
            prop_assert!(
                matrix.size_bound() >= computed_size,
                "Size bound inconsistent after removal: expected at least {}, got {}",
                computed_size,
                matrix.size_bound(),
            );
        }
    }

    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_size_consistent_after_removal_and_shrink(mut matrix: M) {
        let items: Vec<_> = matrix.iter().map(|(row, col, _)| (row, col)).collect();
        for (row, col) in items {
            matrix.remove(row, col);
            matrix.shrink_to_fit();
            let expected_size = compute_size(&matrix);
            prop_assert_eq!(matrix.size_bound(), expected_size);
        }
    }

    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_insert_and_get_consistent(mut matrix: M) {
        let entries: Vec<_> = matrix
            .iter()
            .map(|(row, col, data)| (row, col, *data))
            .collect();

        for (row, col, data) in entries {
            prop_assert_eq!(matrix.insert(row, col, data ^ 1), Some(data));
            if let Some(val) = matrix.get(row, col) {
                prop_assert_eq!(*val, data ^ 1);
            } else {
                prop_assert!(false, "Value just inserted not found");
            }
        }
    }

    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_entries_in_row_consistent(matrix: M) {
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

            assert_eq_sets!(expected, entries_in_row);
        }
    }

    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_entries_in_col_consistent(matrix: M) {
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

            assert_eq_sets!(expected, entries_in_col);
        }
    }

    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_clear_and_len_consistent(mut matrix: M) {
        matrix.clear();
        prop_assert!(matrix.iter().next().is_none());
        prop_assert_eq!(matrix.len(), 0);
    }

    #[proptest(args in arb_matrix::<M>().prop_filter("Matrix is empty", |matrix| {
        !matrix.is_empty() && matrix.size_bound() > 0
    }).prop_flat_map(|matrix| {
        let size_bound = matrix.size_bound();
        (Just(matrix), 0..size_bound, 0..size_bound)
    }))]
    fn prop_clear_row_and_column_consistent(args: (M, usize, usize)) {
        let (mut matrix, row, col) = args;
        matrix.clear_row_and_column(row, col);
        prop_assert!(matrix.entries_in_row(row).next().is_none());
        prop_assert!(matrix.entries_in_col(col).next().is_none());
    }

    #[proptest(matrix in arb_matrix::<M>())]
    fn prop_compact_consistent(matrix: M) {
        let mut compacted = matrix.clone();
        let mut index_map = HashMap::new();
        compacted.compact(Some(&mut index_map));
        prop_assert!(compacted.size_bound() <= matrix.size_bound());
        prop_assert_eq!(compacted.len(), matrix.len());
        for (row, col, data) in matrix.iter() {
            let (c_row, c_col) = index_map.get(&(row, col)).cloned().unwrap_or((row, col));
            prop_assert_eq!(compacted.get(c_row, c_col), Some(data));
        }
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
