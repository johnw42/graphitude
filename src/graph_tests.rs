use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

use generate_test_macro::generate_test_macro;
use quickcheck::TestResult;

use crate::end_pair::EndPair as _;
use crate::generate_large_graph::generate_large_graph;
use crate::graph_test_support::{ArbGraph, check_graph_consistency, has_duplicates};
use crate::{GraphCopier, prelude::*};

#[doc(hidden)]
#[derive(Default)]
pub struct GraphTests<G> {
    next_node_index: usize,
    next_edge_index: usize,
    phantom: PhantomData<G>,
}

#[generate_test_macro(graph_test_suite)]
impl<G> GraphTests<G>
where
    G: GraphMut<NodeData = String, EdgeData = String> + Default + Clone,
{
    pub fn new() -> Self {
        Self {
            next_node_index: 0,
            next_edge_index: 0,
            phantom: PhantomData,
        }
    }

    fn new_graph(&self) -> G {
        G::default()
    }

    fn new_node_data(&mut self) -> String {
        let id = self.next_node_index;
        self.next_node_index += 1;
        format!("n{}", id)
    }

    fn new_edge_data(&mut self) -> String {
        let id = self.next_edge_index;
        self.next_edge_index += 1;
        format!("e{}", id)
    }

    /// Generates a large graph using the `TestDataBuilder` trait for data generation.
    ///
    /// This is a convenience wrapper around [`generate_large_graph_with`] that uses
    /// the TestDataBuilder trait to provide node and edge data.
    fn generate_large_graph(&self) -> G {
        let mut graph = self.new_graph();
        generate_large_graph(
            &mut graph,
            |i| format!("n{}", i),
            |i| format!("e{}", i),
            true,
        );
        graph
    }

    #[quickcheck]
    pub fn prop_node_ids_is_correct(
        ArbGraph {
            graph, node_ids, ..
        }: ArbGraph<G>,
    ) -> TestResult {
        let actual_node_ids = graph.node_ids().collect::<HashSet<_>>();
        let expected_node_ids = node_ids.into_iter().collect::<HashSet<_>>();
        if actual_node_ids == expected_node_ids {
            TestResult::passed()
        } else {
            TestResult::error(format!(
                "Node ID mismatch: expected {:?} but got {:?}",
                expected_node_ids, actual_node_ids
            ))
        }
    }

    #[quickcheck]
    pub fn prop_edge_ids_is_correct(
        ArbGraph {
            graph, edge_ids, ..
        }: ArbGraph<G>,
    ) -> TestResult {
        let actual_edge_ids = graph.edge_ids().collect::<HashSet<_>>();
        let expected_edge_ids = edge_ids.into_iter().collect::<HashSet<_>>();
        if actual_edge_ids == expected_edge_ids {
            TestResult::passed()
        } else {
            TestResult::error(format!(
                "Edge ID mismatch: expected {:?} but got {:?}",
                expected_edge_ids, actual_edge_ids
            ))
        }
    }

    #[quickcheck]
    pub fn prop_node_data_is_correct(
        ArbGraph {
            graph, node_data, ..
        }: ArbGraph<G>,
    ) -> TestResult {
        let actual_node_data = graph
            .node_ids()
            .map(|node_id| graph.node_data(&node_id).clone())
            .collect::<HashSet<_>>();
        let expected_node_data = node_data.into_iter().collect::<HashSet<_>>();
        if actual_node_data == expected_node_data {
            TestResult::passed()
        } else {
            TestResult::error(format!(
                "Node data mismatch: expected {:?} but got {:?}",
                expected_node_data, actual_node_data
            ))
        }
    }

    #[quickcheck]
    pub fn prop_edge_data_is_correct(
        ArbGraph {
            graph, edge_data, ..
        }: ArbGraph<G>,
    ) -> TestResult {
        let expected_edge_data = edge_data
            .into_iter()
            .map(|(_, data)| data)
            .collect::<HashSet<_>>();
        let actual_edge_data = graph
            .edge_ids()
            .map(|edge_id| graph.edge_data(&edge_id).clone())
            .collect::<HashSet<_>>();
        if actual_edge_data.is_subset(&expected_edge_data)
            && (!graph.allows_parallel_edges() || actual_edge_data == expected_edge_data)
        {
            TestResult::passed()
        } else {
            TestResult::error(format!(
                "Edge data mismatch: expected {:?} but got {:?}",
                expected_edge_data, actual_edge_data
            ))
        }
    }

    #[quickcheck]
    pub fn prop_num_nodes_is_correct(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        let actual_node_count = graph.node_ids().count();
        let expected_node_count = graph.num_nodes();
        actual_node_count == expected_node_count
    }

    #[quickcheck]
    pub fn prop_num_edges_is_correct(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        let actual_edge_count = graph.edge_ids().count();
        let expected_edge_count = graph.num_edges();
        actual_edge_count == expected_edge_count
    }

    #[quickcheck]
    pub fn prop_node_ids_are_unique(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        !has_duplicates(graph.node_ids())
    }

    #[quickcheck]
    pub fn prop_edge_ids_are_unique(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        !has_duplicates(graph.edge_ids())
    }

    #[quickcheck]
    pub fn prop_edges_from_returns_unique_values(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph
            .node_ids()
            .all(|node_id| !has_duplicates(graph.edges_from(&node_id)))
    }

    #[quickcheck]
    pub fn prop_edges_into_returns_unique_values(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph
            .node_ids()
            .all(|node_id| !has_duplicates(graph.edges_into(&node_id)))
    }

    #[quickcheck]
    pub fn prop_edges_from_into_returns_unique_values(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph.node_ids().all(|node_id| {
            graph.node_ids().all(|other_node_id| {
                !has_duplicates(graph.edges_from_into(&node_id, &other_node_id))
            })
        })
    }

    #[quickcheck]
    pub fn prop_edges_from_into_finds_all_edges(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph.edge_ids().all(|edge_id| {
            let (left, right) = graph.edge_ends(&edge_id).into_values();
            graph.edges_from_into(&left, &right).any(|e| e == edge_id)
        })
    }

    #[quickcheck]
    pub fn prop_num_edges_from_is_correct(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph.node_ids().all(|node_id| {
            let actual_count = graph.edges_from(&node_id).count();
            let expected_count = graph.num_edges_from(&node_id);
            actual_count == expected_count
        })
    }

    #[quickcheck]
    pub fn prop_num_edges_into_is_correct(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph.node_ids().all(|node_id| {
            let actual_count = graph.edges_into(&node_id).count();
            let expected_count = graph.num_edges_into(&node_id);
            actual_count == expected_count
        })
    }

    #[quickcheck]
    pub fn prop_num_edges_from_into_is_correct(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph.node_ids().all(|node_id| {
            graph.node_ids().all(|other_node_id| {
                let actual_count = graph.edges_from_into(&node_id, &other_node_id).count();
                let expected_count = graph.num_edges_from_into(&node_id, &other_node_id);
                actual_count == expected_count
            })
        })
    }

    #[quickcheck]
    pub fn prop_has_edge_from_is_correct(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph.node_ids().all(|node_id| {
            let has_edge = graph.has_edge_from(&node_id);
            let expected_has_edge = graph.edges_from(&node_id).next().is_some();
            has_edge == expected_has_edge
        })
    }

    #[quickcheck]
    pub fn prop_has_edge_into_is_correct(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph.node_ids().all(|node_id| {
            let has_edge = graph.has_edge_into(&node_id);
            let expected_has_edge = graph.edges_into(&node_id).next().is_some();
            has_edge == expected_has_edge
        })
    }

    #[quickcheck]
    pub fn prop_has_edge_from_into_is_correct(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        graph.node_ids().all(|node_id| {
            graph.node_ids().all(|other_node_id| {
                let has_edge = graph.has_edge_from_into(&node_id, &other_node_id);
                let expected_has_edge = graph
                    .edges_from_into(&node_id, &other_node_id)
                    .next()
                    .is_some();
                has_edge == expected_has_edge
            })
        })
    }

    #[quickcheck]
    pub fn prop_is_empty_is_correct(
        ArbGraph {
            graph,
            node_data,
            edge_data,
            ..
        }: ArbGraph<G>,
    ) {
        assert_eq!(
            graph.is_empty(),
            node_data.is_empty() && edge_data.is_empty()
        );
    }

    #[quickcheck]
    pub fn prop_clear_removes_all_nodes_and_edges(ArbGraph { mut graph, .. }: ArbGraph<G>) {
        graph.clear();
        assert!(graph.node_ids().next().is_none());
        assert!(graph.edge_ids().next().is_none());
    }

    #[quickcheck]
    pub fn prop_no_orphan_edges(ArbGraph { graph, .. }: ArbGraph<G>) {
        let all_edges = graph.edge_ids().collect::<HashSet<_>>();
        let from_edges = graph
            .node_ids()
            .flat_map(|node_id| graph.edges_from(&node_id).collect::<Vec<_>>())
            .collect::<HashSet<_>>();
        let into_edges = graph
            .node_ids()
            .flat_map(|node_id| graph.edges_into(&node_id).collect::<Vec<_>>())
            .collect::<HashSet<_>>();
        assert_eq!(all_edges, from_edges);
        assert_eq!(all_edges, into_edges);
    }

    #[quickcheck]
    pub fn prop_remove_node_removes_edges(ArbGraph { mut graph, .. }: ArbGraph<G>) {
        let node_id = graph.node_ids().next();
        if let Some(node_id) = node_id {
            let num_nodes = graph.num_nodes();
            let num_edges = graph.num_edges();
            let num_node_edges = graph
                .edges_from(&node_id)
                .chain(graph.edges_into(&node_id))
                .collect::<HashSet<_>>()
                .len();
            graph.remove_node(&node_id);
            assert_eq!(graph.num_nodes(), num_nodes - 1);
            assert!(graph.num_edges() <= num_edges - num_node_edges);
        }
    }

    #[quickcheck]
    pub fn prop_edges_in_and_out_are_consistent(ArbGraph { graph, .. }: ArbGraph<G>) -> bool {
        for node_id in graph.node_ids() {
            for edge_from in graph.edges_from(&node_id) {
                let other_node = graph
                    .edge_ends(&edge_from)
                    .into_other_value(&node_id)
                    .into_inner();
                if !graph.edges_into(&other_node).any(|e| e == edge_from) {
                    return false;
                }
            }
            for edge_into in graph.edges_into(&node_id) {
                let other_node = graph
                    .edge_ends(&edge_into)
                    .into_other_value(&node_id)
                    .into_inner();
                if !graph.edges_from(&other_node).any(|e| e == edge_into) {
                    return false;
                }
            }
        }
        true
    }

    #[quickcheck]
    pub fn prop_edges_from_into_is_consistent(ArbGraph { graph, .. }: ArbGraph<G>) {
        for node_id in graph.node_ids() {
            for other_node_id in graph.node_ids() {
                for edge_from_into in graph.edges_from_into(&node_id, &other_node_id) {
                    assert!(graph.edges_from(&node_id).any(|e| e == edge_from_into));
                    assert!(
                        graph
                            .edges_into(&other_node_id)
                            .any(|e| e == edge_from_into)
                    );
                }
            }
        }
    }

    #[test]
    pub fn test_large_graph_structure(&mut self) {
        let graph = self.generate_large_graph();
        check_graph_consistency(&graph);

        // Verify basic structure
        assert_eq!(
            graph.num_nodes(),
            410,
            "Expected 410 nodes: 50 + 80 + 150 + 20 + 100 + 10"
        );

        // The exact edge counts were verified via large_graph_to_dot + dot_summary.
        let num_edges = graph.num_edges();
        let expected_edges = match (graph.is_directed(), graph.allows_parallel_edges()) {
            (_, true) => 6454,
            (true, false) => 6439,
            (false, false) => 6383,
        };
        assert_eq!(
            num_edges, expected_edges,
            "Expected {} edges based on graph properties",
            expected_edges
        );

        // Count edges to verify consistency
        let edge_count_via_iteration = graph.edge_ids().count();
        assert_eq!(
            edge_count_via_iteration, num_edges,
            "Edge count mismatch: num_edges() returned {} but iteration counted {}",
            num_edges, edge_count_via_iteration
        );
    }

    #[cfg(feature = "slow_tests")]
    #[test]
    pub fn test_deconstruct_large_graph_by_nodes(&mut self) {
        use crate::tracing_support::{
            TimingScope, dump_method_timings, info_span, reset_method_timings, set_timing_scope,
        };

        reset_method_timings();
        let _scope = set_timing_scope(TimingScope::Test);
        let test_span = info_span!("test_deconstruct_large_graph_by_nodes");
        let _test_guard = test_span.entered();
        let mut graph = {
            let _span = info_span!("generate_large_graph").entered();
            self.generate_large_graph()
        };

        // We use a hash set instead of a vec so the nodes are removed in
        // random order.
        let mut node_ids = graph.node_ids().collect::<HashSet<_>>();

        // We deliberately fix the number of iterations because we know it
        // in advance; each iteraction removes one node.
        let _remove_loop_span = info_span!("remove_nodes_loop").entered();
        for i in 0..node_ids.len() {
            // Test removing a random node.
            assert!(!node_ids.is_empty());
            let num_nodes = node_ids.len();
            let num_edges = graph.num_edges();
            assert_eq!(num_nodes, node_ids.len());
            let node_id = node_ids.iter().next().cloned().unwrap();
            node_ids.remove(&node_id);
            {
                let _span = info_span!("remove_node").entered();
                graph.remove_node(&node_id);
            }
            assert_eq!(graph.num_nodes(), num_nodes - 1);
            assert!(graph.num_edges() <= num_edges);

            // Test compaction periodically
            if i % 50 == 0 {
                let num_nodes = node_ids.len();
                let num_edges = graph.num_edges();

                {
                    let _span = info_span!("compact").entered();
                    graph.compact(
                        Some(&mut |old_id, new_id| {
                            let removed = node_ids.remove(&old_id);
                            assert!(removed);
                            let inserted = node_ids.insert(new_id);
                            assert!(inserted);
                        }),
                        None,
                    );
                }
                assert_eq!(graph.num_nodes(), num_nodes);
                assert_eq!(graph.num_edges(), num_edges);
                {
                    let _span = info_span!("check_graph_consistency").entered();
                    check_graph_consistency(&graph);
                }
            }
        }
        drop(_remove_loop_span);

        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
        assert!(graph.is_empty());
        drop(_test_guard);
        dump_method_timings();
    }

    #[cfg(feature = "slow_tests")]
    #[test]
    pub fn test_deconstruct_large_graph_by_edges(&mut self) {
        use crate::tracing_support::{
            TimingScope, dump_method_timings, info_span, reset_method_timings, set_timing_scope,
        };

        reset_method_timings();
        let _scope = set_timing_scope(TimingScope::Test);
        let test_span = info_span!("test_deconstruct_large_graph_by_edges");
        let _test_guard = test_span.entered();
        let mut graph = {
            let _span = info_span!("generate_large_graph").entered();
            self.generate_large_graph()
        };

        // We use a hash set instead of a vec so the edges are removed in
        // random order.
        let mut edge_ids = graph.edge_ids().collect::<HashSet<_>>();

        let _remove_loop_span = info_span!("remove_edges_loop").entered();
        for i in 0..edge_ids.len() {
            // Test compaction periodically
            if i % 250 == 0 {
                let num_nodes = graph.num_nodes();
                let num_edges = edge_ids.len();

                {
                    let _span = info_span!("compact").entered();
                    graph.compact(
                        None,
                        Some(&mut |old_id, new_id| {
                            let removed = edge_ids.remove(&old_id);
                            assert!(removed);
                            let inserted = edge_ids.insert(new_id);
                            assert!(inserted);
                        }),
                    );
                }
                assert_eq!(graph.num_nodes(), num_nodes);
                assert_eq!(graph.num_edges(), num_edges);
                {
                    let _span = info_span!("check_graph_consistency").entered();
                    check_graph_consistency(&graph);
                }
            }

            // Test removing a random edge.
            assert!(!edge_ids.is_empty());
            let num_nodes = graph.num_nodes();
            let num_edges = edge_ids.len();
            assert_eq!(num_edges, edge_ids.len());
            let edge_id = edge_ids.iter().next().cloned().unwrap();
            edge_ids.remove(&edge_id);
            {
                let _span = info_span!("remove_edge").entered();
                graph.remove_edge(&edge_id);
            }
            assert_eq!(graph.num_nodes(), num_nodes);
            assert_eq!(graph.num_edges(), num_edges - 1);
        }
        drop(_remove_loop_span);

        assert_eq!(graph.num_edges(), 0);
        drop(_test_guard);
        dump_method_timings();
    }

    #[test]
    pub fn test_new_graph_is_empty(&mut self) {
        let graph = self.new_graph();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    pub fn test_node_data_retrieval(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let n1 = graph.add_node(nd1.clone());
        assert_eq!(*graph.node_data(&n1), nd1);
    }

    #[test]
    pub fn test_node_data_mutation(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let n1 = graph.add_node(nd1.clone());
        *graph.node_data_mut(&n1) = nd2.clone();
        assert_eq!(*graph.node_data(&n1), nd2);
    }

    #[test]
    pub fn test_edge_data_retrieval(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).edge_id();
        assert_eq!(*graph.edge_data(&e1), ed1);
    }

    #[test]
    pub fn test_edge_data_mutation(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();
        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).edge_id();
        *graph.edge_data_mut(&e1) = ed2.clone();
        assert_eq!(*graph.edge_data(&e1), ed2);
    }

    #[test]
    pub fn test_edge_creation(&mut self) {
        use std::collections::HashSet;

        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let nd3 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();
        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let n3 = graph.add_node(nd3);
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).edge_id();
        let e2 = graph.add_edge(&n2, &n3, ed2.clone()).edge_id();

        // Check edges_from and num_edges_from for each node
        if graph.is_directed() {
            assert_eq!(graph.num_edges_from(&n1), 1);
            assert_eq!(graph.num_edges_from(&n2), 1);
            assert_eq!(graph.num_edges_from(&n3), 0);
            assert_eq!(
                graph
                    .edges_from(&n1)
                    .map(|edge_id| graph.edge_ends(&edge_id).into_other_value(&n1).into_inner())
                    .collect::<Vec<_>>(),
                vec![n2.clone()]
            );
            assert_eq!(
                graph
                    .edges_from(&n2)
                    .map(|edge_id| graph.edge_ends(&edge_id).into_other_value(&n2).into_inner())
                    .collect::<Vec<_>>(),
                vec![n3.clone()]
            );
        } else {
            // For undirected: n1-n2, n2-n3
            assert_eq!(graph.num_edges_from(&n1), 1);
            assert_eq!(graph.num_edges_from(&n2), 2); // n2 connects to both n1 and n3
            assert_eq!(graph.num_edges_from(&n3), 1);
        }

        // Check edges_into and num_edges_into for each node
        if graph.is_directed() {
            assert_eq!(graph.num_edges_into(&n1), 0);
            assert_eq!(graph.num_edges_into(&n2), 1);
            assert_eq!(graph.num_edges_into(&n3), 1);
            assert_eq!(
                graph.edges_into(&n2).collect::<HashSet<_>>(),
                HashSet::from([e1.clone()])
            );
            assert_eq!(
                graph.edges_into(&n3).collect::<HashSet<_>>(),
                HashSet::from([e2.clone()])
            );
        } else {
            // For undirected graphs, edges_into should equal edges_from
            assert_eq!(graph.num_edges_into(&n1), 1);
            assert_eq!(graph.num_edges_into(&n2), 2);
            assert_eq!(graph.num_edges_into(&n3), 1);
            assert_eq!(
                graph.edges_into(&n1).collect::<HashSet<_>>(),
                HashSet::from([e1.clone()])
            );
            assert_eq!(
                graph.edges_into(&n2).collect::<HashSet<_>>(),
                HashSet::from([e1.clone(), e2.clone()])
            );
            assert_eq!(
                graph.edges_into(&n3).collect::<HashSet<_>>(),
                HashSet::from([e2.clone()])
            );
        }

        assert_eq!(graph.edge_data(&e1), (&ed1));
        assert_eq!(graph.edge_data(&e2), (&ed2));

        assert_eq!(graph.num_edges(), 2);
        assert_eq!(
            graph.edge_ids().collect::<HashSet<_>>(),
            HashSet::from([e1.clone(), e2.clone()])
        );
        assert_eq!(*graph.edge_data(&e1), ed1);
        assert_eq!(*graph.edge_data(&e2), ed2);
    }

    #[test]
    pub fn test_edge_ids(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let nd3 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();
        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let n3 = graph.add_node(nd3);
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).edge_id();
        let e2 = graph.add_edge(&n1, &n3, ed2.clone()).edge_id();

        let edge_ids: Vec<_> = graph.edge_ids().collect();
        assert_eq!(edge_ids.len(), 2);
        assert!(edge_ids.contains(&e1));
        assert!(edge_ids.contains(&e2));
    }

    #[test]
    pub fn test_edges_by_node(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();

        let n1 = graph.add_node(nd1.clone());
        let n2 = graph.add_node(nd2.clone());

        // Normal edge.
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).edge_id();
        assert_eq!(graph.num_edges(), 1);
        // Self edge.
        let e2 = graph.add_edge(&n1, &n1, ed2.clone()).edge_id();
        assert_eq!(graph.num_edges(), 2);
        // Duplicate edge.
        let add_3 = graph.add_edge(&n1, &n2, ed1.clone());
        if graph.allows_parallel_edges() {
            assert_eq!(graph.num_edges(), 3);
        } else {
            assert_eq!(graph.num_edges(), 2);
        }

        let edges_from_n1: Vec<_> = graph.edges_from(&n1).collect();
        assert!(edges_from_n1.contains(&e1));
        assert!(edges_from_n1.contains(&e2));
        if graph.allows_parallel_edges() {
            assert!(edges_from_n1.contains(&add_3.clone().edge_id()));
            assert_eq!(edges_from_n1.len(), 3);
        } else {
            assert!(matches!(&add_3, AddEdgeResult::Updated(_, data) if *data == ed1));
            dbg!(&edges_from_n1);
            assert_eq!(edges_from_n1.len(), 2);
        }

        let edges_into_n1: Vec<_> = graph.edges_into(&n1).collect();
        assert!(edges_into_n1.contains(&e2));
        if graph.is_directed() {
            assert_eq!(edges_into_n1.len(), 1);
        } else {
            assert!(edges_into_n1.contains(&e1));
            if graph.allows_parallel_edges() {
                assert!(edges_into_n1.contains(&add_3.edge_id()));
                assert_eq!(edges_into_n1.len(), 3);
            } else {
                assert!(matches!(&add_3, AddEdgeResult::Updated(_, data) if *data == ed1));
                assert_eq!(edges_into_n1.len(), 2);
            }
        }
    }

    #[test]
    pub fn test_node_removal(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();
        let ed3 = self.new_edge_data();

        let n1 = graph.add_node(nd1.clone());
        let n2 = graph.add_node(nd2.clone());

        // Normal edge.
        graph.add_edge(&n1, &n2, ed1.clone());
        assert_eq!(graph.num_edges(), 1);
        // Self edge.
        graph.add_edge(&n1, &n1, ed3.clone());
        assert_eq!(graph.num_edges(), 2);
        // Duplicate edge.
        graph.add_edge(&n1, &n2, ed2.clone());
        if graph.allows_parallel_edges() {
            assert_eq!(graph.num_edges(), 3);
        } else {
            assert_eq!(graph.num_edges(), 2);
        }

        let removed_data = graph.remove_node(&n1);
        assert_eq!(removed_data, nd1);
        assert_eq!(graph.num_nodes(), 1);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    pub fn test_remove_node_cleans_edges(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();
        let ed3 = self.new_edge_data();

        let n1 = graph.add_node(nd1.clone());
        let n2 = graph.add_node(nd2.clone());

        // Normal edge.
        graph.add_edge(&n1, &n2, ed1.clone());
        // Duplicate edge.
        graph.add_edge(&n1, &n2, ed2.clone());
        // Self edge.
        graph.add_edge(&n1, &n1, ed3.clone());

        graph.remove_node(&n1);
        assert_eq!(graph.num_nodes(), 1);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    pub fn test_edges_from(&mut self) {
        use std::collections::HashSet;

        let mut graph = self.new_graph();
        let n0 = graph.add_node(self.new_node_data());
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());
        let n3 = graph.add_node(self.new_node_data());

        let e0 = graph.add_edge(&n0, &n1, self.new_edge_data()).edge_id();
        let e1 = graph.add_edge(&n0, &n2, self.new_edge_data()).edge_id();
        let e2 = graph.add_edge(&n1, &n2, self.new_edge_data()).edge_id();
        let e3 = graph.add_edge(&n1, &n3, self.new_edge_data()).edge_id();
        let e4 = graph.add_edge(&n2, &n3, self.new_edge_data()).edge_id();
        // Check edges_from for all nodes
        assert_eq!(
            graph.edges_from(&n0).collect::<HashSet<_>>(),
            HashSet::from([e0.clone(), e1.clone()])
        );
        assert_eq!(graph.num_edges_from(&n0), 2);

        if graph.is_directed() {
            assert_eq!(
                graph.edges_from(&n1).collect::<HashSet<_>>(),
                HashSet::from([e2.clone(), e3.clone()])
            );
            assert_eq!(graph.num_edges_from(&n1), 2);
            assert_eq!(
                graph.edges_from(&n2).collect::<HashSet<_>>(),
                HashSet::from([e4.clone()])
            );
            assert_eq!(graph.num_edges_from(&n2), 1);
            assert_eq!(graph.edges_from(&n3).count(), 0);
            assert_eq!(graph.num_edges_from(&n3), 0);

            // Check edges_into for directed graphs
            assert_eq!(graph.num_edges_into(&n0), 0);
            assert_eq!(graph.num_edges_into(&n1), 1);
            assert_eq!(graph.num_edges_into(&n2), 2);
            assert_eq!(graph.num_edges_into(&n3), 2);
        } else {
            assert_eq!(
                graph.edges_from(&n1).collect::<HashSet<_>>(),
                HashSet::from([e0.clone(), e2.clone(), e3.clone()])
            );
            assert_eq!(graph.num_edges_from(&n1), 3);
            assert_eq!(
                graph.edges_from(&n2).collect::<HashSet<_>>(),
                HashSet::from([e1.clone(), e2.clone(), e4.clone()])
            );
            assert_eq!(graph.num_edges_from(&n2), 3);
            assert_eq!(
                graph.edges_from(&n3).collect::<HashSet<_>>(),
                HashSet::from([e3.clone(), e4.clone()])
            );
            assert_eq!(graph.num_edges_from(&n3), 2);

            // Check edges_into for undirected graphs (should match edges_from)
            assert_eq!(graph.num_edges_into(&n0), 2);
            assert_eq!(graph.num_edges_into(&n1), 3);
            assert_eq!(graph.num_edges_into(&n2), 3);
            assert_eq!(graph.num_edges_into(&n3), 2);
        }
    }

    #[test]
    pub fn test_edges_into(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();

        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).edge_id();

        // Check edges_into and num_edges_into
        assert_eq!(graph.edges_into(&n2).collect::<Vec<_>>(), vec![e1.clone()]);
        assert_eq!(graph.num_edges_into(&n2), 1);
        assert_eq!(
            graph.num_edges_into(&n1),
            if graph.is_directed() { 0 } else { 1 }
        );
        if !graph.is_directed() {
            assert_eq!(graph.edges_into(&n1).collect::<Vec<_>>(), vec![e1.clone()]);
        }

        // Check edges_from and num_edges_from
        assert_eq!(graph.num_edges_from(&n1), 1);
        assert_eq!(graph.edges_from(&n1).collect::<Vec<_>>(), vec![e1.clone()]);
        assert_eq!(
            graph.num_edges_from(&n2),
            if graph.is_directed() { 0 } else { 1 }
        );
    }

    #[test]
    pub fn test_edges_between(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();

        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let e1 = graph.add_edge(&n1, &n2, ed1).edge_id();

        assert_eq!(graph.num_edges_from_into(&n1, &n2), 1);
        assert_eq!(
            graph.edges_from_into(&n1, &n2).collect::<Vec<_>>(),
            vec![e1.clone()]
        );
        assert_eq!(
            graph.edges_from_into(&n2, &n1).collect::<Vec<_>>(),
            if graph.is_directed() {
                vec![]
            } else {
                vec![e1.clone()]
            }
        );
    }

    #[test]
    pub fn test_copy_from(&mut self) {
        let mut source = self.new_graph();
        let n1 = source.add_node(self.new_node_data());
        let n2 = source.add_node(self.new_node_data());
        let n3 = source.add_node(self.new_node_data());
        let e1 = source.add_edge(&n1, &n2, self.new_edge_data()).edge_id();
        let e2 = source.add_edge(&n2, &n3, self.new_edge_data()).edge_id();

        let mut node_map = HashMap::new();
        let mut edge_map = HashMap::new();
        let target = GraphCopier::new(&source)
            .clone_nodes()
            .clone_edges()
            .with_node_map(&mut node_map)
            .with_edge_map(&mut edge_map)
            .copy::<G>();

        assert_eq!(target.node_ids().count(), 3);
        assert_eq!(target.edge_ids().count(), 2);
        assert_eq!(source.node_data(&n1), target.node_data(&node_map[&n1]));
        assert_eq!(source.node_data(&n2), target.node_data(&node_map[&n2]));
        assert_eq!(source.node_data(&n3), target.node_data(&node_map[&n3]));
        assert_eq!(source.edge_data(&e1), target.edge_data(&edge_map[&e1]));
        assert_eq!(source.edge_data(&e2), target.edge_data(&edge_map[&e2]));
    }

    #[test]
    pub fn test_clear(&mut self) {
        let mut graph = self.new_graph();
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());
        graph.add_edge(&n1, &n2, self.new_edge_data());

        assert_eq!(graph.node_ids().count(), 2);
        assert_eq!(graph.edge_ids().count(), 1);

        graph.clear();

        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
        assert_eq!(graph.node_ids().count(), 0);
        assert_eq!(graph.edge_ids().count(), 0);
        assert!(graph.is_empty());
    }

    #[test]
    pub fn test_successors(&mut self) {
        let mut graph = self.new_graph();
        let n0 = graph.add_node(self.new_node_data());
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());
        let n3 = graph.add_node(self.new_node_data());

        let e0 = graph.add_edge(&n0, &n1, self.new_edge_data()).edge_id();
        let e1 = graph.add_edge(&n0, &n2, self.new_edge_data()).edge_id();
        let e2 = graph.add_edge(&n1, &n2, self.new_edge_data()).edge_id();
        let e3 = graph.add_edge(&n1, &n3, self.new_edge_data()).edge_id();
        let e4 = graph.add_edge(&n2, &n3, self.new_edge_data()).edge_id();
        if graph.is_directed() {
            assert_eq!(graph.edge_ends(&e0).into_values(), (n0.clone(), n1.clone()));
            assert_eq!(graph.edge_ends(&e1).into_values(), (n0.clone(), n2.clone()));
            assert_eq!(graph.edge_ends(&e2).into_values(), (n1.clone(), n2.clone()));
            assert_eq!(graph.edge_ends(&e3).into_values(), (n1.clone(), n3.clone()));
            assert_eq!(graph.edge_ends(&e4).into_values(), (n2.clone(), n3.clone()));
        } else {
            // For undirected graphs, edges can be in either direction
            let edge_pairs = vec![
                (e0.clone(), (n0.clone(), n1.clone())),
                (e1.clone(), (n0.clone(), n2.clone())),
                (e2.clone(), (n1.clone(), n2.clone())),
                (e3.clone(), (n1.clone(), n3.clone())),
                (e4.clone(), (n2.clone(), n3.clone())),
            ];
            for (edge, (a, b)) in edge_pairs {
                assert!(
                    graph.edge_ends(&edge).has_both(&a, &b),
                    "Edge {:?} does not connect nodes {:?} and {:?}",
                    edge,
                    a,
                    b
                );
            }
        }
        assert_eq!(
            graph.successors(&n0).collect::<HashSet<_>>(),
            HashSet::from([n1.clone(), n2.clone()])
        );

        // Check edge counts for all nodes
        if graph.is_directed() {
            assert_eq!(
                graph.successors(&n1).collect::<HashSet<_>>(),
                HashSet::from([n2.clone(), n3.clone()])
            );
            assert_eq!(
                graph.successors(&n2).collect::<HashSet<_>>(),
                HashSet::from([n3.clone()])
            );
            assert_eq!(graph.successors(&n3).count(), 0);

            // Check num_edges_from for directed graphs
            assert_eq!(graph.num_edges_from(&n0), 2);
            assert_eq!(graph.num_edges_from(&n1), 2);
            assert_eq!(graph.num_edges_from(&n2), 1);
            assert_eq!(graph.num_edges_from(&n3), 0);

            // Check num_edges_into for directed graphs
            assert_eq!(graph.num_edges_into(&n0), 0);
            assert_eq!(graph.num_edges_into(&n1), 1);
            assert_eq!(graph.num_edges_into(&n2), 2);
            assert_eq!(graph.num_edges_into(&n3), 2);
        } else {
            assert_eq!(
                graph.successors(&n1).collect::<HashSet<_>>(),
                HashSet::from([n0.clone(), n2.clone(), n3.clone()])
            );
            assert_eq!(
                graph.successors(&n2).collect::<HashSet<_>>(),
                HashSet::from([n0.clone(), n1.clone(), n3.clone()])
            );
            assert_eq!(
                graph.successors(&n3).collect::<HashSet<_>>(),
                HashSet::from([n1.clone(), n2.clone()])
            );

            // Check num_edges_from for undirected graphs
            assert_eq!(graph.num_edges_from(&n0), 2);
            assert_eq!(graph.num_edges_from(&n1), 3);
            assert_eq!(graph.num_edges_from(&n2), 3);
            assert_eq!(graph.num_edges_from(&n3), 2);

            // Check num_edges_into for undirected graphs (should equal num_edges_from)
            assert_eq!(graph.num_edges_into(&n0), 2);
            assert_eq!(graph.num_edges_into(&n1), 3);
            assert_eq!(graph.num_edges_into(&n2), 3);
            assert_eq!(graph.num_edges_into(&n3), 2);
        }
        assert_eq!(graph.num_edges(), 5);
        assert_eq!(graph.num_edges_from_into(&n0, &n1), 1);
        assert_eq!(graph.num_edges_from_into(&n0, &n2), 1);
        assert_eq!(graph.num_edges_from_into(&n1, &n2), 1);
        assert_eq!(graph.num_edges_from_into(&n1, &n3), 1);
        assert_eq!(graph.num_edges_from_into(&n2, &n3), 1);
        assert_eq!(graph.num_edges_from_into(&n0, &n3), 0);
    }

    #[test]
    pub fn test_predecessors(&mut self) {
        let mut graph = self.new_graph();
        let n0 = graph.add_node(self.new_node_data());
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());
        let n3 = graph.add_node(self.new_node_data());

        graph.add_edge(&n0, &n1, self.new_edge_data());
        graph.add_edge(&n0, &n2, self.new_edge_data());
        graph.add_edge(&n1, &n2, self.new_edge_data());
        graph.add_edge(&n1, &n3, self.new_edge_data());
        graph.add_edge(&n2, &n3, self.new_edge_data());

        if graph.is_directed() {
            assert_eq!(
                graph.predecessors(&n0).collect::<HashSet<_>>(),
                HashSet::new()
            );
            assert_eq!(
                graph.predecessors(&n1).collect::<HashSet<_>>(),
                HashSet::from([n0.clone()])
            );
            assert_eq!(
                graph.predecessors(&n2).collect::<HashSet<_>>(),
                HashSet::from([n0.clone(), n1.clone()])
            );
            assert_eq!(
                graph.predecessors(&n3).collect::<HashSet<_>>(),
                HashSet::from([n1.clone(), n2.clone()])
            );
        } else {
            // For undirected graphs, predecessors should equal successors
            assert_eq!(
                graph.predecessors(&n0).collect::<HashSet<_>>(),
                HashSet::from([n1.clone(), n2.clone()])
            );
            assert_eq!(
                graph.predecessors(&n1).collect::<HashSet<_>>(),
                HashSet::from([n0.clone(), n2.clone(), n3.clone()])
            );
            assert_eq!(
                graph.predecessors(&n2).collect::<HashSet<_>>(),
                HashSet::from([n0.clone(), n1.clone(), n3.clone()])
            );
            assert_eq!(
                graph.predecessors(&n3).collect::<HashSet<_>>(),
                HashSet::from([n1.clone(), n2.clone()])
            );
        }
    }

    #[cfg(feature = "pathfinding")]
    #[test]
    pub fn test_shortest_paths(&mut self) {
        let mut graph = self.new_graph();
        let n0 = graph.add_node(self.new_node_data());
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());
        let n3 = graph.add_node(self.new_node_data());

        graph.add_edge(&n0, &n1, self.new_edge_data());
        graph.add_edge(&n0, &n2, self.new_edge_data());
        graph.add_edge(&n1, &n2, self.new_edge_data());
        graph.add_edge(&n1, &n3, self.new_edge_data());
        graph.add_edge(&n2, &n3, self.new_edge_data());

        let paths = graph.shortest_paths(&n0, |_| 1);
        assert_eq!(paths[&n0].0.nodes().collect::<Vec<_>>(), vec![n0.clone()]);
        assert_eq!(
            paths[&n1].0.nodes().collect::<Vec<_>>(),
            vec![n0.clone(), n1.clone()]
        );
        assert_eq!(
            paths[&n2].0.nodes().collect::<Vec<_>>(),
            vec![n0.clone(), n2.clone()]
        );
        assert!(
            paths[&n3].0.nodes().collect::<Vec<_>>() == vec![n0.clone(), n1.clone(), n3.clone()]
                || paths[&n3].0.nodes().collect::<Vec<_>>()
                    == vec![n0.clone(), n2.clone(), n3.clone()]
        );
        assert_eq!(paths[&n3].1, 2);
    }

    #[cfg(feature = "pathfinding")]
    #[test]
    pub fn test_shortest_paths_disconnected(&mut self) {
        let mut graph = self.new_graph();
        let n0 = graph.add_node(self.new_node_data());
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());

        graph.add_edge(&n0, &n1, self.new_edge_data());

        let paths = graph.shortest_paths(&n0, |_| 1);
        assert_eq!(paths.get(&n0).map(|(_, dist)| *dist), Some(0));
        assert_eq!(paths.get(&n1).map(|(_, dist)| *dist), Some(1));
        assert_eq!(paths.get(&n2).map(|(_, dist)| *dist), None);
    }

    #[test]
    pub fn test_compaction(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();
        let ed3 = self.new_edge_data();

        let n1 = graph.add_node(nd1.clone());
        let n2 = graph.add_node(nd2.clone());
        let n3 = graph.add_node(self.new_node_data());
        let e1 = graph.add_edge(&n1, &n1, ed1.clone()).edge_id();
        let e2 = graph.add_edge(&n1, &n2, ed2.clone()).edge_id();
        let _e3 = graph.add_edge(&n2, &n3, ed3.clone()).edge_id();
        graph.remove_node(&n3);
        assert_eq!(graph.node_ids().count(), 2);
        assert_eq!(graph.edge_ids().count(), 2);

        let mut nid_map = HashMap::new();
        let mut eid_map = HashMap::new();
        graph.compact(Some(&mut nid_map), Some(&mut eid_map));
        assert_eq!(graph.node_ids().count(), 2);
        assert_eq!(graph.edge_ids().count(), 2);

        // After compaction, we need to use the new node ID.
        // Find the node with the same data
        assert_eq!(graph.node_data(nid_map.get(&n1).unwrap_or(&n1)), &nd1);
        assert_eq!(graph.node_data(nid_map.get(&n2).unwrap_or(&n2)), &nd2);
        assert_eq!(graph.edge_data(eid_map.get(&e1).unwrap_or(&e1)), &ed1);
        assert_eq!(graph.edge_data(eid_map.get(&e2).unwrap_or(&e2)), &ed2);
    }

    #[test]
    pub fn test_copy_from_with(&mut self) {
        let mut source = self.new_graph();
        let n1 = source.add_node(self.new_node_data());
        let n2 = source.add_node(self.new_node_data());
        let n3 = source.add_node(self.new_node_data());
        let e0 = source.add_edge(&n1, &n2, self.new_edge_data()).edge_id();
        let e1 = source.add_edge(&n2, &n3, self.new_edge_data()).edge_id();

        let mut transform_node = |x: &String| format!("ND[{x}]");
        let mut transform_edge = |x: &String| format!("ED[{x}]");

        let mut node_map = HashMap::new();
        let mut edge_map = HashMap::new();
        let target = GraphCopier::new(&source)
            .transform_nodes(&mut transform_node)
            .transform_edges(&mut transform_edge)
            .with_node_map(&mut node_map)
            .with_edge_map(&mut edge_map)
            .copy::<G>();

        assert_eq!(target.node_ids().count(), 3);
        assert_eq!(target.edge_ids().count(), 2);
        assert_eq!(
            transform_node(source.node_data(&n1)),
            *target.node_data(&node_map[&n1])
        );
        assert_eq!(
            transform_node(source.node_data(&n2)),
            *target.node_data(&node_map[&n2])
        );
        assert_eq!(
            transform_node(source.node_data(&n3)),
            *target.node_data(&node_map[&n3])
        );
        assert_eq!(
            transform_edge(source.edge_data(&e0)),
            *target.edge_data(&edge_map[&e0])
        );
        assert_eq!(
            transform_edge(source.edge_data(&e1)),
            *target.edge_data(&edge_map[&e1])
        );
    }

    #[test]
    pub fn test_edge_multiplicity(&mut self) {
        let mut graph = self.new_graph();
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());
        let result = graph.add_edge(&n1, &n2, self.new_edge_data());
        assert!(matches!(result, AddEdgeResult::Added(_)));
        let result = graph.add_edge(&n1, &n2, self.new_edge_data());
        assert_eq!(
            graph.allows_parallel_edges(),
            matches!(result, AddEdgeResult::Added(_))
        );

        if graph.allows_parallel_edges() {
            assert_eq!(graph.num_edges(), 2);
        } else {
            assert_eq!(graph.num_edges(), 1);
        }
    }
}
