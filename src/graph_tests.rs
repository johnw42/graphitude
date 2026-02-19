use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

use crate::generate_large_graph::generate_large_graph;
use crate::graph_test_support::{ArbGraph, check_graph_consistency, has_duplicates};
use crate::{GraphCopier, prelude::*};

/// Trait for building test data for graphs.  Graph implementations used in
/// tests should implement this trait to provide consistent node and edge data.
pub trait TestDataBuilder {
    type Graph: GraphMut;

    /// Creates a new graph instance for testing.
    fn new_graph(&self) -> Self::Graph;

    /// Creates new edge data for testing, given an index.  Tests will call this
    /// method with consecutive indices starting from zero.
    fn new_edge_data(&self, i: usize) -> <Self::Graph as Graph>::EdgeData;

    /// Creates new node data for testing, given an index.  Tests will call
    /// this method with consecutive indices starting from zero.
    fn new_node_data(&self, i: usize) -> <Self::Graph as Graph>::NodeData;
}

#[doc(hidden)]
#[allow(clippy::type_complexity)]
pub struct GraphTests<B>
where
    B: TestDataBuilder,
{
    pub builder: B,
    pub next_node_index: usize,
    pub next_edge_index: usize,
    pub transform_node: Box<dyn Fn(&TestNodeData<B>) -> TestNodeData<B>>,
    pub transform_edge: Box<dyn Fn(&TestEdgeData<B>) -> TestEdgeData<B>>,
}

type TestGraph<B> = <B as TestDataBuilder>::Graph;
type TestNodeData<B> = <TestGraph<B> as Graph>::NodeData;
type TestEdgeData<B> = <TestGraph<B> as Graph>::EdgeData;

impl<B> GraphTests<B>
where
    B: TestDataBuilder,
    TestNodeData<B>: Clone + Eq + Debug,
    TestEdgeData<B>: Clone + Eq + Debug,
{
    pub fn new_graph(&self) -> B::Graph {
        self.builder.new_graph()
    }

    pub fn new_node_data(&mut self) -> <B::Graph as Graph>::NodeData {
        let id = self.next_node_index;
        self.next_node_index += 1;
        self.builder.new_node_data(id)
    }

    pub fn new_edge_data(&mut self) -> <B::Graph as Graph>::EdgeData {
        let id = self.next_edge_index;
        self.next_edge_index += 1;
        self.builder.new_edge_data(id)
    }

    /// Generates a large graph using the `TestDataBuilder` trait for data generation.
    ///
    /// This is a convenience wrapper around [`generate_large_graph_with`] that uses
    /// the TestDataBuilder trait to provide node and edge data.
    fn generate_large_graph(&self) -> TestGraph<B>
    where
        B: TestDataBuilder,
        B::Graph: GraphMut,
    {
        let mut graph = self.new_graph();
        generate_large_graph(
            &mut graph,
            |i| self.builder.new_node_data(i),
            |i| self.builder.new_edge_data(i),
        );
        graph
    }

    pub fn prop_node_ids_are_valid(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        graph
            .node_ids()
            .all(|node_id| graph.check_valid_node_id(&node_id).is_ok())
    }

    pub fn prop_edge_ids_are_valid(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        graph
            .edge_ids()
            .all(|edge_id| graph.check_valid_edge_id(&edge_id).is_ok())
    }

    pub fn prop_num_nodes_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        let actual_node_count = graph.node_ids().count();
        let expected_node_count = graph.num_nodes();
        actual_node_count == expected_node_count
    }

    pub fn prop_num_edges_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        let actual_edge_count = graph.edge_ids().count();
        let expected_edge_count = graph.num_edges();
        actual_edge_count == expected_edge_count
    }

    pub fn prop_node_ids_are_unique(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        !has_duplicates(graph.node_ids())
    }

    pub fn prop_edge_ids_are_unique(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        !has_duplicates(graph.edge_ids())
    }

    pub fn prop_edges_from_returns_unique_values(
        ArbGraph { graph }: ArbGraph<TestGraph<B>>,
    ) -> bool {
        graph
            .node_ids()
            .all(|node_id| !has_duplicates(graph.edges_from(&node_id)))
    }

    pub fn prop_edges_into_returns_unique_values(
        ArbGraph { graph }: ArbGraph<TestGraph<B>>,
    ) -> bool {
        graph
            .node_ids()
            .all(|node_id| !has_duplicates(graph.edges_into(&node_id)))
    }

    pub fn prop_edges_from_into_returns_unique_values(
        ArbGraph { graph }: ArbGraph<TestGraph<B>>,
    ) -> bool {
        graph.node_ids().all(|node_id| {
            graph.node_ids().all(|other_node_id| {
                !has_duplicates(graph.edges_from_into(&node_id, &other_node_id))
            })
        })
    }

    pub fn prop_edges_from_into_finds_all_edges(
        ArbGraph { graph }: ArbGraph<TestGraph<B>>,
    ) -> bool {
        graph.edge_ids().all(|edge_id| {
            let (left, right) = edge_id.ends();
            graph.edges_from_into(&left, &right).any(|e| e == edge_id)
        })
    }

    pub fn prop_num_edges_from_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        graph.node_ids().all(|node_id| {
            let actual_count = graph.edges_from(&node_id).count();
            let expected_count = graph.num_edges_from(&node_id);
            actual_count == expected_count
        })
    }

    pub fn prop_num_edges_into_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        graph.node_ids().all(|node_id| {
            let actual_count = graph.edges_into(&node_id).count();
            let expected_count = graph.num_edges_into(&node_id);
            actual_count == expected_count
        })
    }

    pub fn prop_num_edges_from_into_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        graph.node_ids().all(|node_id| {
            graph.node_ids().all(|other_node_id| {
                let actual_count = graph.edges_from_into(&node_id, &other_node_id).count();
                let expected_count = graph.num_edges_from_into(&node_id, &other_node_id);
                actual_count == expected_count
            })
        })
    }

    pub fn prop_has_edge_from_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        graph.node_ids().all(|node_id| {
            let has_edge = graph.has_edge_from(&node_id);
            let expected_has_edge = graph.edges_from(&node_id).next().is_some();
            has_edge == expected_has_edge
        })
    }

    pub fn prop_has_edge_into_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        graph.node_ids().all(|node_id| {
            let has_edge = graph.has_edge_into(&node_id);
            let expected_has_edge = graph.edges_into(&node_id).next().is_some();
            has_edge == expected_has_edge
        })
    }

    pub fn prop_has_edge_from_into_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
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

    pub fn prop_is_empty_is_correct(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        let is_empty = graph.is_empty();
        let expected_is_empty = graph.node_ids().next().is_none();
        is_empty == expected_is_empty
    }

    pub fn prop_clear_removes_all_nodes_and_edges(
        ArbGraph { mut graph }: ArbGraph<TestGraph<B>>,
    ) -> bool {
        graph.clear();
        graph.node_ids().next().is_none() && graph.edge_ids().next().is_none()
    }

    pub fn prop_no_orphan_edges(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        let all_edges = graph.edge_ids().collect::<HashSet<_>>();
        let from_edges = graph
            .node_ids()
            .flat_map(|node_id| graph.edges_from(&node_id).collect::<Vec<_>>())
            .collect::<HashSet<_>>();
        let into_edges = graph
            .node_ids()
            .flat_map(|node_id| graph.edges_into(&node_id).collect::<Vec<_>>())
            .collect::<HashSet<_>>();
        all_edges == from_edges && all_edges == into_edges
    }

    pub fn prop_remove_node_removes_edges(ArbGraph { mut graph }: ArbGraph<TestGraph<B>>) -> bool {
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
            if graph.num_nodes() != num_nodes - 1 {
                return false;
            }
            if graph.num_edges() > num_edges - num_node_edges {
                return false;
            }
        }
        true
    }

    pub fn prop_edges_in_and_out_are_consistent(
        ArbGraph { graph }: ArbGraph<TestGraph<B>>,
    ) -> bool {
        for node_id in graph.node_ids() {
            for edge_from in graph.edges_from(&node_id) {
                let other_node = edge_from.other_end(&node_id);
                if !graph.edges_into(&other_node).any(|e| e == edge_from) {
                    return false;
                }
            }
            for edge_into in graph.edges_into(&node_id) {
                let other_node = edge_into.other_end(&node_id);
                if !graph.edges_from(&other_node).any(|e| e == edge_into) {
                    return false;
                }
            }
        }
        true
    }

    pub fn prop_edges_from_into_is_consistent(ArbGraph { graph }: ArbGraph<TestGraph<B>>) -> bool {
        for node_id in graph.node_ids() {
            for other_node_id in graph.node_ids() {
                for edge_from_into in graph.edges_from_into(&node_id, &other_node_id) {
                    if !graph.edges_from(&node_id).any(|e| e == edge_from_into) {
                        return false;
                    }
                    if !graph
                        .edges_into(&other_node_id)
                        .any(|e| e == edge_from_into)
                    {
                        return false;
                    }
                }
            }
        }
        true
    }

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
            (true, true) => 6454,
            (true, false) => 6439,
            (false, true) => 6454,
            (false, false) => 6383,
        };
        assert_eq!(
            num_edges, expected_edges,
            "Expected {} edges based on graph properties",
            expected_edges
        );

        // Verify all nodes are valid
        for node_id in graph.node_ids() {
            assert_eq!(graph.check_valid_node_id(&node_id), Ok(()));
        }

        // Verify all edges are valid
        for edge_id in graph.edge_ids() {
            assert_eq!(graph.check_valid_edge_id(&edge_id), Ok(()));
        }

        // Count edges to verify consistency
        let edge_count_via_iteration = graph.edge_ids().count();
        assert_eq!(
            edge_count_via_iteration, num_edges,
            "Edge count mismatch: num_edges() returned {} but iteration counted {}",
            num_edges, edge_count_via_iteration
        );
    }

    #[cfg(feature = "slow_tests")]
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
        let mut node_ids = graph.node_ids().collect::<std::collections::HashSet<_>>();

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
                    graph.compact_with(
                        |old_id, new_id| {
                            let removed = node_ids.remove(old_id);
                            assert!(removed);
                            let inserted = node_ids.insert(new_id.clone());
                            assert!(inserted);
                        },
                        |_, _new_e| {},
                    );
                }
                assert_eq!(graph.num_nodes(), num_nodes);
                assert_eq!(graph.num_edges(), num_edges);
                {
                    let _span = info_span!("check_valid_ids").entered();
                    for node_id in graph.node_ids() {
                        assert_eq!(graph.check_valid_node_id(&node_id), Ok(()));
                    }
                    for edge_id in graph.edge_ids() {
                        assert_eq!(graph.check_valid_edge_id(&edge_id), Ok(()));
                    }
                }
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
            generate_large_graph(&self.builder.builder)
        };

        // We use a hash set instead of a vec so the edges are removed in
        // random order.
        let mut edge_ids = graph.edge_ids().collect::<std::collections::HashSet<_>>();

        let _remove_loop_span = info_span!("remove_edges_loop").entered();
        for i in 0..edge_ids.len() {
            // Test compaction periodically
            if i % 250 == 0 {
                let num_nodes = graph.num_nodes();
                let num_edges = edge_ids.len();

                {
                    let _span = info_span!("compact").entered();
                    graph.compact_with(
                        |_, _| {},
                        |old_id, new_id| {
                            let removed = edge_ids.remove(old_id);
                            assert!(removed);
                            let inserted = edge_ids.insert(new_id.clone());
                            assert!(inserted);
                        },
                    );
                }
                assert_eq!(graph.num_nodes(), num_nodes);
                assert_eq!(graph.num_edges(), num_edges);
                {
                    let _span = info_span!("check_valid_ids").entered();
                    for node_id in graph.node_ids() {
                        assert_eq!(graph.check_valid_node_id(&node_id), Ok(()));
                    }
                    for edge_id in graph.edge_ids() {
                        assert_eq!(graph.check_valid_edge_id(&edge_id), Ok(()));
                    }
                }
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

    pub fn test_new_graph_is_empty(&mut self) {
        let builder = &mut self.builder;
        let graph = builder.new_graph();
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }

    pub fn test_node_data_retrieval(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let n1 = graph.add_node(nd1.clone());
        assert_eq!(*graph.node_data(&n1), nd1);
    }

    pub fn test_node_data_mutation(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let n1 = graph.add_node(nd1.clone());
        *graph.node_data_mut(&n1) = nd2.clone();
        assert_eq!(*graph.node_data(&n1), nd2);
    }

    pub fn test_edge_data_retrieval(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();
        assert_eq!(*graph.edge_data(&e1), ed1);
    }

    pub fn test_edge_data_mutation(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();
        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();
        *graph.edge_data_mut(&e1) = ed2.clone();
        assert_eq!(*graph.edge_data(&e1), ed2);
    }

    pub fn test_edge_creation(&mut self) {
        use crate::EdgeIdTrait;
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
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();
        let e2 = graph.add_edge(&n2, &n3, ed2.clone()).unwrap();

        // Check edges_from and num_edges_from for each node
        if graph.is_directed() {
            assert_eq!(graph.num_edges_from(&n1), 1);
            assert_eq!(graph.num_edges_from(&n2), 1);
            assert_eq!(graph.num_edges_from(&n3), 0);
            assert_eq!(
                graph
                    .edges_from(&n1)
                    .map(|edge_id| edge_id.other_end(&n1))
                    .collect::<Vec<_>>(),
                vec![n2.clone()]
            );
            assert_eq!(
                graph
                    .edges_from(&n2)
                    .map(|edge_id| edge_id.other_end(&n2))
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
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();
        let e2 = graph.add_edge(&n1, &n3, ed2.clone()).unwrap();

        let edge_ids: Vec<_> = graph.edge_ids().collect();
        assert_eq!(edge_ids.len(), 2);
        assert!(edge_ids.contains(&e1));
        assert!(edge_ids.contains(&e2));
    }

    pub fn test_edges_by_node(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();
        let ed2 = self.new_edge_data();

        let n1 = graph.add_node(nd1.clone());
        let n2 = graph.add_node(nd2.clone());

        // Normal edge.
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();
        assert_eq!(graph.num_edges(), 1);
        // Self edge.
        let e2 = graph.add_edge(&n1, &n1, ed2.clone()).unwrap();
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
            assert!(edges_from_n1.contains(&add_3.clone().unwrap()));
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
                assert!(edges_into_n1.contains(&add_3.unwrap()));
                assert_eq!(edges_into_n1.len(), 3);
            } else {
                assert!(matches!(&add_3, AddEdgeResult::Updated(_, data) if *data == ed1));
                assert_eq!(edges_into_n1.len(), 2);
            }
        }
    }

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

    pub fn test_edges_from(&mut self) {
        use std::collections::HashSet;

        let mut graph = self.new_graph();
        let n0 = graph.add_node(self.new_node_data());
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());
        let n3 = graph.add_node(self.new_node_data());

        let e0 = graph.add_edge(&n0, &n1, self.new_edge_data()).unwrap();
        let e1 = graph.add_edge(&n0, &n2, self.new_edge_data()).unwrap();
        let e2 = graph.add_edge(&n1, &n2, self.new_edge_data()).unwrap();
        let e3 = graph.add_edge(&n1, &n3, self.new_edge_data()).unwrap();
        let e4 = graph.add_edge(&n2, &n3, self.new_edge_data()).unwrap();
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

    pub fn test_edges_into(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();

        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();

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

    pub fn test_edges_between(&mut self) {
        let mut graph = self.new_graph();
        let nd1 = self.new_node_data();
        let nd2 = self.new_node_data();
        let ed1 = self.new_edge_data();

        let n1 = graph.add_node(nd1);
        let n2 = graph.add_node(nd2);
        let e1 = graph.add_edge(&n1, &n2, ed1).unwrap();

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

    pub fn test_copy_from(&mut self) {
        let mut source = self.new_graph();
        let n1 = source.add_node(self.new_node_data());
        let n2 = source.add_node(self.new_node_data());
        let n3 = source.add_node(self.new_node_data());
        let e1 = source.add_edge(&n1, &n2, self.new_edge_data()).unwrap();
        let e2 = source.add_edge(&n2, &n3, self.new_edge_data()).unwrap();

        let mut node_map = HashMap::new();
        let mut edge_map = HashMap::new();
        let target = GraphCopier::new(&source)
            .clone_nodes()
            .clone_edges()
            .with_node_map(&mut node_map)
            .with_edge_map(&mut edge_map)
            .copy::<TestGraph<B>>();

        assert_eq!(target.node_ids().count(), 3);
        assert_eq!(target.edge_ids().count(), 2);
        assert_eq!(source.node_data(&n1), target.node_data(&node_map[&n1]));
        assert_eq!(source.node_data(&n2), target.node_data(&node_map[&n2]));
        assert_eq!(source.node_data(&n3), target.node_data(&node_map[&n3]));
        assert_eq!(source.edge_data(&e1), target.edge_data(&edge_map[&e1]));
        assert_eq!(source.edge_data(&e2), target.edge_data(&edge_map[&e2]));
    }

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

    pub fn test_successors(&mut self) {
        use crate::EdgeIdTrait;
        use std::collections::HashSet;

        let mut graph = self.new_graph();
        let n0 = graph.add_node(self.new_node_data());
        let n1 = graph.add_node(self.new_node_data());
        let n2 = graph.add_node(self.new_node_data());
        let n3 = graph.add_node(self.new_node_data());

        let e0 = graph.add_edge(&n0, &n1, self.new_edge_data()).unwrap();
        let e1 = graph.add_edge(&n0, &n2, self.new_edge_data()).unwrap();
        let e2 = graph.add_edge(&n1, &n2, self.new_edge_data()).unwrap();
        let e3 = graph.add_edge(&n1, &n3, self.new_edge_data()).unwrap();
        let e4 = graph.add_edge(&n2, &n3, self.new_edge_data()).unwrap();
        if graph.is_directed() {
            assert_eq!(e0.ends(), (n0.clone(), n1.clone()));
            assert_eq!(e1.ends(), (n0.clone(), n2.clone()));
            assert_eq!(e2.ends(), (n1.clone(), n2.clone()));
            assert_eq!(e3.ends(), (n1.clone(), n3.clone()));
            assert_eq!(e4.ends(), (n2.clone(), n3.clone()));
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
                    edge.has_ends(&a, &b),
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
        let e1 = graph.add_edge(&n1, &n1, ed1.clone()).unwrap();
        let e2 = graph.add_edge(&n1, &n2, ed2.clone()).unwrap();
        let _e3 = graph.add_edge(&n2, &n3, ed3.clone()).unwrap();
        graph.remove_node(&n3);
        assert_eq!(graph.node_ids().count(), 2);
        assert_eq!(graph.edge_ids().count(), 2);

        let mut nid_map = std::collections::HashMap::new();
        let mut eid_map = std::collections::HashMap::new();
        graph.compact_with(
            |old_nid, new_nid| {
                nid_map.insert(old_nid.clone(), new_nid.clone());
            },
            |old_eid, new_eid| {
                eid_map.insert(old_eid.clone(), new_eid.clone());
            },
        );
        assert_eq!(graph.node_ids().count(), 2);
        assert_eq!(graph.edge_ids().count(), 2);

        // After compaction, we need to use the new node ID.
        // Find the node with the same data
        assert_eq!(graph.node_data(nid_map.get(&n1).unwrap_or(&n1)), &nd1);
        assert_eq!(graph.node_data(nid_map.get(&n2).unwrap_or(&n2)), &nd2);
        assert_eq!(graph.edge_data(eid_map.get(&e1).unwrap_or(&e1)), &ed1);
        assert_eq!(graph.edge_data(eid_map.get(&e2).unwrap_or(&e2)), &ed2);
    }

    pub fn test_copy_from_with(&mut self) {
        let mut source = self.new_graph();
        let n1 = source.add_node(self.new_node_data());
        let n2 = source.add_node(self.new_node_data());
        let n3 = source.add_node(self.new_node_data());
        let e0 = source.add_edge(&n1, &n2, self.new_edge_data()).unwrap();
        let e1 = source.add_edge(&n2, &n3, self.new_edge_data()).unwrap();

        let mut node_map = HashMap::new();
        let mut edge_map = HashMap::new();
        let target = GraphCopier::new(&source)
            .transform_nodes(&mut self.transform_node)
            .transform_edges(&mut self.transform_edge)
            .with_node_map(&mut node_map)
            .with_edge_map(&mut edge_map)
            .copy::<TestGraph<B>>();

        assert_eq!(target.node_ids().count(), 3);
        assert_eq!(target.edge_ids().count(), 2);
        assert_eq!(
            (self.transform_node)(source.node_data(&n1)),
            *target.node_data(&node_map[&n1])
        );
        assert_eq!(
            (self.transform_node)(source.node_data(&n2)),
            *target.node_data(&node_map[&n2])
        );
        assert_eq!(
            (self.transform_node)(source.node_data(&n3)),
            *target.node_data(&node_map[&n3])
        );
        assert_eq!(
            (self.transform_edge)(source.edge_data(&e0)),
            *target.edge_data(&edge_map[&e0])
        );
        assert_eq!(
            (self.transform_edge)(source.edge_data(&e1)),
            *target.edge_data(&edge_map[&e1])
        );
    }

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

/// Macro to generate standard graph tests for a given graph type.
#[macro_export]
macro_rules! graph_tests {
    ($name:ident, $builder_type:ty, $builder:expr, $f:expr, $g:expr $(; $($rest:tt)*)?) => {
        mod $name {
            use super::*;
            use $crate::graph_tests::*;

            $($($rest)*)?

            macro_rules! quickcheck_test {
                ($test_name:ident) => {
                    #[test]
                    fn $test_name() {
                        let f: fn(_) -> _ = GraphTests::<$builder_type>::$test_name;
                        quickcheck::quickcheck(f);
                    }
                };
            }

            macro_rules! builder_test {
                ($test_name:ident) => {
                    #[test]
                    fn $test_name() {
                        GraphTests::<$builder_type> {
                            builder: $builder,
                            next_node_index: 0,
                            next_edge_index: 0,
                            transform_node: Box::new($f),
                            transform_edge: Box::new($g),
                        }.$test_name();
                    }
                };
            }

            quickcheck_test!(prop_node_ids_are_valid);
            quickcheck_test!(prop_edge_ids_are_valid);
            quickcheck_test!(prop_num_nodes_is_correct);
            quickcheck_test!(prop_num_edges_is_correct);
            quickcheck_test!(prop_node_ids_are_unique);
            quickcheck_test!(prop_edge_ids_are_unique);
            quickcheck_test!(prop_edges_from_returns_unique_values);
            quickcheck_test!(prop_edges_into_returns_unique_values);
            quickcheck_test!(prop_edges_from_into_returns_unique_values);
            quickcheck_test!(prop_edges_from_into_finds_all_edges);
            quickcheck_test!(prop_num_edges_from_is_correct);
            quickcheck_test!(prop_num_edges_into_is_correct);
            quickcheck_test!(prop_num_edges_from_into_is_correct);
            quickcheck_test!(prop_has_edge_from_is_correct);
            quickcheck_test!(prop_has_edge_into_is_correct);
            quickcheck_test!(prop_has_edge_from_into_is_correct);
            quickcheck_test!(prop_is_empty_is_correct);
            quickcheck_test!(prop_clear_removes_all_nodes_and_edges);
            quickcheck_test!(prop_no_orphan_edges);
            quickcheck_test!(prop_remove_node_removes_edges);
            quickcheck_test!(prop_edges_in_and_out_are_consistent);
            quickcheck_test!(prop_edges_from_into_is_consistent);

            builder_test!(test_large_graph_structure);
            #[cfg(feature = "slow_tests")]
            builder_test!(test_deconstruct_large_graph_by_nodes);
            #[cfg(feature = "slow_tests")]
            builder_test!(test_deconstruct_large_graph_by_edges);
            builder_test!(test_node_data_mutation);
            builder_test!(test_edge_data_mutation);
            builder_test!(test_edge_creation);
            builder_test!(test_edge_ids);
            builder_test!(test_edges_by_node);
            builder_test!(test_node_removal);
            builder_test!(test_remove_node_cleans_edges);
            builder_test!(test_edges_from);
            builder_test!(test_edges_into);
            builder_test!(test_edges_between);
            builder_test!(test_copy_from);
            builder_test!(test_clear);
            builder_test!(test_successors);
            builder_test!(test_predecessors);
            #[cfg(feature = "pathfinding")]
            builder_test!(test_shortest_paths);
            #[cfg(feature = "pathfinding")]
            builder_test!(test_shortest_paths_disconnected);
            builder_test!(test_compaction);
            builder_test!(test_copy_from_with);
            builder_test!(test_edge_multiplicity);
        }
    };
}
