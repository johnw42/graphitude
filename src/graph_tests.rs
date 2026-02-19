use std::collections::HashSet;
use std::hash::Hash;

use quickcheck::Arbitrary;

use crate::prelude::*;
use crate::tracing_support::{TimingScope, info_span, init_tracing, set_timing_scope};

/// Trait for building test data for graphs.  Graph implementations used in
/// tests should implement this trait to provide consistent node and edge data.
pub trait TestDataBuilder {
    type Graph: Graph;

    /// Creates a new graph instance for testing.
    fn new_graph(&self) -> Self::Graph;

    /// Creates new edge data for testing, given an index.  Tests will call this
    /// method with consecutive indices starting from zero.
    fn new_edge_data(&self, i: usize) -> <Self::Graph as Graph>::EdgeData;

    /// Creates new node data for testing, given an index.  Tests will call
    /// this method with consecutive indices starting from zero.
    fn new_node_data(&self, i: usize) -> <Self::Graph as Graph>::NodeData;
}

/// Internal implementation of test data builder.
///
/// This type should not be used directly; use the test macros instead.
#[doc(hidden)]
pub struct InternalBuilder<B> {
    next_node_index: usize,
    next_edge_index: usize,
    builder: B,
}

impl<B> InternalBuilder<B>
where
    B: TestDataBuilder,
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
}

impl<B> From<B> for InternalBuilder<B>
where
    B: TestDataBuilder,
{
    fn from(builder: B) -> Self {
        Self {
            next_node_index: 0,
            next_edge_index: 0,
            builder,
        }
    }
}

/// Generates a large graph with an irregular structure using custom closures
/// for node and edge data generation.
///
/// The graph structure includes:
/// - Cluster 1: Dense cluster (50 nodes, ~60% connectivity)
/// - Cluster 2: Medium cluster (80 nodes, ~30% connectivity)
/// - Cluster 3: Large sparse cluster (150 nodes, ~8% connectivity)
/// - Hub nodes (20 nodes with many connections)
/// - Scattered nodes (100 nodes with few connections)
/// - Bridge nodes connecting clusters (10 nodes)
/// - Long-range connections between random nodes
/// - Self loops
///
/// The resulting graph has approximately 500 nodes and 2000 edges.
///
/// # Arguments
///
/// * `new_node_data` - A closure that takes an index and returns node data
/// * `new_edge_data` - A closure that takes an index and returns edge data
pub fn generate_large_graph_with<G, FN, FE>(
    graph: &mut G,
    mut new_node_data: FN,
    mut new_edge_data: FE,
) where
    G: GraphMut,
    FN: FnMut(usize) -> <G as Graph>::NodeData,
    FE: FnMut(usize) -> <G as Graph>::EdgeData,
{
    let mut node_counter = 0;
    let mut edge_counter = 0;

    // Create an irregular graph with ~500 nodes and ~2000 edges
    // Structure includes: clusters, hubs, sparse regions, and bridges

    let mut all_nodes = Vec::new();

    // Cluster 1: Dense cluster (50 nodes, highly connected)
    let cluster1_start = all_nodes.len();
    for _ in 0..50 {
        let node = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(node);
    }

    // Connect nodes within cluster 1 with ~60% density
    for i in cluster1_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 7 + j * 11) % 10 < 6 {
                graph.add_edge(&all_nodes[i], &all_nodes[j], new_edge_data(edge_counter));
                edge_counter += 1;
            }
        }
    }

    // Cluster 2: Medium cluster (80 nodes, moderately connected)
    let cluster2_start = all_nodes.len();
    for _ in 0..80 {
        let node = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(node);
    }
    // Connect nodes within cluster 2 with ~30% density
    for i in cluster2_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 13 + j * 17) % 10 < 3 {
                graph.add_edge(&all_nodes[i], &all_nodes[j], new_edge_data(edge_counter));
                edge_counter += 1;
            }
        }
    }

    // Cluster 3: Large sparse cluster (150 nodes, sparsely connected)
    let cluster3_start = all_nodes.len();
    for _ in 0..150 {
        let node = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(node);
    }
    // Connect nodes within cluster 3 with ~8% density
    for i in cluster3_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 19 + j * 23) % 100 < 8 {
                graph.add_edge(&all_nodes[i], &all_nodes[j], new_edge_data(edge_counter));
                edge_counter += 1;
            }
        }
    }

    // Add hub nodes (20 nodes with many connections)
    let hubs_start = all_nodes.len();
    for _ in 0..20 {
        let hub = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(hub.clone());

        // Connect each hub to random existing nodes
        #[allow(clippy::needless_range_loop)]
        for i in 0..all_nodes.len() - 1 {
            if (hubs_start * 29 + i * 31) % 7 < 4 {
                graph.add_edge(&hub, &all_nodes[i], new_edge_data(edge_counter));
                edge_counter += 1;
            }
        }
    }

    // Add scattered nodes (100 nodes with few connections)
    let scattered_start = all_nodes.len();
    for _ in 0..100 {
        let node = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(node.clone());

        // Connect to 1-3 random other nodes
        let num_connections = ((scattered_start + all_nodes.len()) % 3) + 1;
        for c in 0..num_connections {
            let target_idx =
                (scattered_start * 37 + all_nodes.len() * 41 + c * 43) % (all_nodes.len() - 1);
            graph.add_edge(&node, &all_nodes[target_idx], new_edge_data(edge_counter));
            edge_counter += 1;
        }
    }

    // Add bridge nodes connecting clusters (10 nodes)
    for i in 0..10 {
        let bridge = graph.add_node(new_node_data(node_counter));
        node_counter += 1;

        // Connect to nodes from different clusters
        let idx1 = (i * 47) % (cluster2_start - cluster1_start) + cluster1_start;
        let idx2 = (i * 53) % (cluster3_start - cluster2_start) + cluster2_start;
        let idx3 = (i * 59) % (hubs_start - cluster3_start) + cluster3_start;

        graph.add_edge(&bridge, &all_nodes[idx1], new_edge_data(edge_counter));
        edge_counter += 1;
        graph.add_edge(&bridge, &all_nodes[idx2], new_edge_data(edge_counter));
        edge_counter += 1;
        graph.add_edge(&bridge, &all_nodes[idx3], new_edge_data(edge_counter));
        edge_counter += 1;

        all_nodes.push(bridge);
    }

    // Add some long-range connections between random nodes
    for i in 0..200 {
        let idx1 = (i * 61) % all_nodes.len();
        let idx2 = (i * 67 + 100) % all_nodes.len();
        if idx1 != idx2 {
            graph.add_edge(
                &all_nodes[idx1],
                &all_nodes[idx2],
                new_edge_data(edge_counter),
            );
            edge_counter += 1;
        }
    }

    // Add reciprocal edge loops between pairs of nodes
    for i in 0..50 {
        let idx1 = (i * 73 + 7) % all_nodes.len();
        let idx2 = (i * 79 + 11) % all_nodes.len();
        if idx1 == idx2 {
            continue;
        }
        graph.add_edge(
            &all_nodes[idx1],
            &all_nodes[idx2],
            new_edge_data(edge_counter),
        );
        edge_counter += 1;
        graph.add_edge(
            &all_nodes[idx2],
            &all_nodes[idx1],
            new_edge_data(edge_counter),
        );
        edge_counter += 1;
    }

    // Add some self loops
    for i in 0..50 {
        let idx = (i * 71) % all_nodes.len();
        graph.add_edge(
            &all_nodes[idx],
            &all_nodes[idx],
            new_edge_data(edge_counter),
        );
        edge_counter += 1;
    }
}

/// Generates a large graph using the `TestDataBuilder` trait for data generation.
///
/// This is a convenience wrapper around [`generate_large_graph_with`] that uses
/// the TestDataBuilder trait to provide node and edge data.
#[doc(hidden)]
pub fn generate_large_graph<B>(builder: &B) -> B::Graph
where
    B: TestDataBuilder,
    B::Graph: GraphMut,
{
    let mut graph = builder.new_graph();
    generate_large_graph_with::<B::Graph, _, _>(
        &mut graph,
        |i| builder.new_node_data(i),
        |i| builder.new_edge_data(i),
    );
    graph
}

/// Checks the internal consistency of a graph.
#[doc(hidden)]
pub fn check_graph_consistency<G: Graph>(graph: &G) {
    let _scope = set_timing_scope(TimingScope::Consistency);
    init_tracing();
    if graph.is_very_slow() {
        eprintln!("Skipping consistency check for very slow graph implementation.");
        return;
    }

    // Verify all nodes are valid
    for node_id in graph.node_ids() {
        {
            let _span = info_span!("check_valid_node_id").entered();
            let valid = graph.check_valid_node_id(&node_id);
            assert_eq!(valid, Ok(()));
        }

        let num_from = {
            let _span = info_span!("num_edges_from").entered();
            graph.num_edges_from(&node_id)
        };

        let edges_from_count = {
            let _span = info_span!("edges_from.count").entered();
            graph.edges_from(&node_id).count()
        };

        let num_into = {
            let _span = info_span!("num_edges_into").entered();
            graph.num_edges_into(&node_id)
        };

        let edges_into_count = {
            let _span = info_span!("edges_into.count").entered();
            graph.edges_into(&node_id).count()
        };

        assert_eq!(num_from, edges_from_count);
        assert_eq!(num_into, edges_into_count);

        let has_from = {
            let _span = info_span!("has_edge_from").entered();
            graph.has_edge_from(&node_id)
        };

        let has_into = {
            let _span = info_span!("has_edge_into").entered();
            graph.has_edge_into(&node_id)
        };

        assert_eq!(has_from, num_from > 0);
        assert_eq!(has_into, num_into > 0);
    }

    // Verify all edges are valid
    for edge_id in graph.edge_ids() {
        assert_eq!(
            graph
                .edges_from_into(&edge_id.left(), &edge_id.right())
                .count(),
            graph
                .edges_from_into(&edge_id.left(), &edge_id.right())
                .collect::<HashSet<_>>()
                .len()
        );

        if !graph.is_directed() {
            assert!(graph.has_edge_from_into(&edge_id.right(), &edge_id.left()))
        }
        if !graph.allows_parallel_edges() {
            dbg!(&edge_id.left(), &edge_id.right());
            assert_eq!(
                graph.num_edges_from_into(&edge_id.left(), &edge_id.right()),
                1
            );
        }

        {
            let _span = info_span!("check_valid_edge_id").entered();
            let valid = graph.check_valid_edge_id(&edge_id);
            assert_eq!(valid, Ok(()));
        }

        {
            let _span = info_span!("has_edge").entered();
            let has_edge = graph.has_edge_from_into(&edge_id.left(), &edge_id.right());
            assert!(has_edge);
        }

        {
            let _span = info_span!("edges_between.any").entered();
            let between_has = graph
                .edges_from_into(&edge_id.left(), &edge_id.right())
                .any(|e| e == edge_id);
            assert!(between_has);
        }

        {
            let _span = info_span!("edges_from.any").entered();
            let from_has = graph.edges_from(&edge_id.left()).any(|e| e == edge_id);
            assert!(from_has);
        }

        {
            let _span = info_span!("edges_into.any").entered();
            let into_has = graph.edges_into(&edge_id.right()).any(|e| e == edge_id);
            assert!(into_has);
        }

        let num_from = {
            let _span = info_span!("num_edges_from").entered();
            graph.num_edges_from(&edge_id.left())
        };

        let edges_from_count = {
            let _span = info_span!("edges_from.count").entered();
            graph.edges_from(&edge_id.left()).count()
        };

        let num_into = {
            let _span = info_span!("num_edges_into").entered();
            graph.num_edges_into(&edge_id.right())
        };

        let edges_into_count = {
            let _span = info_span!("edges_into.count").entered();
            graph.edges_into(&edge_id.right()).count()
        };

        assert_eq!(num_from, edges_from_count);
        assert_eq!(num_into, edges_into_count);
    }

    // Verify node and edge IDs are unique.
    let node_ids: HashSet<_> = graph.node_ids().collect();
    assert_eq!(node_ids.len(), graph.node_ids().count());
    let edge_ids: HashSet<_> = graph.edge_ids().collect();
    assert_eq!(edge_ids.len(), graph.edge_ids().count());

    // Verify counts are correct
    assert_eq!(graph.node_ids().count(), graph.num_nodes(),);
    assert_eq!(graph.edge_ids().count(), graph.num_edges(),);

    // Check is_empty consistency
    assert_eq!(graph.is_empty(), graph.num_nodes() == 0);

    // If there are edges, there must be nodes
    assert!(graph.num_nodes() > 0 || graph.num_edges() == 0);
}

#[derive(Debug, Clone)]
pub struct GraphWrapper<G>(pub G);

impl<G> Arbitrary for GraphWrapper<G>
where
    G: GraphMut + Clone + 'static,
    G::NodeData: Arbitrary + Clone + 'static,
    G::EdgeData: Arbitrary + Clone + 'static,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let num_nodes = usize::arbitrary(g) % 20; // Limit size for testing
        let num_edges = usize::arbitrary(g) % 50;
        let num_extra_parallel_edges = usize::arbitrary(g) % 5;
        let num_extra_self_loops = usize::arbitrary(g) % 5;

        let mut graph = G::new(
            G::Directedness::arbitrary(g),
            G::EdgeMultiplicity::arbitrary(g),
        );
        let nodes: Vec<_> = (0..num_nodes)
            .map(|_| graph.add_node(G::NodeData::arbitrary(g)))
            .collect();

        for i in 0..num_edges {
            if nodes.len() < 2 {
                break;
            }
            let source = nodes[usize::arbitrary(g) % nodes.len()].clone();
            let target = nodes[usize::arbitrary(g) % nodes.len()].clone();
            graph.add_edge(&source, &target, G::EdgeData::arbitrary(g));
            if i < num_extra_parallel_edges {
                graph.add_edge(&source, &target, G::EdgeData::arbitrary(g));
            }
            if i < num_extra_self_loops {
                graph.add_edge(&source, &source, G::EdgeData::arbitrary(g));
            }
        }

        GraphWrapper(graph)
    }
}

#[doc(hidden)]
pub fn has_duplicates<T: Eq + Hash>(items: impl IntoIterator<Item = T>) -> bool {
    let mut seen = HashSet::new();
    for item in items {
        if !seen.insert(item) {
            return true;
        }
    }
    false
}

/// Macro to generate standard graph tests for a given graph type.
#[macro_export]
macro_rules! graph_tests {
    ($name:ident, $builder_type:ty, $builder:expr, $f:expr, $g:expr $(; $($rest:tt)*)?) => {
        mod $name {
            use super::*;
            use $crate::graph_tests::*;
            use $crate::GraphCopier;
            use std::collections::{HashMap, HashSet};
            use quickcheck_macros::quickcheck;

            type TestGraph = <$builder_type as TestDataBuilder>::Graph;
            type BuilderImpl = InternalBuilder<$builder_type>;

            $($($rest)*)?

            #[quickcheck]
            fn node_ids_are_valid(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph
                    .node_ids()
                    .all(|node_id| graph.check_valid_node_id(&node_id).is_ok())
            }

            #[quickcheck]
            fn edge_ids_are_valid(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph
                    .edge_ids()
                    .all(|edge_id| graph.check_valid_edge_id(&edge_id).is_ok())
            }

            #[quickcheck]
            fn prop_num_nodes_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                let actual_node_count = graph.node_ids().count();
                let expected_node_count = graph.num_nodes();
                actual_node_count == expected_node_count
            }

            #[quickcheck]
            fn prop_num_edges_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                let actual_edge_count = graph.edge_ids().count();
                let expected_edge_count = graph.num_edges();
                actual_edge_count == expected_edge_count
            }

            #[quickcheck]
            fn prop_node_ids_are_unique(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                !has_duplicates(graph.node_ids())
            }

            #[quickcheck]
            fn prop_edge_ids_are_unique(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                !has_duplicates(graph.edge_ids())
            }

            #[quickcheck]
            fn prop_edges_from_returns_unique_values(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph
                    .node_ids()
                    .all(|node_id| !has_duplicates(graph.edges_from(&node_id)))
            }

            #[quickcheck]
            fn prop_edges_into_returns_unique_values(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph
                    .node_ids()
                    .all(|node_id| !has_duplicates(graph.edges_into(&node_id)))
            }

            #[quickcheck]
            fn prop_edges_from_into_returns_unique_values(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph.node_ids().all(|node_id| {
                    graph.node_ids().all(|other_node_id| {
                        !has_duplicates(graph.edges_from_into(&node_id, &other_node_id))
                    })
                })
            }

            #[quickcheck]
            fn prop_edges_from_into_finds_all_edges(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph.edge_ids().all(|edge_id| {
                    let (left, right) = edge_id.ends();
                    graph
                        .edges_from_into(&left, &right)
                        .any(|e| e == edge_id)
                })
            }

            #[quickcheck]
            fn prop_num_edges_from_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph.node_ids().all(|node_id| {
                    let actual_count = graph.edges_from(&node_id).count();
                    let expected_count = graph.num_edges_from(&node_id);
                    actual_count == expected_count
                })
            }

            #[quickcheck]
            fn prop_num_edges_into_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph.node_ids().all(|node_id| {
                    let actual_count = graph.edges_into(&node_id).count();
                    let expected_count = graph.num_edges_into(&node_id);
                    actual_count == expected_count
                })
            }

            #[quickcheck]
            fn prop_num_edges_from_into_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph.node_ids().all(|node_id| {
                    graph.node_ids().all(|other_node_id| {
                        let actual_count = graph.edges_from_into(&node_id, &other_node_id).count();
                        let expected_count = graph.num_edges_from_into(&node_id, &other_node_id);
                        actual_count == expected_count
                    })
                })
            }

            #[quickcheck]
            fn prop_has_edge_from_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph.node_ids().all(|node_id| {
                    let has_edge = graph.has_edge_from(&node_id);
                    let expected_has_edge = graph.edges_from(&node_id).next().is_some();
                    has_edge == expected_has_edge
                })
            }

            #[quickcheck]
            fn prop_has_edge_into_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                graph.node_ids().all(|node_id| {
                    let has_edge = graph.has_edge_into(&node_id);
                    let expected_has_edge = graph.edges_into(&node_id).next().is_some();
                    has_edge == expected_has_edge
                })
            }

            #[quickcheck]
            fn prop_has_edge_from_into_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
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
            fn prop_is_empty_is_correct(
                GraphWrapper(graph): GraphWrapper<TestGraph>,
            ) -> bool {
                let is_empty = graph.is_empty();
                let expected_is_empty = graph.node_ids().next().is_none();
                is_empty == expected_is_empty
            }

            #[quickcheck]
            fn prop_clear_removes_all_nodes_and_edges(
                GraphWrapper(mut graph): GraphWrapper<
                    TestGraph,
                >,
            ) -> bool {
                graph.clear();
                graph.node_ids().next().is_none() && graph.edge_ids().next().is_none()
            }

            #[quickcheck]
            fn prop_no_orphan_edges(GraphWrapper(graph): GraphWrapper<TestGraph>) -> bool {
                let all_edges = graph.edge_ids().collect::<HashSet<_>>();
                let from_edges = graph
                    .node_ids()
                    .flat_map(|node_id| graph.edges_from(&node_id).collect::<Vec<_>>())
                    .collect::<HashSet<_>>();
                let into_edges = graph.node_ids()
                    .flat_map(|node_id| graph.edges_into(&node_id).collect::<Vec<_>>())
                    .collect::<HashSet<_>>();
                all_edges == from_edges && all_edges == into_edges
            }

            #[quickcheck]
            fn prop_remove_node_removes_edges(
                GraphWrapper(mut graph): GraphWrapper<TestGraph>,
            ) -> bool {
                let node_id = graph.node_ids().next();
                if let Some(node_id) = node_id {
                    let num_nodes = graph.num_nodes();
                    let num_edges = graph.num_edges();
                    let num_node_edges = graph.edges_from(&node_id)
                        .chain(graph.edges_into(&node_id))
                        .collect::<HashSet<_>>().len();
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

            #[quickcheck]
            fn prop_edges_in_and_out_are_consistent(GraphWrapper(graph): GraphWrapper<TestGraph>) -> bool {
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

            #[quickcheck]
            fn prop_edges_from_into_is_consistent(GraphWrapper(graph): GraphWrapper<TestGraph>) -> bool {
                for node_id in graph.node_ids() {
                    for other_node_id in graph.node_ids() {
                        for edge_from_into in graph.edges_from_into(&node_id, &other_node_id) {
                            if !graph.edges_from(&node_id).any(|e| e == edge_from_into) {
                                return false;
                            }
                            if !graph.edges_into(&other_node_id).any(|e| e == edge_from_into) {
                                return false;
                            }
                        }
                    }
                }
                true
            }

            #[test]
            fn test_large_graph_structure() {
                let builder: $builder_type = $builder;
                let graph: TestGraph = generate_large_graph(&builder);
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

            #[test]
            #[cfg(feature = "slow_tests")]
            fn test_deconstruct_large_graph_by_nodes() {
                use $crate::tracing_support::{
                    TimingScope, dump_method_timings, info_span, reset_method_timings, set_timing_scope,
                };

                reset_method_timings();
                let _scope = set_timing_scope(TimingScope::Test);
                let test_span = info_span!("test_deconstruct_large_graph_by_nodes");
                let _test_guard = test_span.entered();
                let builder: $builder_type = $builder;
                let mut graph: TestGraph = {
                    let _span = info_span!("generate_large_graph").entered();
                    $crate::tests::generate_large_graph(&builder)
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
                            $crate::tests::check_graph_consistency(&graph);
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

            #[test]
            #[cfg(feature = "slow_tests")]
            fn test_deconstruct_large_graph_by_edges() {
                use $crate::tracing_support::{
                    TimingScope, dump_method_timings, info_span, reset_method_timings, set_timing_scope,
                };

                reset_method_timings();
                let _scope = set_timing_scope(TimingScope::Test);
                let test_span = info_span!("test_deconstruct_large_graph_by_edges");
                let _test_guard = test_span.entered();
                let mut graph: TestGraph = {
                    let _span = info_span!("generate_large_graph").entered();
                    $crate::tests::generate_large_graph(&$builder)
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
                                    let removed = edge_ids.remove(&old_id);
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
                            $crate::tests::check_graph_consistency(&graph);
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
            fn test_new_graph_is_empty() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                assert_eq!(graph.num_nodes(), 0);
                assert_eq!(graph.num_edges(), 0);
            }

            #[test]
            fn test_node_data_retrieval() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let n1 = graph.add_node(nd1.clone());
                assert_eq!(*graph.node_data(&n1), nd1);
            }

            #[test]
            fn test_node_data_mutation() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let n1 = graph.add_node(nd1.clone());
                *graph.node_data_mut(&n1) = nd2.clone();
                assert_eq!(*graph.node_data(&n1), nd2);
            }

            #[test]
            fn test_edge_data_retrieval() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let ed1 = builder.new_edge_data();
                let n1 = graph.add_node(nd1);
                let n2 = graph.add_node(nd2);
                let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();
                assert_eq!(*graph.edge_data(&e1), ed1);
            }

            #[test]
            fn test_edge_data_mutation() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let ed1 = builder.new_edge_data();
                let ed2 = builder.new_edge_data();
                let n1 = graph.add_node(nd1);
                let n2 = graph.add_node(nd2);
                let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();
                *graph.edge_data_mut(&e1) = ed2.clone();
                assert_eq!(*graph.edge_data(&e1), ed2);
            }

            #[test]
            fn test_edge_creation() {
                use std::collections::HashSet;
                use $crate::EdgeIdTrait;

                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let nd3 = builder.new_node_data();
                let ed1 = builder.new_edge_data();
                let ed2 = builder.new_edge_data();
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

            #[test]
            fn test_edge_ids() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let nd3 = builder.new_node_data();
                let ed1 = builder.new_edge_data();
                let ed2 = builder.new_edge_data();
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


            #[test]
            fn test_edges_by_node() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let ed1 = builder.new_edge_data();
                let ed2 = builder.new_edge_data();
                let ed3 = builder.new_edge_data();

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


            #[test]
            fn test_node_removal() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let ed1 = builder.new_edge_data();
                let ed2 = builder.new_edge_data();
                let ed3 = builder.new_edge_data();

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
            fn test_remove_node_cleans_edges() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let ed1 = builder.new_edge_data();
                let ed2 = builder.new_edge_data();
                let ed3 = builder.new_edge_data();

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
            fn test_edges_from() {
                use std::collections::HashSet;

                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let n0 = graph.add_node(builder.new_node_data());
                let n1 = graph.add_node(builder.new_node_data());
                let n2 = graph.add_node(builder.new_node_data());
                let n3 = graph.add_node(builder.new_node_data());

                let e0 = graph.add_edge(&n0, &n1, builder.new_edge_data()).unwrap();
                let e1 = graph.add_edge(&n0, &n2, builder.new_edge_data()).unwrap();
                let e2 = graph.add_edge(&n1, &n2, builder.new_edge_data()).unwrap();
                let e3 = graph.add_edge(&n1, &n3, builder.new_edge_data()).unwrap();
                let e4 = graph.add_edge(&n2, &n3, builder.new_edge_data()).unwrap();
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
            fn test_edges_into() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let ed1 = builder.new_edge_data();

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

            #[test]
            fn test_edges_between() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let ed1 = builder.new_edge_data();

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

            #[test]
            fn test_copy_from() {
                let mut builder = BuilderImpl::from($builder);
                let mut source = builder.new_graph();
                let n1 = source.add_node(builder.new_node_data());
                let n2 = source.add_node(builder.new_node_data());
                let n3 = source.add_node(builder.new_node_data());
                let e1 = source.add_edge(&n1, &n2, builder.new_edge_data()).unwrap();
                let e2 = source.add_edge(&n2, &n3, builder.new_edge_data()).unwrap();

                let mut node_map = HashMap::new();
                let mut edge_map = HashMap::new();
                let target = GraphCopier::new(&source)
                    .clone_nodes()
                    .clone_edges()
                    .with_node_map(&mut node_map)
                    .with_edge_map(&mut edge_map)
                    .copy::<TestGraph>();

                assert_eq!(target.node_ids().count(), 3);
                assert_eq!(target.edge_ids().count(), 2);
                assert_eq!(source.node_data(&n1), target.node_data(&node_map[&n1]));
                assert_eq!(source.node_data(&n2), target.node_data(&node_map[&n2]));
                assert_eq!(source.node_data(&n3), target.node_data(&node_map[&n3]));
                assert_eq!(source.edge_data(&e1), target.edge_data(&edge_map[&e1]));
                assert_eq!(source.edge_data(&e2), target.edge_data(&edge_map[&e2]));
            }

            #[test]
            fn test_clear() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let n1 = graph.add_node(builder.new_node_data());
                let n2 = graph.add_node(builder.new_node_data());
                graph.add_edge(&n1, &n2, builder.new_edge_data());

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
            fn test_successors() {
                use std::collections::HashSet;
                use $crate::{EdgeIdTrait};

                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let n0 = graph.add_node(builder.new_node_data());
                let n1 = graph.add_node(builder.new_node_data());
                let n2 = graph.add_node(builder.new_node_data());
                let n3 = graph.add_node(builder.new_node_data());

                let e0 = graph.add_edge(&n0, &n1, builder.new_edge_data()).unwrap();
                let e1 = graph.add_edge(&n0, &n2, builder.new_edge_data()).unwrap();
                let e2 = graph.add_edge(&n1, &n2, builder.new_edge_data()).unwrap();
                let e3 = graph.add_edge(&n1, &n3, builder.new_edge_data()).unwrap();
                let e4 = graph.add_edge(&n2, &n3, builder.new_edge_data()).unwrap();
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
                        assert!(edge.has_ends(&a, &b), "Edge {:?} does not connect nodes {:?} and {:?}", edge, a, b);
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
            fn test_predecessors() {
                use $crate::{EdgeIdTrait};
                use std::collections::HashSet;

                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let n0 = graph.add_node(builder.new_node_data());
                let n1 = graph.add_node(builder.new_node_data());
                let n2 = graph.add_node(builder.new_node_data());
                let n3 = graph.add_node(builder.new_node_data());

                let e0 = graph.add_edge(&n0, &n1, builder.new_edge_data()).unwrap();
                let e1 = graph.add_edge(&n0, &n2, builder.new_edge_data()).unwrap();
                let e2 = graph.add_edge(&n1, &n2, builder.new_edge_data()).unwrap();
                let e3 = graph.add_edge(&n1, &n3, builder.new_edge_data()).unwrap();
                let e4 = graph.add_edge(&n2, &n3, builder.new_edge_data()).unwrap();

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

            #[test]
            #[cfg(feature = "pathfinding")]
            fn test_shortest_paths() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let n0 = graph.add_node(builder.new_node_data());
                let n1 = graph.add_node(builder.new_node_data());
                let n2 = graph.add_node(builder.new_node_data());
                let n3 = graph.add_node(builder.new_node_data());

                graph.add_edge(&n0, &n1, builder.new_edge_data());
                graph.add_edge(&n0, &n2, builder.new_edge_data());
                graph.add_edge(&n1, &n2, builder.new_edge_data());
                graph.add_edge(&n1, &n3, builder.new_edge_data());
                graph.add_edge(&n2, &n3, builder.new_edge_data());

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
                    paths[&n3].0.nodes().collect::<Vec<_>>()
                        == vec![n0.clone(), n1.clone(), n3.clone()]
                        || paths[&n3].0.nodes().collect::<Vec<_>>()
                            == vec![n0.clone(), n2.clone(), n3.clone()]
                );
                assert_eq!(paths[&n3].1, 2);
            }

            #[test]
            #[cfg(feature = "pathfinding")]
            fn test_shortest_paths_disconnected() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let n0 = graph.add_node(builder.new_node_data());
                let n1 = graph.add_node(builder.new_node_data());
                let n2 = graph.add_node(builder.new_node_data());

                graph.add_edge(&n0, &n1, builder.new_edge_data());

                let paths = graph.shortest_paths(&n0, |_| 1);
                assert_eq!(paths.get(&n0).map(|(_, dist)| *dist), Some(0));
                assert_eq!(paths.get(&n1).map(|(_, dist)| *dist), Some(1));
                assert_eq!(paths.get(&n2).map(|(_, dist)| *dist), None);
            }

            #[test]
            fn test_compaction() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let nd1 = builder.new_node_data();
                let nd2 = builder.new_node_data();
                let ed1 = builder.new_edge_data();
                let ed2 = builder.new_edge_data();
                let ed3 = builder.new_edge_data();

                let n1 = graph.add_node(nd1.clone());
                let n2 = graph.add_node(nd2.clone());
                let n3 = graph.add_node(builder.new_node_data());
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

            #[test]
            fn test_copy_from_with() {
                let mut builder = BuilderImpl::from($builder);
                let mut source = builder.new_graph();
                let n1 = source.add_node(builder.new_node_data());
                let n2 = source.add_node(builder.new_node_data());
                let n3 = source.add_node(builder.new_node_data());
                let e0 = source.add_edge(&n1, &n2, builder.new_edge_data()).unwrap();
                let e1 = source.add_edge(&n2, &n3, builder.new_edge_data()).unwrap();
                let mut target = builder.new_graph();

                // Extra boxing here works around being unable to declare a variable
                // of an `impl` type.  This allows the caller of the macro to use
                // closures without declaring the types of the arguments explicitly.
                let mut f: Box<
                    dyn Fn(&<TestGraph as Graph>::NodeData) -> <TestGraph as Graph>::NodeData,
                > = Box::new($f);
                let mut g: Box<
                    dyn Fn(&<TestGraph as Graph>::EdgeData) -> <TestGraph as Graph>::EdgeData,
                > = Box::new($g);

                let mut node_map = HashMap::new();
                let mut edge_map = HashMap::new();
                let target = GraphCopier::new(&source)
                    .transform_nodes(&mut f)
                    .transform_edges(&mut g)
                    .with_node_map(&mut node_map)
                    .with_edge_map(&mut edge_map)
                    .copy::<TestGraph>();

                assert_eq!(target.node_ids().count(), 3);
                assert_eq!(target.edge_ids().count(), 2);
                assert_eq!(f(source.node_data(&n1)), *target.node_data(&node_map[&n1]));
                assert_eq!(f(source.node_data(&n2)), *target.node_data(&node_map[&n2]));
                assert_eq!(f(source.node_data(&n3)), *target.node_data(&node_map[&n3]));
                assert_eq!(g(source.edge_data(&e0)), *target.edge_data(&edge_map[&e0]));
                assert_eq!(g(source.edge_data(&e1)), *target.edge_data(&edge_map[&e1]));
            }

            #[test]
            fn test_edge_multiplicity() {
                let mut builder = BuilderImpl::from($builder);
                let mut graph = builder.new_graph();
                let n1 = graph.add_node(builder.new_node_data());
                let n2 = graph.add_node(builder.new_node_data());
                let result = graph.add_edge(&n1, &n2, builder.new_edge_data());
                assert!(matches!(result, AddEdgeResult::Added(_)));
                let result = graph.add_edge(&n1, &n2, builder.new_edge_data());
                assert_eq!(graph.allows_parallel_edges(), matches!(result, AddEdgeResult::Added(_)));

                if graph.allows_parallel_edges() {
                    assert_eq!(graph.num_edges(), 2);
                } else {
                    assert_eq!(graph.num_edges(), 1);
                }
            }
        }
    };
}
