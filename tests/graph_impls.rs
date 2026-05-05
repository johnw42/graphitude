mod linked {
    pub use graphitude::{BagGraph, prelude::*};
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
        type Graph = BagGraph<i32, String, D, M>;

        fn new_graph(&self) -> <Self as TestDataBuilder>::Graph {
            BagGraph::new(self.directedness, self.edge_multiplicity)
        }

        fn new_edge_data(&self, i: usize) -> String {
            format!("e{}", i)
        }

        fn new_node_data(&self, i: usize) -> i32 {
            i as i32
        }
    }

    graph_test_suite!(directed_multiple:
        GraphTests<LinkedGraphBuilder<Directed, MultipleEdges>> =
        GraphTests::new(
            LinkedGraphBuilder::new(Directed, MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data)));

    graph_test_suite!(directed_single:
        GraphTests<LinkedGraphBuilder<Directed, SingleEdge>> =
        GraphTests::new(
            LinkedGraphBuilder::new(Directed, SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data)));

    graph_test_suite!(undirected_multiple:
        GraphTests<LinkedGraphBuilder<Undirected, MultipleEdges>> =
        GraphTests::new(
            LinkedGraphBuilder::new(Undirected, MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data)));

    graph_test_suite!(undirected_single:
         GraphTests<LinkedGraphBuilder<Undirected, SingleEdge>> =
        GraphTests::new(
            LinkedGraphBuilder::new(Undirected, SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data)));

    graph_test_suite!(dyn_directed_multiple:
        GraphTests<LinkedGraphBuilder<Directedness, EdgeMultiplicity>> =
        GraphTests::new( LinkedGraphBuilder::new(
            Directedness::Directed, EdgeMultiplicity::MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data)));

    graph_test_suite!(dyn_directed_single:
        GraphTests<LinkedGraphBuilder<Directedness, EdgeMultiplicity>> =
        GraphTests::new(
            LinkedGraphBuilder::new(Directedness::Directed, EdgeMultiplicity::SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data)));

    graph_test_suite!(dyn_undirected_multiple:
        GraphTests<LinkedGraphBuilder<Directedness, EdgeMultiplicity>> =
        GraphTests::new(
            LinkedGraphBuilder::new(Directedness::Undirected, EdgeMultiplicity::MultipleEdges),
            |data| data * 2,
            |data| format!("{}-copied", data)));

    graph_test_suite!(dyn_undirected_single:
        GraphTests<LinkedGraphBuilder<Directedness, EdgeMultiplicity>> =
        GraphTests::new(
            LinkedGraphBuilder::new(Directedness::Undirected, EdgeMultiplicity::SingleEdge),
            |data| data * 2,
            |data| format!("{}-copied", data)));
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

    graph_test_suite!(
        directed_single_bitvec:
        GraphTests<AdjacencyGraphBuilder<Directed, SingleEdge, BitvecStorage>> =
        GraphTests::new(
            AdjacencyGraphBuilder::new(),
            |data| data * 2,
            |data: &String| format!("{}-copied", data)));

    graph_test_suite!(
        undirected_single_bitvec:
        GraphTests<AdjacencyGraphBuilder<Undirected, SingleEdge, BitvecStorage>> =
        GraphTests::new(
            AdjacencyGraphBuilder::new(),
            |data| data * 2,
            |data: &String| format!("{}-copied", data)));

    graph_test_suite!(
        directed_single_hash:
        GraphTests<AdjacencyGraphBuilder<Directed, SingleEdge, HashStorage>> =
        GraphTests::new(
            AdjacencyGraphBuilder::new(),
            |data| data * 2,
            |data: &String| format!("{}-copied", data)));

    graph_test_suite!(
        undirected_single_hash:
        GraphTests<AdjacencyGraphBuilder<Undirected, SingleEdge, HashStorage>> =
        GraphTests::new(
            AdjacencyGraphBuilder::new(),
            |data| data * 2,
            |data: &String| format!("{}-copied", data)));

    graph_test_suite!(
        directed_multiple_bitvec:
        GraphTests<AdjacencyGraphBuilder<Directed, MultipleEdges, BitvecStorage>> =
        GraphTests::new(
            AdjacencyGraphBuilder::new(),
            |data| data * 2,
            |data: &String| format!("{}-copied", data)));

    graph_test_suite!(
        undirected_multiple_bitvec:
        GraphTests<AdjacencyGraphBuilder<Undirected, MultipleEdges, BitvecStorage>> =
        GraphTests::new(
            AdjacencyGraphBuilder::new(),
            |data| data * 2,
            |data: &String| format!("{}-copied", data)));

    graph_test_suite!(
        directed_multiple_hash:
        GraphTests<AdjacencyGraphBuilder<Directed, MultipleEdges, HashStorage>> =
        GraphTests::new(
            AdjacencyGraphBuilder::new(),
            |data| data * 2,
            |data: &String| format!("{}-copied", data)));

    graph_test_suite!(
        undirected_multiple_hash:
        GraphTests<AdjacencyGraphBuilder<Undirected, MultipleEdges, HashStorage>> =
        GraphTests::new(
            AdjacencyGraphBuilder::new(),
            |data| data * 2,
            |data: &String| format!("{}-copied", data)));
}
