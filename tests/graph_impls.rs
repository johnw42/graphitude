mod bag {
    pub use graphitude::{BagGraph, prelude::*};
    use graphitude::{graph_test_suite, graph_tests::GraphTests};

    graph_test_suite!(directed:
        GraphTests<BagGraph<String, String, Directed>>);

    graph_test_suite!(undirected:
        GraphTests<BagGraph<String, String, Undirected>>);
}

mod linked {
    pub use graphitude::{LinkedGraph, prelude::*};
    use graphitude::{graph_test_suite, graph_tests::GraphTests};

    graph_test_suite!(directed_multiple:
        GraphTests<LinkedGraph<String, String, Directed, MultipleEdges>>);

    graph_test_suite!(directed_single:
        GraphTests<LinkedGraph<String, String, Directed, SingleEdge>>);

    graph_test_suite!(undirected_multiple:
        GraphTests<LinkedGraph<String, String, Undirected, MultipleEdges>>);

    graph_test_suite!(undirected_single:
         GraphTests<LinkedGraph<String, String, Undirected, SingleEdge>>);
}

mod adjacency {
    use graphitude::{AdjacencyGraph, graph_test_suite};
    use graphitude::{BitvecStorage, HashStorage, graph_tests::GraphTests, prelude::*};

    graph_test_suite!(directed_single_bitvec:
        GraphTests<AdjacencyGraph<String, String, Directed, SingleEdge, BitvecStorage>>);

    graph_test_suite!(undirected_single_bitvec:
        GraphTests<AdjacencyGraph<String, String, Undirected, SingleEdge, BitvecStorage>>);

    graph_test_suite!(directed_single_hash:
        GraphTests<AdjacencyGraph<String, String, Directed, SingleEdge, HashStorage>>);

    graph_test_suite!(undirected_single_hash:
        GraphTests<AdjacencyGraph<String, String, Undirected, SingleEdge, HashStorage>>);

    graph_test_suite!(directed_multiple_bitvec:
        GraphTests<AdjacencyGraph<String, String, Directed, MultipleEdges, BitvecStorage>>);

    graph_test_suite!(undirected_multiple_bitvec:
        GraphTests<AdjacencyGraph<String, String, Undirected, MultipleEdges, BitvecStorage>>);

    graph_test_suite!(directed_multiple_hash:
        GraphTests<AdjacencyGraph<String, String, Directed, MultipleEdges, HashStorage>>);

    graph_test_suite!(undirected_multiple_hash:
        GraphTests<AdjacencyGraph<String, String, Undirected, MultipleEdges, HashStorage>>);
}
