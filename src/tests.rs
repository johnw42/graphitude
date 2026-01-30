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
    ($type:ty) => {
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
            assert_eq!(*graph.node_data(n1), vd1);
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
            let e1 = graph.add_edge(n1.clone(), n2.clone(), ed1.clone());
            let e2 = graph.add_edge(n2.clone(), n3.clone(), ed2.clone());

            assert_eq!(
                graph
                    .edges_from(n1)
                    .into_iter()
                    .map(|edge_id| edge_id.target())
                    .collect::<Vec<_>>(),
                vec![n2.clone()]
            );

            assert_eq!(graph.edge_data(e1.clone()), (&ed1));
            assert_eq!(graph.edge_data(e2.clone()), (&ed2));

            assert_eq!(graph.num_edges(), 2);
            assert_eq!(
                graph.edge_ids().collect::<HashSet<_>>(),
                HashSet::from([e1.clone(), e2.clone()])
            );
            assert_eq!(*graph.edge_data(e1), ed1);
            assert_eq!(*graph.edge_data(e2), ed2);
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
            let e1 = graph.add_edge(n1.clone(), n2.clone(), ed1.clone());
            let e2 = graph.add_edge(n1.clone(), n3.clone(), ed2.clone());

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
            graph.add_edge(n1.clone(), n2.clone(), ed1.clone());
            // Duplicate edge.
            graph.add_edge(n1.clone(), n2.clone(), ed2.clone());
            // Self edge.
            graph.add_edge(n1.clone(), n1.clone(), ed3.clone());

            let removed_data = graph.remove_node(n1.clone());
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
            graph.add_edge(n1.clone(), n2.clone(), ed1.clone());
            // Duplicate edge.
            graph.add_edge(n1.clone(), n2.clone(), ed2.clone());
            // Self edge.
            graph.add_edge(n1.clone(), n1.clone(), ed3.clone());

            graph.remove_node(n1.clone());
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

            let e0 = graph.add_edge(n0.clone(), n1.clone(), builder.new_edge_data());
            let e1 = graph.add_edge(n0.clone(), n2.clone(), builder.new_edge_data());
            let e2 = graph.add_edge(n1.clone(), n2.clone(), builder.new_edge_data());
            let e3 = graph.add_edge(n1.clone(), n3.clone(), builder.new_edge_data());
            let e4 = graph.add_edge(n2.clone(), n3.clone(), builder.new_edge_data());

            assert_eq!(
                graph.edges_from(n0.clone()).collect::<HashSet<_>>(),
                HashSet::from([e0.clone(), e1.clone()])
            );
            if graph.is_directed() {
                assert_eq!(
                    graph.edges_from(n1.clone()).collect::<HashSet<_>>(),
                    HashSet::from([e2.clone(), e3.clone()])
                );
                assert_eq!(
                    graph.edges_from(n2.clone()).collect::<HashSet<_>>(),
                    HashSet::from([e4.clone()])
                );
                assert_eq!(graph.edges_from(n3.clone()).count(), 0,);
            } else {
                assert_eq!(
                    graph.edges_from(n1.clone()).collect::<HashSet<_>>(),
                    HashSet::from([e0.clone(), e2.clone(), e3.clone()])
                );
                assert_eq!(
                    graph.edges_from(n2.clone()).collect::<HashSet<_>>(),
                    HashSet::from([e1.clone(), e2.clone(), e4.clone()])
                );
                assert_eq!(
                    graph.edges_from(n3.clone()).collect::<HashSet<_>>(),
                    HashSet::from([e3.clone(), e4.clone()])
                );
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
            let e1 = graph.add_edge(n1.clone(), n2.clone(), ed1.clone());

            assert_eq!(graph.edges_into(n2.clone()).collect::<Vec<_>>(), vec![e1]);
            assert_eq!(
                graph.num_edges_into(n1.clone()),
                (!graph.is_directed()).into()
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
            let e1 = graph.add_edge(n1.clone(), n2.clone(), ed1);

            assert_eq!(graph.num_edges_between(n1.clone(), n2.clone()), 1);
            assert_eq!(
                graph
                    .edges_between(n1.clone(), n2.clone())
                    .collect::<Vec<_>>(),
                vec![e1.clone()]
            );
            assert_eq!(
                graph
                    .edges_between(n2.clone(), n1.clone())
                    .collect::<Vec<_>>(),
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
            let e1 = source.add_edge(n1.clone(), n2.clone(), builder.new_edge_data());
            let e2 = source.add_edge(n2.clone(), n3.clone(), builder.new_edge_data());

            let mut target = <$type>::new();
            let node_map = target.copy_from(&source);
            let edge_map = target.make_edge_map(&source, &node_map);

            assert_eq!(target.node_ids().count(), 3);
            assert_eq!(target.edge_ids().count(), 2);
            assert_eq!(
                source.node_data(n1.clone()),
                target.node_data(node_map[&n1].clone())
            );
            assert_eq!(
                source.node_data(n2.clone()),
                target.node_data(node_map[&n2].clone())
            );
            assert_eq!(
                source.node_data(n3.clone()),
                target.node_data(node_map[&n3].clone())
            );
            assert_eq!(
                source.edge_data(e1.clone()),
                target.edge_data(edge_map[&e1].clone())
            );
            assert_eq!(
                source.edge_data(e2.clone()),
                target.edge_data(edge_map[&e2].clone())
            );
        }

        #[test]
        fn test_clear() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = <$type>::new();
            let n1 = graph.add_node(builder.new_node_data());
            let n2 = graph.add_node(builder.new_node_data());
            graph.add_edge(n1, n2, builder.new_edge_data());

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

            let e0 = graph.add_edge(n0.clone(), n1.clone(), builder.new_edge_data());
            let e1 = graph.add_edge(n0.clone(), n2.clone(), builder.new_edge_data());
            let e2 = graph.add_edge(n1.clone(), n2.clone(), builder.new_edge_data());
            let e3 = graph.add_edge(n1.clone(), n3.clone(), builder.new_edge_data());
            let e4 = graph.add_edge(n2.clone(), n3.clone(), builder.new_edge_data());

            assert_eq!((e0.source(), e0.target()), (n0.clone(), n1.clone()));
            assert_eq!((e1.source(), e1.target()), (n0.clone(), n2.clone()));
            assert_eq!((e2.source(), e2.target()), (n1.clone(), n2.clone()));
            assert_eq!((e3.source(), e3.target()), (n1.clone(), n3.clone()));
            assert_eq!((e4.source(), e4.target()), (n2.clone(), n3.clone()));
            assert_eq!(
                graph.successors(n0.clone()).collect::<HashSet<_>>(),
                HashSet::from([n1.clone(), n2.clone()])
            );
            if graph.is_directed() {
                assert_eq!(
                    graph.successors(n1.clone()).collect::<HashSet<_>>(),
                    HashSet::from([n2.clone(), n3.clone()])
                );
                assert_eq!(
                    graph.successors(n2.clone()).collect::<HashSet<_>>(),
                    HashSet::from([n3.clone()])
                );
                assert_eq!(graph.successors(n3.clone()).count(), 0,);
            } else {
                assert_eq!(
                    graph.successors(n1.clone()).collect::<HashSet<_>>(),
                    HashSet::from([n0.clone(), n2.clone(), n3.clone()])
                );
                assert_eq!(
                    graph.successors(n2.clone()).collect::<HashSet<_>>(),
                    HashSet::from([n0.clone(), n1.clone(), n3.clone()])
                );
                assert_eq!(
                    graph.successors(n3.clone()).collect::<HashSet<_>>(),
                    HashSet::from([n1.clone(), n2.clone()])
                );
            }
            assert_eq!(graph.num_edges(), 5);
            assert_eq!(graph.num_edges_between(n0.clone(), n1.clone()), 1);
            assert_eq!(graph.num_edges_between(n0.clone(), n2.clone()), 1);
            assert_eq!(graph.num_edges_between(n1.clone(), n2.clone()), 1);
            assert_eq!(graph.num_edges_between(n1.clone(), n3.clone()), 1);
            assert_eq!(graph.num_edges_between(n2.clone(), n3.clone()), 1);
            assert_eq!(graph.num_edges_between(n0.clone(), n3.clone()), 0);
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

            graph.add_edge(n0.clone(), n1.clone(), builder.new_edge_data());
            graph.add_edge(n0.clone(), n2.clone(), builder.new_edge_data());
            graph.add_edge(n1.clone(), n2.clone(), builder.new_edge_data());
            graph.add_edge(n1.clone(), n3.clone(), builder.new_edge_data());
            graph.add_edge(n2.clone(), n3.clone(), builder.new_edge_data());

            let paths = graph.shortest_paths(n0.clone(), |_| 1);
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

            graph.add_edge(n0.clone(), n1.clone(), builder.new_edge_data());

            let paths = graph.shortest_paths(n0.clone(), |_| 1);
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

            let n1 = graph.add_node(nd1.clone());
            let n2 = graph.add_node(nd2.clone());
            let n3 = graph.add_node(builder.new_node_data());
            let e1 = graph.add_edge(n1.clone(), n1.clone(), ed1.clone());
            let e2 = graph.add_edge(n1.clone(), n2.clone(), ed2.clone());
            let _e3 = graph.add_edge(n2.clone(), n3.clone(), builder.new_edge_data());
            graph.remove_node(n3.clone());
            assert_eq!(graph.node_ids().count(), 2);
            assert_eq!(graph.edge_ids().count(), 2);

            let mut nid_map = std::collections::HashMap::new();
            let mut eid_map = std::collections::HashMap::new();
            graph.compact_with(
                Some(
                    |old_nid: <$type as Graph>::NodeId, new_nid: <$type as Graph>::NodeId| {
                        nid_map.insert(old_nid.clone(), new_nid.clone());
                    },
                ),
                Some(
                    |old_eid: <$type as Graph>::EdgeId, new_eid: <$type as Graph>::EdgeId| {
                        eid_map.insert(old_eid.clone(), new_eid.clone());
                    },
                ),
            );
            assert_eq!(graph.node_ids().count(), 2);
            assert_eq!(graph.edge_ids().count(), 2);

            // After compaction, we need to use the new node ID.
            // Find the node with the same data
            assert_eq!(
                graph.node_data(nid_map.get(&n1).unwrap_or(&n1).clone()),
                &nd1
            );
            assert_eq!(
                graph.node_data(nid_map.get(&n2).unwrap_or(&n2).clone()),
                &nd2
            );
            assert_eq!(
                graph.edge_data(eid_map.get(&e1).unwrap_or(&e1).clone()),
                &ed1
            );
            assert_eq!(
                graph.edge_data(eid_map.get(&e2).unwrap_or(&e2).clone()),
                &ed2
            );
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
            let e0 = source.add_edge(n1.clone(), n2.clone(), builder.new_edge_data());
            let e1 = source.add_edge(n2.clone(), n3.clone(), builder.new_edge_data());

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
            assert_eq!(
                f(source.node_data(n1.clone())),
                *target.node_data(node_map[&n1].clone())
            );
            assert_eq!(
                f(source.node_data(n2.clone())),
                *target.node_data(node_map[&n2].clone())
            );
            assert_eq!(
                f(source.node_data(n3.clone())),
                *target.node_data(node_map[&n3].clone())
            );
            assert_eq!(
                g(source.edge_data(e0.clone())),
                *target.edge_data(edge_map[&e0].clone())
            );
            assert_eq!(
                g(source.edge_data(e1.clone())),
                *target.edge_data(edge_map[&e1].clone())
            );
        }
    };
}
