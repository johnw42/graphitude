mod linked {
    pub use graphitude::{LinkedGraph, prelude::*};
    use graphitude::{
        directedness::Directedness,
        edge_multiplicity::EdgeMultiplicity,
        graph_test_suite,
        graph_tests::{GraphTests, TestDataBuilder},
    };

    pub struct LinkedGraphBuilder<D, M> {
        directedness: D,
        edge_multiplicity: M,
    }

    impl<D, M> LinkedGraphBuilder<D, M> {
        pub fn new(directedness: D, edge_multiplicity: M) -> Self {
            Self {
                directedness,
                edge_multiplicity,
            }
        }
    }

    impl<D, M> TestDataBuilder for LinkedGraphBuilder<D, M>
    where
        D: DirectednessTrait,
        M: EdgeMultiplicityTrait,
    {
        type Graph = LinkedGraph<i32, String, D, M>;

        fn new_graph(&self) -> <Self as TestDataBuilder>::Graph {
            LinkedGraph::new(self.directedness, self.edge_multiplicity)
        }

        fn new_edge_data(&self, i: usize) -> String {
            format!("e{}", i)
        }

        fn new_node_data(&self, i: usize) -> i32 {
            i as i32
        }
    }

    graph_test_suite!(
        directed_multiple = GraphTests::<LinkedGraphBuilder<Directed, MultipleEdges>>::new(
            LinkedGraphBuilder::new(Directed, MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data)
        )
    );

    graph_test_suite!(
        directed_single = GraphTests::<LinkedGraphBuilder<Directed, SingleEdge>>::new(
            LinkedGraphBuilder::new(Directed, SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data)
        )
    );

    graph_test_suite!(
        undirected_multiple = GraphTests::<LinkedGraphBuilder<Undirected, MultipleEdges>>::new(
            LinkedGraphBuilder::new(Undirected, MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data)
        )
    );

    graph_test_suite!(
        undirected_single = GraphTests::<LinkedGraphBuilder<Undirected, SingleEdge>>::new(
            LinkedGraphBuilder::new(Undirected, SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data)
        )
    );

    graph_test_suite!(
        dyn_directed_multiple =
            GraphTests::<LinkedGraphBuilder<Directedness, EdgeMultiplicity>>::new(
                LinkedGraphBuilder::new(Directedness::Directed, EdgeMultiplicity::MultipleEdges),
                |data| data * 2,
                |data| format!("{}-copied", data)
            )
    );

    graph_test_suite!(
        dyn_directed_single = GraphTests::<LinkedGraphBuilder<Directedness, EdgeMultiplicity>>::new(
            LinkedGraphBuilder::new(Directedness::Directed, EdgeMultiplicity::SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data)
        )
    );

    graph_test_suite!(
        dyn_undirected_multiple =
            GraphTests::<LinkedGraphBuilder<Directedness, EdgeMultiplicity>>::new(
                LinkedGraphBuilder::new(Directedness::Undirected, EdgeMultiplicity::MultipleEdges),
                |data| data * 2,
                |data| format!("{}-copied", data)
            )
    );

    graph_test_suite!(
        dyn_undirected_single =
            GraphTests::<LinkedGraphBuilder<Directedness, EdgeMultiplicity>>::new(
                LinkedGraphBuilder::new(Directedness::Undirected, EdgeMultiplicity::SingleEdge),
                |data| data * 2,
                |data| format!("{}-copied", data)
            )
    );
}

mod adjacency {
    use graphitude::{
        BitvecStorage, HashStorage, Storage,
        adjacency_graph::AdjacencyGraph,
        graph_tests::{GraphTests, TestDataBuilder},
        prelude::*,
    };
    use graphitude::{adjacency_graph::edge_container::EdgeContainerSelector, graph_test_suite};
    use std::marker::PhantomData;

    pub struct AdjacencyGraphBuilder<D, M, S>(PhantomData<(D, M, S)>);

    impl<D, M, S> AdjacencyGraphBuilder<D, M, S> {
        pub fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<D, M, S> TestDataBuilder for AdjacencyGraphBuilder<D, M, S>
    where
        D: DirectednessTrait + Default,
        M: EdgeContainerSelector,
        S: Storage,
    {
        type Graph = AdjacencyGraph<i32, String, D, M, S>;

        fn new_graph(&self) -> Self::Graph {
            AdjacencyGraph::default()
        }

        fn new_edge_data(&self, i: usize) -> String {
            format!("e{}", i)
        }

        fn new_node_data(&self, i: usize) -> i32 {
            i as i32
        }
    }

    // macro_rules! graph_test_suite_with_compaction {
    //     ($mod_name:ident, $builder_type:ty) => {
    //         #[cfg(not(feature = "unchecked"))]
    //         #[test]
    //         #[should_panic]
    //         fn test_check_node_id_panics_after_compaction() {
    //             type Graph = <$builder as TestDataBuilder>::Graph;
    //             let mut graph = Graph::default();
    //             let n1 = graph.add_node(1);
    //             graph.compact();
    //             graph.assert_valid_node_id(&n1);
    //         }

    //         #[cfg(not(feature = "unchecked"))]
    //         #[test]
    //         #[should_panic]
    //         fn test_check_edge_id_panics_after_compaction() {
    //             type Graph = <$builder as TestDataBuilder>::Graph;
    //             let mut graph = Graph::default();
    //             let n1 = graph.add_node(1);
    //             let n2 = graph.add_node(2);
    //             if let AddEdgeResult::Added(e1) = graph.add_edge(&n1, &n2, "edge".to_string()) {
    //                 graph.compact();
    //                 graph.assert_valid_edge_id(&e1);
    //             }
    //         }
    //     };
    // }

    graph_test_suite!(
        directed_single_bitvec =
            GraphTests::<AdjacencyGraphBuilder<Directed, SingleEdge, BitvecStorage>>::new(
                AdjacencyGraphBuilder::new(),
                |data| data * 2,
                |data: &String| format!("{}-copied", data)
            ) //test_compaction!(AdjacencyGraphBuilder<Directed, SingleEdge, BitvecStorage>);
    );

    graph_test_suite!(
        undirected_single_bitvec =
            GraphTests::<AdjacencyGraphBuilder<Undirected, SingleEdge, BitvecStorage>>::new(
                AdjacencyGraphBuilder::new(),
                |data| data * 2,
                |data: &String| format!("{}-copied", data)
            ) //test_compaction!(AdjacencyGraphBuilder<Undirected, SingleEdge, BitvecStorage>);
    );

    graph_test_suite!(
        directed_single_hash =
            GraphTests::<AdjacencyGraphBuilder<Directed, SingleEdge, HashStorage>>::new(
                AdjacencyGraphBuilder::new(),
                |data| data * 2,
                |data: &String| format!("{}-copied", data)
            )
    );

    graph_test_suite!(
        undirected_single_hash =
            GraphTests::<AdjacencyGraphBuilder<Undirected, SingleEdge, HashStorage>>::new(
                AdjacencyGraphBuilder::new(),
                |data| data * 2,
                |data: &String| format!("{}-copied", data)
            )
    );

    graph_test_suite!(
        directed_multiple_bitvec =
            GraphTests::<AdjacencyGraphBuilder<Directed, MultipleEdges, BitvecStorage>>::new(
                AdjacencyGraphBuilder::new(),
                |data| data * 2,
                |data: &String| format!("{}-copied", data)
            ) // test_compaction!(AdjacencyGraphBuilder<Directed, MultipleEdges, BitvecStorage>);
    );

    graph_test_suite!(
        undirected_multiple_bitvec =
            GraphTests::<AdjacencyGraphBuilder<Undirected, MultipleEdges, BitvecStorage>>::new(
                AdjacencyGraphBuilder::new(),
                |data| data * 2,
                |data: &String| format!("{}-copied", data)
            ) // test_compaction!(AdjacencyGraphBuilder<Undirected, MultipleEdges, BitvecStorage>);
    );

    graph_test_suite!(
        directed_multiple_hash =
            GraphTests::<AdjacencyGraphBuilder<Directed, MultipleEdges, HashStorage>>::new(
                AdjacencyGraphBuilder::new(),
                |data| data * 2,
                |data: &String| format!("{}-copied", data)
            )
    );

    graph_test_suite!(
        undirected_multiple_hash =
            GraphTests::<AdjacencyGraphBuilder<Undirected, MultipleEdges, HashStorage>>::new(
                AdjacencyGraphBuilder::new(),
                |data| data * 2,
                |data: &String| format!("{}-copied", data)
            )
    );
}
