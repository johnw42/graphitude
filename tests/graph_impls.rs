mod linked {
    pub use graphitude::{LinkedGraph, graph_tests, prelude::*, tests::TestDataBuilder};
    use std::marker::PhantomData;

    pub struct LinkedGraphBuilder<D, M>(PhantomData<(D, M)>);

    impl<D, M> TestDataBuilder for LinkedGraphBuilder<D, M>
    where
        D: DirectednessTrait,
        M: EdgeMultiplicityTrait,
    {
        type Graph = LinkedGraph<i32, String, D, M>;

        fn new_edge_data(i: usize) -> String {
            format!("e{}", i)
        }

        fn new_node_data(i: usize) -> i32 {
            i as i32
        }
    }

    graph_tests!(directed_multiple, LinkedGraphBuilder<Directed, MultipleEdges>,
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(directed_single, LinkedGraphBuilder<Directed, SingleEdge>,
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(undirected_multiple, LinkedGraphBuilder<Undirected, MultipleEdges>,
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(undirected_single, LinkedGraphBuilder<Undirected, SingleEdge>,
            |data| data * 2,
            |data| format!("{}-copied", data));
}

mod adjacency {
    pub use graphitude::{
        BitvecStorage, Graph, HashStorage, Storage, adjacency_graph::AdjacencyGraph,
        adjacency_matrix::AdjacencyMatrixSelector, graph_tests, prelude::*, tests::TestDataBuilder,
    };
    use std::marker::PhantomData;

    pub struct AdjacencyGraphBuilder<D, S>(PhantomData<(D, S)>);

    impl<D, S> TestDataBuilder for AdjacencyGraphBuilder<D, S>
    where
        D: DirectednessTrait,
        S: Storage,
        (D::Symmetry, S): AdjacencyMatrixSelector<usize, String>,
    {
        type Graph = AdjacencyGraph<i32, String, D, S>;

        fn new_edge_data(i: usize) -> String {
            format!("e{}", i)
        }

        fn new_node_data(i: usize) -> i32 {
            i as i32
        }
    }

    macro_rules! test_compaction {
        ($builder:ty) => {
            #[cfg(not(feature = "unchecked"))]
            #[test]
            #[should_panic]
            fn test_check_node_id_panics_after_compaction() {
                type Graph = <$builder as TestDataBuilder>::Graph;
                let mut graph = Graph::default();
                let n1 = graph.add_node(1);
                graph.compact();
                graph.assert_valid_node_id(&n1);
            }

            #[cfg(not(feature = "unchecked"))]
            #[test]
            #[should_panic]
            fn test_check_edge_id_panics_after_compaction() {
                type Graph = <$builder as TestDataBuilder>::Graph;
                let mut graph = Graph::default();
                let n1 = graph.add_node(1);
                let n2 = graph.add_node(2);
                if let AddEdgeResult::Added(e1) = graph.add_edge(&n1, &n2, "edge".to_string()) {
                    graph.compact();
                    graph.assert_valid_edge_id(&e1);
                }
            }
        };
    }

    graph_tests!(
        directed_bitvec,
        AdjacencyGraphBuilder<Directed, BitvecStorage>,
        |data| data * 2,
        |data: &String| format!("{}-copied", data);

        test_compaction!(AdjacencyGraphBuilder<Directed, BitvecStorage>);
    );

    graph_tests!(
        undirected_bitvec,
        AdjacencyGraphBuilder<Undirected, BitvecStorage>,
        |data| data * 2,
        |data: &String| format!("{}-copied", data);

        test_compaction!(AdjacencyGraphBuilder<Undirected, BitvecStorage>);
    );

    graph_tests!(directed_hash, AdjacencyGraphBuilder<Directed, HashStorage>, |data| data * 2, |data: &String| format!(
        "{}-copied",
        data
    ));

    graph_tests!(undirected_hash, AdjacencyGraphBuilder<Undirected, HashStorage>, |data| data * 2, |data: &String| format!(
        "{}-copied",
        data
    ));
}
