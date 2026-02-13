mod linked {
    pub use graphitude::{LinkedGraph, graph_tests, prelude::*, tests::TestDataBuilder};
    use graphitude::{directedness::Directedness, edge_multiplicity::EdgeMultiplicity};

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

        fn new_graph(&self) -> Self::Graph {
            LinkedGraph::new(self.directedness, self.edge_multiplicity)
        }

        fn new_edge_data(&self, i: usize) -> String {
            format!("e{}", i)
        }

        fn new_node_data(&self, i: usize) -> i32 {
            i as i32
        }
    }

    graph_tests!(directed_multiple, LinkedGraphBuilder<Directed, MultipleEdges>, LinkedGraphBuilder::new(Directed, MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(directed_single, LinkedGraphBuilder<Directed, SingleEdge>, LinkedGraphBuilder::new(Directed, SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(undirected_multiple, LinkedGraphBuilder<Undirected, MultipleEdges>, LinkedGraphBuilder::new(Undirected, MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(undirected_single, LinkedGraphBuilder<Undirected, SingleEdge>, LinkedGraphBuilder::new(Undirected, SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(dyn_directed_multiple, LinkedGraphBuilder<Directedness, EdgeMultiplicity>, LinkedGraphBuilder::new(Directedness::Directed, EdgeMultiplicity::MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(dyn_directed_single, LinkedGraphBuilder<Directedness, EdgeMultiplicity>, LinkedGraphBuilder::new(Directedness::Directed, EdgeMultiplicity::SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(dyn_undirected_multiple, LinkedGraphBuilder<Directedness, EdgeMultiplicity>, LinkedGraphBuilder::new(Directedness::Undirected, EdgeMultiplicity::MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data));

    graph_tests!(dyn_undirected_single, LinkedGraphBuilder<Directedness, EdgeMultiplicity>, LinkedGraphBuilder::new(Directedness::Undirected, EdgeMultiplicity::SingleEdge),
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

    impl<D, S> AdjacencyGraphBuilder<D, S> {
        pub fn new() -> Self {
            Self(PhantomData)
        }
    }

    impl<D, S> TestDataBuilder for AdjacencyGraphBuilder<D, S>
    where
        D: DirectednessTrait + Default,
        S: Storage,
        (D, S): AdjacencyMatrixSelector<usize, String>,
    {
        type Graph = AdjacencyGraph<i32, String, D, S>;

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
        AdjacencyGraphBuilder::new(),
        |data| data * 2,
        |data: &String| format!("{}-copied", data);

        test_compaction!(AdjacencyGraphBuilder<Directed, BitvecStorage>);
    );

    graph_tests!(
        undirected_bitvec,
        AdjacencyGraphBuilder<Undirected, BitvecStorage>,
        AdjacencyGraphBuilder::new(),
        |data| data * 2,
        |data: &String| format!("{}-copied", data);

        test_compaction!(AdjacencyGraphBuilder<Undirected, BitvecStorage>);
    );

    graph_tests!(
        directed_hash,
        AdjacencyGraphBuilder<Directed, HashStorage>,
        AdjacencyGraphBuilder::new(),
        |data| data * 2,
        |data: &String| format!("{}-copied", data));

    graph_tests!(
        undirected_hash,
        AdjacencyGraphBuilder<Undirected, HashStorage>,
        AdjacencyGraphBuilder::new(),
        |data| data * 2,
        |data: &String| format!("{}-copied", data));
}
