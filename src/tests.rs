use std::{
    cell::RefCell,
    collections::{BTreeMap, HashMap},
    marker::PhantomData,
    sync::Once,
    time::{Duration, Instant},
};

use tracing::info_span;
use tracing_subscriber::{
    Layer, Registry, layer::Context, layer::SubscriberExt, registry::LookupSpan,
    util::SubscriberInitExt,
};

use crate::prelude::*;

thread_local! {
    static TIMING_SCOPES: RefCell<HashMap<TimingScope, BTreeMap<&'static str, (Duration, usize)>>> =
        RefCell::new(HashMap::new());
    static TIMING_SCOPE: RefCell<TimingScope> = RefCell::new(TimingScope::Test);
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TimingScope {
    Test,
    Consistency,
}

pub struct TimingScopeGuard {
    previous: TimingScope,
}

impl Drop for TimingScopeGuard {
    fn drop(&mut self) {
        TIMING_SCOPE.with(|scope| {
            *scope.borrow_mut() = self.previous;
        });
    }
}

pub fn set_timing_scope(scope: TimingScope) -> TimingScopeGuard {
    let previous = TIMING_SCOPE.with(|current| {
        let mut current = current.borrow_mut();
        let prev = *current;
        *current = scope;
        prev
    });
    TimingScopeGuard { previous }
}

struct TimingLayer;

impl<S> Layer<S> for TimingLayer
where
    S: tracing::Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_new_span(
        &self,
        _attrs: &tracing::span::Attributes<'_>,
        id: &tracing::Id,
        ctx: Context<'_, S>,
    ) {
        if let Some(span) = ctx.span(id) {
            span.extensions_mut().insert(Instant::now());
        }
    }

    fn on_close(&self, id: tracing::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            let name = span.metadata().name();
            if let Some(start) = span.extensions().get::<Instant>() {
                let elapsed = start.elapsed();
                let scope = TIMING_SCOPE.with(|scope| *scope.borrow());
                TIMING_SCOPES.with(|totals| {
                    let mut totals = totals.borrow_mut();
                    let entries = totals.entry(scope).or_insert_with(BTreeMap::new);
                    let entry = entries.entry(name).or_insert((Duration::ZERO, 0));
                    entry.0 += elapsed;
                    entry.1 += 1;
                });
            }
        }
    }
}

fn init_tracing() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = Registry::default().with(TimingLayer).try_init();
    });
}

#[doc(hidden)]
pub fn dump_method_timings() {
    dump_scope_timings(TimingScope::Test);
    dump_scope_timings(TimingScope::Consistency);
}

#[doc(hidden)]
pub fn reset_method_timings() {
    init_tracing();
    TIMING_SCOPES.with(|totals| totals.borrow_mut().clear());
}

fn dump_scope_timings(scope: TimingScope) {
    TIMING_SCOPES.with(|totals| {
        let totals = totals.borrow();
        let label = format!("{scope:?} timings (desc):");
        let Some(entries) = totals.get(&scope) else {
            eprintln!("{}", label);
            return;
        };
        let mut entries: Vec<_> = entries.iter().collect();
        entries.sort_by(|a, b| b.1.0.cmp(&a.1.0));
        eprintln!("{}", label);
        for (name, (duration, count)) in entries {
            eprintln!("  {name}: {:?} ({}x)", duration, count);
        }
    });
}

/// State tracker for generating sequential test data.
///
/// Maintains counters for nodes and edges to ensure unique test data generation.
pub struct BuilderState {
    v: usize,
    e: usize,
}

impl BuilderState {
    pub fn next_node(&mut self) -> usize {
        let id = self.v;
        self.v += 1;
        id
    }

    pub fn next_edge(&mut self) -> usize {
        let id = self.e;
        self.e += 1;
        id
    }
}

/// Trait for building test data for graphs.  Graph implementations used in
/// tests should implement this trait to provide consistent node and edge data.
pub trait TestDataBuilder {
    type Graph: Graph;

    /// Creates new edge data for testing, given an index.  Tests will call this
    /// method with consecutive indices starting from zero.
    fn new_edge_data(i: usize) -> <Self::Graph as Graph>::EdgeData;

    /// Creates new node data for testing, given an index.  Tests will call
    /// this method with consecutive indices starting from zero.
    fn new_node_data(i: usize) -> <Self::Graph as Graph>::NodeData;
}

/// Internal implementation of test data builder.
///
/// This type should not be used directly; use the test macros instead.
pub struct InternalBuilderImpl<G>(BuilderState, PhantomData<G>);

impl<G> InternalBuilderImpl<G>
where
    G: Graph + TestDataBuilder<Graph = G>,
{
    pub fn new() -> Self {
        Self(BuilderState { v: 0, e: 0 }, PhantomData)
    }

    pub fn new_node_data(&mut self) -> G::NodeData {
        let id = self.0.next_node();
        G::new_node_data(id)
    }

    pub fn new_edge_data(&mut self) -> G::EdgeData {
        let id = self.0.next_edge();
        G::new_edge_data(id)
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
pub fn generate_large_graph_with<G, FN, FE>(mut new_node_data: FN, mut new_edge_data: FE) -> G
where
    G: GraphMut + GraphNew,
    FN: FnMut(usize) -> G::NodeData,
    FE: FnMut(usize) -> G::EdgeData,
{
    let mut graph = G::new();
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

    graph
}

/// Generates a large graph using the `TestDataBuilder` trait for data generation.
///
/// This is a convenience wrapper around [`generate_large_graph_with`] that uses
/// the TestDataBuilder trait to provide node and edge data.
#[doc(hidden)]
pub fn generate_large_graph<G>() -> G
where
    G: GraphNew + TestDataBuilder<Graph = G>,
{
    generate_large_graph_with(|i| G::new_node_data(i), |i| G::new_edge_data(i))
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
        {
            let _span = info_span!("check_valid_edge_id").entered();
            let valid = graph.check_valid_edge_id(&edge_id);
            assert_eq!(valid, Ok(()));
        }

        {
            let _span = info_span!("has_edge").entered();
            let has_edge = graph.has_edge_from_into(&edge_id.source(), &edge_id.target());
            assert!(has_edge);
        }

        {
            let _span = info_span!("edges_between.any").entered();
            let between_has = graph
                .edges_from_into(&edge_id.source(), &edge_id.target())
                .any(|e| e == edge_id);
            assert!(between_has);
        }

        {
            let _span = info_span!("edges_from.any").entered();
            let from_has = graph.edges_from(&edge_id.source()).any(|e| e == edge_id);
            assert!(from_has);
        }

        {
            let _span = info_span!("edges_into.any").entered();
            let into_has = graph.edges_into(&edge_id.target()).any(|e| e == edge_id);
            assert!(into_has);
        }

        let num_from = {
            let _span = info_span!("num_edges_from").entered();
            graph.num_edges_from(&edge_id.source())
        };

        let edges_from_count = {
            let _span = info_span!("edges_from.count").entered();
            graph.edges_from(&edge_id.source()).count()
        };

        let num_into = {
            let _span = info_span!("num_edges_into").entered();
            graph.num_edges_into(&edge_id.target())
        };

        let edges_into_count = {
            let _span = info_span!("edges_into.count").entered();
            graph.edges_into(&edge_id.target()).count()
        };

        assert_eq!(num_from, edges_from_count);
        assert_eq!(num_into, edges_into_count);
    }

    // Verify counts are correct
    assert_eq!(graph.node_ids().count(), graph.num_nodes(),);
    assert_eq!(graph.edge_ids().count(), graph.num_edges(),);

    // Check is_empty consistency
    assert_eq!(graph.is_empty(), graph.num_nodes() == 0);

    // If there are edges, there must be nodes
    assert!(graph.num_nodes() > 0 || graph.num_edges() == 0);
}

/// Macro to generate standard graph tests for a given graph type.
#[macro_export]
macro_rules! graph_tests {
    ($name:ident, $type:ty) => {
        mod $name {
            use super::*;

            graph_tests_impl!($type);
        }
    };
    ($type:ty) => {
        #[test]
        fn test_large_graph_structure() {
            let graph = $crate::tests::generate_large_graph::<$type>();

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
        fn test_deconstruct_large_graph_by_nodes() {
            $crate::tests::reset_method_timings();
            let _scope = $crate::tests::set_timing_scope($crate::tests::TimingScope::Test);
            let test_span = tracing::info_span!("test_deconstruct_large_graph_by_nodes");
            let _test_guard = test_span.entered();
            let mut graph = {
                let _span = tracing::info_span!("generate_large_graph").entered();
                $crate::tests::generate_large_graph::<$type>()
            };

            // We use a hash set instead of a vec so the nodes are removed in
            // random order.
            let mut node_ids = graph.node_ids().collect::<std::collections::HashSet<_>>();

            // We deliberately fix the number of iterations because we know it
            // in advance; each iteraction removes one node.
            let _remove_loop_span = tracing::info_span!("remove_nodes_loop").entered();
            for i in 0..node_ids.len() {
                // Test removing a random node.
                assert!(!node_ids.is_empty());
                let num_nodes = node_ids.len();
                let num_edges = graph.num_edges();
                assert_eq!(num_nodes, node_ids.len());
                let node_id = node_ids.iter().next().cloned().unwrap();
                node_ids.remove(&node_id);
                {
                    let _span = tracing::info_span!("remove_node").entered();
                    graph.remove_node(&node_id);
                }
                assert_eq!(graph.num_nodes(), num_nodes - 1);
                assert!(graph.num_edges() <= num_edges);

                // Test compaction periodically
                if i % 50 == 0 {
                    let num_nodes = node_ids.len();
                    let num_edges = graph.num_edges();

                    {
                        let _span = tracing::info_span!("compact").entered();
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
                        let _span = tracing::info_span!("check_valid_ids").entered();
                        for node_id in graph.node_ids() {
                            assert_eq!(graph.check_valid_node_id(&node_id), Ok(()));
                        }
                        for edge_id in graph.edge_ids() {
                            assert_eq!(graph.check_valid_edge_id(&edge_id), Ok(()));
                        }
                    }
                    {
                        let _span = tracing::info_span!("check_graph_consistency").entered();
                        $crate::tests::check_graph_consistency(&graph);
                    }
                }
            }
            drop(_remove_loop_span);

            assert_eq!(graph.num_nodes(), 0);
            assert_eq!(graph.num_edges(), 0);
            assert!(graph.is_empty());
            drop(_test_guard);
            $crate::tests::dump_method_timings();
        }

        #[test]
        fn test_deconstruct_large_graph_by_edges() {
            $crate::tests::reset_method_timings();
            let _scope = $crate::tests::set_timing_scope($crate::tests::TimingScope::Test);
            let test_span = tracing::info_span!("test_deconstruct_large_graph_by_edges");
            let _test_guard = test_span.entered();
            let mut graph = {
                let _span = tracing::info_span!("generate_large_graph").entered();
                $crate::tests::generate_large_graph::<$type>()
            };

            // We use a hash set instead of a vec so the edges are removed in
            // random order.
            let mut edge_ids = graph.edge_ids().collect::<std::collections::HashSet<_>>();

            let _remove_loop_span = tracing::info_span!("remove_edges_loop").entered();
            for i in 0..edge_ids.len() {
                // Test compaction periodically
                if i % 250 == 0 {
                    let num_nodes = graph.num_nodes();
                    let num_edges = edge_ids.len();

                    {
                        let _span = tracing::info_span!("compact").entered();
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
                        let _span = tracing::info_span!("check_valid_ids").entered();
                        for node_id in graph.node_ids() {
                            assert_eq!(graph.check_valid_node_id(&node_id), Ok(()));
                        }
                        for edge_id in graph.edge_ids() {
                            assert_eq!(graph.check_valid_edge_id(&edge_id), Ok(()));
                        }
                    }
                    {
                        let _span = tracing::info_span!("check_graph_consistency").entered();
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
                    let _span = tracing::info_span!("remove_edge").entered();
                    graph.remove_edge(&edge_id);
                }
                assert_eq!(graph.num_nodes(), num_nodes);
                assert_eq!(graph.num_edges(), num_edges - 1);
            }
            drop(_remove_loop_span);

            assert_eq!(graph.num_edges(), 0);
            drop(_test_guard);
            $crate::tests::dump_method_timings();
        }

        #[test]
        fn test_new_graph_is_empty() {
            let graph: $type = <$type>::new();
            assert_eq!(graph.num_nodes(), 0);
            assert_eq!(graph.num_edges(), 0);
        }

        #[test]
        fn test_node_data_retrieval() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let vd1 = builder.new_node_data();
            let n1 = graph.add_node(vd1.clone());
            assert_eq!(*graph.node_data(&n1), vd1);
        }

        #[test]
        fn test_edge_creation() {
            use crate::EdgeId;
            use std::collections::HashSet;

            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let vd1 = builder.new_node_data();
            let vd2 = builder.new_node_data();
            let vd3 = builder.new_node_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let n1 = graph.add_node(vd1);
            let n2 = graph.add_node(vd2);
            let n3 = graph.add_node(vd3);
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
                        .map(|edge_id| edge_id.target())
                        .collect::<Vec<_>>(),
                    vec![n2.clone()]
                );
                assert_eq!(
                    graph
                        .edges_from(&n2)
                        .map(|edge_id| edge_id.target())
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
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let vd1 = builder.new_node_data();
            let vd2 = builder.new_node_data();
            let vd3 = builder.new_node_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let n1 = graph.add_node(vd1);
            let n2 = graph.add_node(vd2);
            let n3 = graph.add_node(vd3);
            let e1 = graph.add_edge(&n1, &n2, ed1.clone()).unwrap();
            let e2 = graph.add_edge(&n1, &n3, ed2.clone()).unwrap();

            let edge_ids: Vec<_> = graph.edge_ids().collect();
            dbg!(&edge_ids);
            assert_eq!(edge_ids.len(), 2);
            assert!(edge_ids.contains(&e1));
            assert!(edge_ids.contains(&e2));
        }

        #[test]
        fn test_node_removal() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let vd1 = builder.new_node_data();
            let vd2 = builder.new_node_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let ed3 = builder.new_edge_data();

            let n1 = graph.add_node(vd1.clone());
            let n2 = graph.add_node(vd2.clone());

            // Normal edge.
            graph.add_edge(&n1, &n2, ed1.clone());
            // Duplicate edge.
            graph.add_edge(&n1, &n2, ed2.clone());
            // Self edge.
            graph.add_edge(&n1, &n1, ed3.clone());

            let removed_data = graph.remove_node(&n1);
            assert_eq!(removed_data, vd1);
            assert_eq!(graph.num_nodes(), 1);
            assert_eq!(graph.num_edges(), 0);
        }

        #[test]
        fn test_remove_node_cleans_edges() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let vd1 = builder.new_node_data();
            let vd2 = builder.new_node_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let ed3 = builder.new_edge_data();

            let n1 = graph.add_node(vd1.clone());
            let n2 = graph.add_node(vd2.clone());

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

            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
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
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let vd1 = builder.new_node_data();
            let vd2 = builder.new_node_data();
            let ed1 = builder.new_edge_data();

            let n1 = graph.add_node(vd1);
            let n2 = graph.add_node(vd2);
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
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let vd1 = builder.new_node_data();
            let vd2 = builder.new_node_data();
            let ed1 = builder.new_edge_data();

            let n1 = graph.add_node(vd1);
            let n2 = graph.add_node(vd2);
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
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut source = <$type>::new();
            let n1 = source.add_node(builder.new_node_data());
            let n2 = source.add_node(builder.new_node_data());
            let n3 = source.add_node(builder.new_node_data());
            let e1 = source.add_edge(&n1, &n2, builder.new_edge_data()).unwrap();
            let e2 = source.add_edge(&n2, &n3, builder.new_edge_data()).unwrap();

            let mut target = <$type>::new();
            let node_map = target.copy_from(&source);
            let edge_map = target.make_edge_map(&source, &node_map);

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
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let n1 = graph.add_node(builder.new_node_data());
            let n2 = graph.add_node(builder.new_node_data());
            graph.add_edge(&n1, &n2, builder.new_edge_data());

            assert_eq!(graph.node_ids().count(), 2);
            assert_eq!(graph.edge_ids().count(), 1);

            graph.clear();

            assert_eq!(graph.node_ids().count(), 0);
            assert_eq!(graph.edge_ids().count(), 0);
            assert!(graph.is_empty());
        }

        #[test]
        fn test_successors() {
            use crate::EdgeId;
            use std::collections::HashSet;

            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
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
                assert_eq!((e0.source(), e0.target()), (n0.clone(), n1.clone()));
                assert_eq!((e1.source(), e1.target()), (n0.clone(), n2.clone()));
                assert_eq!((e2.source(), e2.target()), (n1.clone(), n2.clone()));
                assert_eq!((e3.source(), e3.target()), (n1.clone(), n3.clone()));
                assert_eq!((e4.source(), e4.target()), (n2.clone(), n3.clone()));
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
                    let (src, tgt) = edge.ends().into();
                    assert!(
                        (src == a && tgt == b) || (src == b && tgt == a),
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
        #[cfg(feature = "pathfinding")]
        fn test_shortest_paths() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
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
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
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
            let mut graph: $type = $crate::GraphNew::new();
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
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
    };
}

/// Macro to generate a test for the `copy_from_with` method of a graph type.
/// The arguments are the graph type, and two closures for transforming node
/// and edge data respectively.
#[macro_export]
macro_rules! graph_test_copy_from_with {
    ($type:ty, $f:expr, $g:expr) => {
        #[test]
        fn test_copy_from_with() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut source = <$type>::new();
            let n1 = source.add_node(builder.new_node_data());
            let n2 = source.add_node(builder.new_node_data());
            let n3 = source.add_node(builder.new_node_data());
            let e0 = source.add_edge(&n1, &n2, builder.new_edge_data()).unwrap();
            let e1 = source.add_edge(&n2, &n3, builder.new_edge_data()).unwrap();
            let mut target = <$type>::new();

            // Extra boxing here works around being unable to declare a variable
            // of an `impl` type.  This allows the caller of the macro to use
            // closures without declaring the types of the arguments explicitly.
            let mut f: Box<dyn Fn(&<$type as Graph>::NodeData) -> <$type as Graph>::NodeData> =
                Box::new($f);
            let mut g: Box<dyn Fn(&<$type as Graph>::EdgeData) -> <$type as Graph>::EdgeData> =
                Box::new($g);

            let node_map = target.copy_from_with(&source, &mut f, &mut g);
            let edge_map = target.make_edge_map(&source, &node_map);

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
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let n1 = graph.add_node(builder.new_node_data());
            let n2 = graph.add_node(builder.new_node_data());
            graph.add_edge(&n1, &n2, builder.new_edge_data());
            graph.add_edge(&n1, &n2, builder.new_edge_data());

            if graph.allows_parallel_edges() {
                assert_eq!(graph.num_edges(), 2);
            } else {
                assert_eq!(graph.num_edges(), 1);
            }
        }
    };
}
