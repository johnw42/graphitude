use std::collections::HashSet;
use std::fmt::Debug;

#[cfg(any(test, feature = "test"))]
use proptest::prelude::*;
use tracing::info_span;

use crate::prelude::*;
use crate::tracing_support::{TimingScope, init_tracing, set_timing_scope};

#[cfg(any(test, feature = "test"))]
pub fn arb_graph<G>() -> impl Strategy<Value = G>
where
    G: GraphMut<NodeData = String, EdgeData = String> + Default + Clone + Debug + 'static,
{
    (0..64_usize).prop_flat_map(move |num_nodes| {
        (0..=2 * num_nodes).prop_flat_map(move |num_edges| {
            (
                0..=num_edges,
                0..=num_nodes,
                proptest::collection::vec((0..num_nodes, 0..num_nodes), num_edges),
            )
                .prop_map(
                    move |(num_extra_parallel_edges, num_extra_self_loops, edge_pairs)| {
                        let mut graph = G::default();

                        let node_ids = (0..num_nodes)
                            .map(|i| graph.add_node(format!("n{}", i)))
                            .collect::<Vec<_>>();

                        for (i, (source, target)) in edge_pairs.into_iter().enumerate() {
                            graph.add_edge(&node_ids[source], &node_ids[target], format!("e{}", i));
                            if i < num_extra_parallel_edges {
                                graph.add_edge(
                                    &node_ids[source],
                                    &node_ids[target],
                                    format!("e{}-extra", i),
                                );
                            }
                            if i < num_extra_self_loops {
                                graph.add_edge(
                                    &node_ids[source],
                                    &node_ids[source],
                                    format!("e{}-self", i),
                                );
                            }
                        }

                        graph
                    },
                )
        })
    })
}

/// Checks the internal consistency of a graph.
#[cfg(any(test, feature = "test"))]
pub fn check_graph_consistency<G: Graph>(graph: &G) {
    let _scope = set_timing_scope(TimingScope::Consistency);
    init_tracing();
    if graph.is_very_slow() {
        eprintln!("Skipping consistency check for very slow graph implementation.");
        return;
    }

    // Verify all nodes are valid
    for node_id in graph.nodes() {
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
    for edge_id in graph.edges() {
        let (left, right) = graph.edge_ends(&edge_id).into();
        assert_eq!(
            graph.edges_from_into(&left, &right).count(),
            graph
                .edges_from_into(&left, &right)
                .collect::<HashSet<_>>()
                .len()
        );

        if !graph.is_directed() {
            assert!(graph.has_edge_from_into(&right, &left))
        }
        if !graph.allows_parallel_edges() {
            assert_eq!(graph.num_edges_from_into(&left, &right), 1);
        }

        {
            let _span = info_span!("has_edge").entered();
            let has_edge = graph.has_edge_from_into(&left, &right);
            assert!(has_edge);
        }

        {
            let _span = info_span!("edges_between.any").entered();
            let between_has = graph.edges_from_into(&left, &right).any(|e| e == edge_id);
            assert!(between_has);
        }

        {
            let _span = info_span!("edges_from.any").entered();
            let from_has = graph.edges_from(&left).any(|e| e == edge_id);
            assert!(from_has);
        }

        {
            let _span = info_span!("edges_into.any").entered();
            let into_has = graph.edges_into(&right).any(|e| e == edge_id);
            assert!(into_has);
        }

        let num_from = {
            let _span = info_span!("num_edges_from").entered();
            graph.num_edges_from(&left)
        };

        let edges_from_count = {
            let _span = info_span!("edges_from.count").entered();
            graph.edges_from(&left).count()
        };

        let num_into = {
            let _span = info_span!("num_edges_into").entered();
            graph.num_edges_into(&right)
        };

        let edges_into_count = {
            let _span = info_span!("edges_into.count").entered();
            graph.edges_into(&right).count()
        };

        assert_eq!(num_from, edges_from_count);
        assert_eq!(num_into, edges_into_count);
    }

    // Verify node and edge IDs are unique.
    let node_ids: HashSet<_> = graph.nodes().collect();
    assert_eq!(node_ids.len(), graph.nodes().count());
    let edge_ids: HashSet<_> = graph.edges().collect();
    assert_eq!(edge_ids.len(), graph.edges().count());

    // Verify counts are correct
    assert_eq!(graph.nodes().count(), graph.num_nodes(),);
    assert_eq!(graph.edges().count(), graph.num_edges(),);

    // Check is_empty consistency
    assert_eq!(graph.is_empty(), graph.num_nodes() == 0);

    // If there are edges, there must be nodes
    assert!(graph.num_nodes() > 0 || graph.num_edges() == 0);
}
