use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

use quickcheck::Arbitrary;
use tracing::info_span;

use crate::prelude::*;
use crate::tracing_support::{TimingScope, init_tracing, set_timing_scope};

#[derive(Debug, Clone)]
pub struct ArbGraph<G> {
    pub graph: G,
}

impl<G> Arbitrary for ArbGraph<G>
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

        ArbGraph { graph }
    }
}

pub fn has_duplicates<T: Eq + Hash>(items: impl IntoIterator<Item = T>) -> bool {
    let mut seen = HashSet::new();
    for item in items {
        if !seen.insert(item) {
            return true;
        }
    }
    false
}

/// Checks the internal consistency of a graph.
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
