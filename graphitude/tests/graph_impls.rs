mod linked {
    pub use graphitude::{LinkedGraph, prelude::*};
    use graphitude::{
        directedness::Directedness, edge_multiplicity::EdgeMultiplicity, graph_test_suite,
        graph_tests::GraphTests,
    };

    graph_test_suite!(
        directed_multiple:
            GraphTests<LinkedGraph<i32, i32, Directed, MultipleEdges>>
    );

    graph_test_suite!(
        directed_single:
            GraphTests<LinkedGraph<i32, i32, Directed, SingleEdge>>
    );

    graph_test_suite!(
        undirected_multiple:
            GraphTests<LinkedGraph<i32, i32, Undirected, MultipleEdges>>
    );

    graph_test_suite!(
        undirected_single:
            GraphTests<LinkedGraph<i32, i32, Undirected, SingleEdge>>
    );

    graph_test_suite!(
        dyn_directed_multiple:
            GraphTests<LinkedGraph<i32, i32, Directed, MultipleEdges>>
    );

    graph_test_suite!(
        dyn_directed_single:
            GraphTests<LinkedGraph<i32, i32, Directed, SingleEdge>>
    );

    graph_test_suite!(
        dyn_undirected_multiple:
            GraphTests<LinkedGraph<i32, i32, Undirected, MultipleEdges>>
    );

    graph_test_suite!(
        dyn_undirected_single =
            GraphTests::<LinkedGraph<i32, i32, Directedness, EdgeMultiplicity>>::new(|| {
                Graph::new(LinkedGraph::new(
                    Directedness::Undirected,
                    EdgeMultiplicity::SingleEdge,
                ))
            },)
    );
}

mod adjacency {
    use graphitude::graph_test_suite;
    use graphitude::{
        BitvecStorage, HashStorage, adjacency_graph::AdjacencyGraph, graph_tests::GraphTests,
        prelude::*,
    };

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
        directed_single_bitvec:
            GraphTests::<AdjacencyGraph<i32, i32, Directed, SingleEdge, BitvecStorage>>
    );

    graph_test_suite!(
        undirected_single_bitvec:
            GraphTests::<AdjacencyGraph<i32, i32, Undirected, SingleEdge, BitvecStorage>>
    );

    graph_test_suite!(
        directed_single_hash:
            GraphTests::<AdjacencyGraph<i32, i32, Directed, SingleEdge, HashStorage>>
    );

    graph_test_suite!(
        undirected_single_hash:
            GraphTests::<AdjacencyGraph<i32, i32, Undirected, SingleEdge, HashStorage>>
    );

    graph_test_suite!(
        directed_multiple_bitvec:
            GraphTests::<AdjacencyGraph<i32, i32, Directed, MultipleEdges, BitvecStorage>>
    );

    graph_test_suite!(
        undirected_multiple_bitvec:
            GraphTests::<AdjacencyGraph<i32, i32, Undirected, MultipleEdges, BitvecStorage>>
    );

    graph_test_suite!(
        directed_multiple_hash:
            GraphTests::<AdjacencyGraph<i32, i32, Directed, MultipleEdges, HashStorage>>
    );

    graph_test_suite!(
        undirected_multiple_hash:
            GraphTests::<AdjacencyGraph<i32, i32, Undirected, MultipleEdges, HashStorage>>
    );
}
