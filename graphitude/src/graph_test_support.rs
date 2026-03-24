use derivative::Derivative;
use quickcheck::Arbitrary;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use tracing::info_span;

use crate::prelude::*;
use crate::tracing_support::{TimingScope, init_tracing, set_timing_scope};
use crate::util::sort_pair_if;

#[derive(Derivative)]
#[derivative(Debug(bound = "G::NodeData: Debug, G::EdgeData: Debug"))]
pub struct ArbGraph<G: GraphImplMut> {
    /// The graph to test.
    pub graph: Graph<G>,
    /// The node data used to construct the graph, for verification purposes.
    pub node_data: Vec<G::NodeData>,
    /// The edge data used to construct the graph, for verification purposes.
    /// Contains a pair of indices into the `node_data` vector for the source
    /// and target of each edge, along with the edge data.
    pub edge_data: Vec<((usize, usize), G::EdgeData)>,
    /// The node IDs corresponding to the `node_data` vector, for verification purposes.
    pub node_ids: Vec<NodeId<G>>,
    /// The edge IDs corresponding to the `edge_data` vector, for verification purposes.
    pub edge_ids: Vec<EdgeId<G>>,
}

impl<G> ArbGraph<G>
where
    G: GraphImplMut + 'static,
    G::NodeData: Arbitrary + Clone + Hash + Eq + 'static,
    G::EdgeData: Arbitrary + Clone + Hash + Eq + 'static,
{
    pub fn new(
        directedness: G::Directedness,
        edge_multiplicity: G::EdgeMultiplicity,
        node_data: Vec<G::NodeData>,
        edge_data: Vec<((usize, usize), G::EdgeData)>,
    ) -> Self {
        let mut graph = Graph::new(directedness, edge_multiplicity);
        let mut node_ids = Vec::new();
        for data in node_data.iter() {
            node_ids.push(graph.add_node(data.clone()));
        }
        let mut edge_ids = Vec::new();
        for ((from, into), data) in edge_data.iter() {
            let from_id = &node_ids[*from];
            let into_id = &node_ids[*into];
            edge_ids.push(graph.add_edge(from_id, into_id, data.clone()).edge_id());
        }
        Self {
            graph,
            node_data,
            edge_data,
            node_ids,
            edge_ids,
        }
    }
}

impl<G> Clone for ArbGraph<G>
where
    G: GraphImplMut + 'static,
    G::NodeData: Arbitrary + Clone + Hash + Eq + 'static,
    G::EdgeData: Arbitrary + Clone + Hash + Eq + 'static,
{
    fn clone(&self) -> Self {
        Self::new(
            self.graph.directedness(),
            self.graph.edge_multiplicity(),
            self.node_data.clone(),
            self.edge_data.clone(),
        )
    }
}

impl<G> Arbitrary for ArbGraph<G>
where
    G: GraphImplMut + 'static,
    G::NodeData: Arbitrary + Clone + Hash + Eq + 'static,
    G::EdgeData: Arbitrary + Clone + Hash + Eq + 'static,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let directedness = G::Directedness::arbitrary(g);
        let edge_multiplicity = G::EdgeMultiplicity::arbitrary(g);

        let num_nodes = g.size().min(100);
        let num_edges = usize::arbitrary(g) % (num_nodes * 2 + 1);

        let node_data: Vec<_> = (0..num_nodes).map(|_| G::NodeData::arbitrary(g)).collect();

        let mut edge_data = Vec::new();
        let mut node_index_pairs = HashSet::new();
        for _ in 0..num_edges {
            if node_data.len() < 2 {
                break;
            }
            let (source, target) = loop {
                let (source, target) = sort_pair_if(
                    !directedness.is_directed(),
                    (
                        usize::arbitrary(g) % node_data.len(),
                        usize::arbitrary(g) % node_data.len(),
                    ),
                );
                if edge_multiplicity.allows_parallel_edges()
                    || node_index_pairs.insert((source, target))
                {
                    break (source, target);
                }
            };
            edge_data.push(((source, target), G::EdgeData::arbitrary(g)));
            if edge_multiplicity.allows_parallel_edges() {
                if usize::arbitrary(g) % 3 == 0 {
                    edge_data.push(((source, target), G::EdgeData::arbitrary(g)));
                }
                if usize::arbitrary(g) % 3 == 0 {
                    edge_data.push(((source, source), G::EdgeData::arbitrary(g)));
                }
            }
        }

        ArbGraph::new(directedness, edge_multiplicity, node_data, edge_data)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let node_data_clone1 = self.node_data.clone();
        let node_data_clone2 = self.node_data.clone();
        let edge_data_clone1 = self.edge_data.clone();
        let edge_data_clone2 = self.edge_data.clone();
        let directedness = self.graph.directedness();
        let edge_multiplicity = self.graph.edge_multiplicity();

        Box::new(
            (0..node_data_clone1.len())
                .map(move |i| {
                    let mut new_node_data = node_data_clone1.clone();
                    let new_edge_data = edge_data_clone1
                        .clone()
                        .into_iter()
                        .filter_map(|((mut from, mut into), data)| {
                            if from == i || into == i {
                                None
                            } else {
                                if from > i {
                                    from -= 1;
                                }
                                if into > i {
                                    into -= 1;
                                }
                                Some(((from, into), data))
                            }
                        })
                        .collect();
                    new_node_data.remove(i);
                    ArbGraph::new(
                        directedness,
                        edge_multiplicity,
                        new_node_data,
                        new_edge_data,
                    )
                })
                .chain((0..edge_data_clone2.len()).map(move |i| {
                    let mut new_edge_data = edge_data_clone2.clone();
                    new_edge_data.remove(i);
                    ArbGraph::new(
                        directedness,
                        edge_multiplicity,
                        node_data_clone2.clone(),
                        new_edge_data,
                    )
                })),
        )
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
pub fn check_graph_consistency<G: GraphImpl>(graph: &Graph<G>) {
    let _scope = set_timing_scope(TimingScope::Consistency);
    init_tracing();

    // Verify all nodes are valid
    for node_id in graph.node_ids() {
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
