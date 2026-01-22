use std::marker::PhantomData;

use crate::Graph;

pub struct BuilderState {
    v: usize,
    e: usize,
}

impl BuilderState {
    pub fn next_vertex(&mut self) -> usize {
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

pub trait TestDataBuilder<G: Graph> {
    fn new() -> BuilderState {
        BuilderState { v: 0, e: 0 }
    }

    fn new_edge_data(i: usize) -> G::EdgeData;
    fn new_vertex_data(i: usize) -> G::VertexData;
}

pub struct Builder<G>(BuilderState, PhantomData<G>);

impl<G> Builder<G>
where
    G: Graph + TestDataBuilder<G>,
    <G as Graph>::VertexData: Clone + Eq,
    <G as Graph>::EdgeData: Clone + Eq,
{
    pub fn new() -> Self {
        Self(<G as TestDataBuilder<G>>::new(), PhantomData)
    }

    pub fn new_vertex_data(&mut self) -> G::VertexData {
        let id = self.0.next_vertex();
        <G as TestDataBuilder<G>>::new_vertex_data(id)
    }

    pub fn new_edge_data(&mut self) -> G::EdgeData {
        let id = self.0.next_edge();
        <G as TestDataBuilder<G>>::new_edge_data(id)
    }
}

#[macro_export]
macro_rules! graph_tests {
    ($type:ty) => {
        #[test]
        fn test_new_graph_is_empty() {
            let graph: $type = <$type>::new();
            assert_eq!(graph.num_vertices(), 0);
            assert_eq!(graph.num_edges(), 0);
        }

        #[test]
        fn test_vertex_data_retrieval() {
            let mut graph: $type = <$type>::new();
            let mut builder = crate::tests::Builder::<$type>::new();
            let vd1 = builder.new_vertex_data();
            let v1 = graph.add_vertex(vd1);
            assert_eq!(*graph.vertex_data(&v1), vd1);
        }

        #[test]
        fn test_edge_creation() {
            let mut graph: $type = <$type>::new();
            let mut builder = crate::tests::Builder::<$type>::new();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();
            let v1 = graph.add_vertex(vd1);
            let v2 = graph.add_vertex(vd2);
            let e1 = graph.add_edge(&v1, &v2, ed1.clone()).0;

            assert_eq!(graph.edge_ids().count(), 1);
            assert_eq!(*graph.edge_data(&e1), ed1);
        }

        #[test]
        fn test_vertex_removal() {
            let mut graph: $type = <$type>::new();
            let mut builder = crate::tests::Builder::<$type>::new();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let ed3 = builder.new_edge_data();

            let v1 = graph.add_vertex(vd1);
            let v2 = graph.add_vertex(vd2);

            // Normal edge.
            graph.add_edge(&v1, &v2, ed1.clone());
            // Duplicate edge.
            graph.add_edge(&v1, &v2, ed2.clone());
            // Self edge.
            graph.add_edge(&v1, &v1, ed3.clone());

            let removed_data = graph.remove_vertex(&v1);
            assert_eq!(removed_data, vd1);
            assert_eq!(graph.num_vertices(), 1);
            assert_eq!(graph.num_edges(), 0);
        }

        #[test]
        fn test_edges_out() {
            let mut graph: $type = <$type>::new();
            let mut builder = crate::tests::Builder::<$type>::new();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();

            let v1 = graph.add_vertex(vd1);
            let v2 = graph.add_vertex(vd2);
            let e1 = graph.add_edge(&v1, &v2, ed1.clone()).0;

            assert_eq!(graph.edges_out(v1).collect::<Vec<_>>(), vec![e1]);
            assert_eq!(graph.num_edges_out(v2), 0);
        }

        #[test]
        fn test_edges_between() {
            let mut graph: $type = <$type>::new();
            let mut builder = crate::tests::Builder::<$type>::new();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();

            let v1 = graph.add_vertex(vd1);
            let v2 = graph.add_vertex(vd2);
            graph.add_edge(&v1, &v2, ed1);

            assert_eq!(graph.num_edges_between(v1, v2), 1);
            assert_eq!(graph.num_edges_between(v2, v1), 0);
        }
    };
}
