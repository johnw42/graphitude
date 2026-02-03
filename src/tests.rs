use std::marker::PhantomData;

use crate::Graph;

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
    G::NodeData: Clone + Eq,
    G::EdgeData: Clone + Eq,
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
        fn generate_large_graph() -> $type {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();

            // Create an irregular graph with ~500 nodes and ~2000 edges
            // Structure includes: clusters, hubs, sparse regions, and bridges

            let mut all_nodes = Vec::new();

            // Cluster 1: Dense cluster (50 nodes, highly connected)
            let cluster1_start = all_nodes.len();
            for _ in 0..50 {
                let node = graph.add_node(builder.new_node_data());
                all_nodes.push(node);
            }
            // Connect nodes within cluster 1 with ~60% density
            for i in cluster1_start..all_nodes.len() {
                for j in (i + 1)..all_nodes.len() {
                    if (i * 7 + j * 11) % 10 < 6 {
                        graph.add_edge(&all_nodes[i], &all_nodes[j], builder.new_edge_data());
                    }
                }
            }

            // Cluster 2: Medium cluster (80 nodes, moderately connected)
            let cluster2_start = all_nodes.len();
            for _ in 0..80 {
                let node = graph.add_node(builder.new_node_data());
                all_nodes.push(node);
            }
            // Connect nodes within cluster 2 with ~30% density
            for i in cluster2_start..all_nodes.len() {
                for j in (i + 1)..all_nodes.len() {
                    if (i * 13 + j * 17) % 10 < 3 {
                        graph.add_edge(&all_nodes[i], &all_nodes[j], builder.new_edge_data());
                    }
                }
            }

            // Cluster 3: Large sparse cluster (150 nodes, sparsely connected)
            let cluster3_start = all_nodes.len();
            for _ in 0..150 {
                let node = graph.add_node(builder.new_node_data());
                all_nodes.push(node);
            }
            // Connect nodes within cluster 3 with ~8% density
            for i in cluster3_start..all_nodes.len() {
                for j in (i + 1)..all_nodes.len() {
                    if (i * 19 + j * 23) % 100 < 8 {
                        graph.add_edge(&all_nodes[i], &all_nodes[j], builder.new_edge_data());
                    }
                }
            }

            // Add hub nodes (20 nodes with many connections)
            let hubs_start = all_nodes.len();
            for _ in 0..20 {
                let hub = graph.add_node(builder.new_node_data());
                all_nodes.push(hub.clone());

                // Connect each hub to random existing nodes
                for i in 0..all_nodes.len() - 1 {
                    if (hubs_start * 29 + i * 31) % 7 < 4 {
                        graph.add_edge(&hub, &all_nodes[i], builder.new_edge_data());
                    }
                }
            }

            // Add scattered nodes (100 nodes with few connections)
            let scattered_start = all_nodes.len();
            for _ in 0..100 {
                let node = graph.add_node(builder.new_node_data());
                all_nodes.push(node.clone());

                // Connect to 1-3 random other nodes
                let num_connections = ((scattered_start + all_nodes.len()) % 3) + 1;
                for c in 0..num_connections {
                    let target_idx = (scattered_start * 37 + all_nodes.len() * 41 + c * 43)
                        % (all_nodes.len() - 1);
                    graph.add_edge(&node, &all_nodes[target_idx], builder.new_edge_data());
                }
            }

            // Add bridge nodes connecting clusters (10 nodes)
            for i in 0..10 {
                let bridge = graph.add_node(builder.new_node_data());

                // Connect to nodes from different clusters
                let idx1 = (i * 47) % (cluster2_start - cluster1_start) + cluster1_start;
                let idx2 = (i * 53) % (cluster3_start - cluster2_start) + cluster2_start;
                let idx3 = (i * 59) % (hubs_start - cluster3_start) + cluster3_start;

                graph.add_edge(&bridge, &all_nodes[idx1], builder.new_edge_data());
                graph.add_edge(&bridge, &all_nodes[idx2], builder.new_edge_data());
                graph.add_edge(&bridge, &all_nodes[idx3], builder.new_edge_data());

                all_nodes.push(bridge);
            }

            // Add some long-range connections between random nodes
            for i in 0..200 {
                let idx1 = (i * 61) % all_nodes.len();
                let idx2 = (i * 67 + 100) % all_nodes.len();
                if idx1 != idx2 {
                    graph.add_edge(&all_nodes[idx1], &all_nodes[idx2], builder.new_edge_data());
                }
            }

            // Add some self loops
            for i in 0..50 {
                let idx = (i * 71) % all_nodes.len();
                graph.add_edge(&all_nodes[idx], &all_nodes[idx], builder.new_edge_data());
            }

            graph
        }

        #[test]
        fn test_large_graph_structure() {
            let graph = generate_large_graph();

            // Verify basic structure
            assert_eq!(
                graph.num_nodes(),
                410,
                "Expected 410 nodes: 50 + 80 + 150 + 20 + 100 + 10"
            );

            // The graph should have a significant number of edges
            // Theoretical maximum (if no duplicates/replacements):
            // - Cluster 1: ~735 edges (50 nodes, 60% of 1225 possible)
            // - Cluster 2: ~960 edges (80 nodes, 30% of 3160 possible)
            // - Cluster 3: ~894 edges (150 nodes, 8% of 11175 possible)
            // - Hubs: ~20 * 410 * 4/7 ≈ 4686 edges (many duplicates/overlaps expected)
            // - Scattered: ~100 * 2 = 200 edges (1-3 per node)
            // - Bridges: 30 edges (3 edges per bridge)
            // - Long-range: ~200 edges (some may be duplicates)
            // - Self-loops: 50 edges
            // Total theoretical max: ~7755 edges, but duplicates will reduce this

            let num_edges = graph.num_edges();

            // For implementations that replace duplicate edges, expect fewer edges
            // For implementations that allow parallel edges, expect more edges
            // Minimum: at least 2000 edges (conservative lower bound)
            // Maximum: at most 10000 edges (very liberal upper bound)
            assert!(
                num_edges >= 2000,
                "Expected at least 2000 edges, got {}",
                num_edges
            );
            assert!(
                num_edges <= 10000,
                "Expected at most 10000 edges, got {}",
                num_edges
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
            let mut graph = generate_large_graph();

            // We use a hash set instead of a vec so the nodes are removed in
            // random order.
            let mut node_ids = graph.node_ids().collect::<std::collections::HashSet<_>>();

            // We deliberately fix the number of iterations because we know it
            // in advance; each iteraction removes one node.
            for i in 0..node_ids.len() {
                // Test removing a random node.
                assert!(!node_ids.is_empty());
                let num_nodes = node_ids.len();
                let num_edges = graph.num_edges();
                assert_eq!(num_nodes, node_ids.len());
                let node_id = node_ids.iter().next().cloned().unwrap();
                node_ids.remove(&node_id);
                graph.remove_node(&node_id);
                assert_eq!(graph.num_nodes(), num_nodes - 1);
                assert!(graph.num_edges() <= num_edges);

                // Test compaction every few iterations
                if i % 50 == 0 {
                    let num_nodes = node_ids.len();
                    let num_edges = graph.num_edges();

                    graph.compact_with(
                        Some(|r| {
                            if let $crate::mapping_result::MappingResult::Remapped(old_id, new_id) =
                                r
                            {
                                let removed = node_ids.remove(&old_id);
                                assert!(removed);
                                let inserted = node_ids.insert(new_id);
                                assert!(inserted);
                            }
                        }),
                        Some(|_| {}),
                    );
                    assert_eq!(graph.num_nodes(), num_nodes);
                    assert_eq!(graph.num_edges(), num_edges);
                    for node_id in graph.node_ids() {
                        assert_eq!(graph.check_valid_node_id(&node_id), Ok(()));
                    }
                    for edge_id in graph.edge_ids() {
                        assert_eq!(graph.check_valid_edge_id(&edge_id), Ok(()));
                    }
                }
            }

            assert_eq!(graph.num_nodes(), 0);
            assert_eq!(graph.num_edges(), 0);
        }

        #[test]
        fn test_deconstruct_large_graph_by_edges() {
            let mut graph = generate_large_graph();

            // We use a hash set instead of a vec so the edges are removed in
            // random order.
            let mut edge_ids = graph.edge_ids().collect::<std::collections::HashSet<_>>();

            for i in 0..edge_ids.len() {
                // Test compaction every few iterations
                if i % 250 == 0 {
                    let num_nodes = graph.num_nodes();
                    let num_edges = edge_ids.len();

                    graph.compact_with(
                        Some(|_| {}),
                        Some(|r| {
                            if let $crate::mapping_result::MappingResult::Remapped(old_id, new_id) =
                                r
                            {
                                let removed = edge_ids.remove(&old_id);
                                assert!(removed);
                                let inserted = edge_ids.insert(new_id);
                                assert!(inserted);
                            }
                        }),
                    );
                    assert_eq!(graph.num_nodes(), num_nodes);
                    assert_eq!(graph.num_edges(), num_edges);
                    for node_id in graph.node_ids() {
                        assert_eq!(graph.check_valid_node_id(&node_id), Ok(()));
                    }
                    for edge_id in graph.edge_ids() {
                        assert_eq!(graph.check_valid_edge_id(&edge_id), Ok(()));
                    }
                }

                // Test removing a random edge.
                assert!(!edge_ids.is_empty());
                let num_nodes = graph.num_nodes();
                let num_edges = edge_ids.len();
                assert_eq!(num_edges, edge_ids.len());
                let edge_id = edge_ids.iter().next().cloned().unwrap();
                edge_ids.remove(&edge_id);
                graph.remove_edge(&edge_id);
                assert_eq!(graph.num_nodes(), num_nodes);
                assert_eq!(graph.num_edges(), num_edges - 1);
            }

            assert_eq!(graph.num_edges(), 0);
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
            let e1 = graph.add_edge(&n1, &n2, ed1.clone());
            let e2 = graph.add_edge(&n2, &n3, ed2.clone());

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
            let e1 = graph.add_edge(&n1, &n2, ed1.clone());
            let e2 = graph.add_edge(&n1, &n3, ed2.clone());

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

            let e0 = graph.add_edge(&n0, &n1, builder.new_edge_data());
            let e1 = graph.add_edge(&n0, &n2, builder.new_edge_data());
            let e2 = graph.add_edge(&n1, &n2, builder.new_edge_data());
            let e3 = graph.add_edge(&n1, &n3, builder.new_edge_data());
            let e4 = graph.add_edge(&n2, &n3, builder.new_edge_data());

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
            let e1 = graph.add_edge(&n1, &n2, ed1.clone());

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
            let e1 = graph.add_edge(&n1, &n2, ed1);

            assert_eq!(graph.num_edges_between(&n1, &n2), 1);
            assert_eq!(
                graph.edges_between(&n1, &n2).collect::<Vec<_>>(),
                vec![e1.clone()]
            );
            assert_eq!(
                graph.edges_between(&n2, &n1).collect::<Vec<_>>(),
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
            let e1 = source.add_edge(&n1, &n2, builder.new_edge_data());
            let e2 = source.add_edge(&n2, &n3, builder.new_edge_data());

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

            let e0 = graph.add_edge(&n0, &n1, builder.new_edge_data());
            let e1 = graph.add_edge(&n0, &n2, builder.new_edge_data());
            let e2 = graph.add_edge(&n1, &n2, builder.new_edge_data());
            let e3 = graph.add_edge(&n1, &n3, builder.new_edge_data());
            let e4 = graph.add_edge(&n2, &n3, builder.new_edge_data());

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
            assert_eq!(graph.num_edges_between(&n0, &n1), 1);
            assert_eq!(graph.num_edges_between(&n0, &n2), 1);
            assert_eq!(graph.num_edges_between(&n1, &n2), 1);
            assert_eq!(graph.num_edges_between(&n1, &n3), 1);
            assert_eq!(graph.num_edges_between(&n2, &n3), 1);
            assert_eq!(graph.num_edges_between(&n0, &n3), 0);
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
            let mut graph: $type = GraphMut::new();
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let nd1 = builder.new_node_data();
            let nd2 = builder.new_node_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let ed3 = builder.new_edge_data();

            let n1 = graph.add_node(nd1.clone());
            let n2 = graph.add_node(nd2.clone());
            let n3 = graph.add_node(builder.new_node_data());
            let e1 = graph.add_edge(&n1, &n1, ed1.clone());
            let e2 = graph.add_edge(&n1, &n2, ed2.clone());
            let _e3 = graph.add_edge(&n2, &n3, ed3.clone());
            graph.remove_node(&n3);
            assert_eq!(graph.node_ids().count(), 2);
            assert_eq!(graph.edge_ids().count(), 2);

            let mut nid_map = std::collections::HashMap::new();
            let mut eid_map = std::collections::HashMap::new();
            graph.compact_with(
                Some(|result: $crate::MappingResult<<$type as Graph>::NodeId>| {
                    if let $crate::MappingResult::Remapped(old_nid, new_nid) = result {
                        nid_map.insert(old_nid.clone(), new_nid.clone());
                    }
                }),
                Some(|result: $crate::MappingResult<<$type as Graph>::EdgeId>| {
                    if let $crate::MappingResult::Remapped(old_eid, new_eid) = result {
                        eid_map.insert(old_eid.clone(), new_eid.clone());
                    }
                }),
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
            let e0 = source.add_edge(&n1, &n2, builder.new_edge_data());
            let e1 = source.add_edge(&n2, &n3, builder.new_edge_data());

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
