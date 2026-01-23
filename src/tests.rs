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

pub trait TestDataBuilder {
    type Graph: Graph;

    fn new() -> BuilderState {
        BuilderState { v: 0, e: 0 }
    }

    fn new_graph() -> Self::Graph;
    fn new_edge_data(i: usize) -> <Self::Graph as Graph>::EdgeData;
    fn new_vertex_data(i: usize) -> <Self::Graph as Graph>::VertexData;
}

pub struct InternalBuilderImpl<G>(BuilderState, PhantomData<G>);

impl<G> InternalBuilderImpl<G>
where
    G: Graph + TestDataBuilder<Graph = G>,
    G::VertexData: Clone + Eq,
    G::EdgeData: Clone + Eq,
{
    pub fn new() -> Self {
        Self(<G as TestDataBuilder>::new(), PhantomData)
    }

    pub fn new_graph(&self) -> G {
        G::new_graph()
    }

    pub fn new_vertex_data(&mut self) -> G::VertexData {
        let id = self.0.next_vertex();
        G::new_vertex_data(id)
    }

    pub fn new_edge_data(&mut self) -> G::EdgeData {
        let id = self.0.next_edge();
        G::new_edge_data(id)
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
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = builder.new_graph();
            let vd1 = builder.new_vertex_data();
            let v1 = graph.add_vertex(vd1.clone());
            assert_eq!(*graph.vertex_data(&v1), vd1);
        }

        #[test]
        fn test_edge_creation() {
            use std::collections::HashSet;

            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = builder.new_graph();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let vd3 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let v1 = graph.add_vertex(vd1);
            let v2 = graph.add_vertex(vd2);
            let v3 = graph.add_vertex(vd3);
            let e1 = graph.add_edge(&v1, &v2, ed1.clone());
            let e2 = graph.add_edge(&v2, &v3, ed2.clone());

            assert_eq!(
                graph
                    .edges_from(v1.clone())
                    .into_iter()
                    .map(|edge_id| graph.edge_target(edge_id))
                    .collect::<Vec<_>>(),
                vec![v2.clone()]
            );

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
        fn test_vertex_removal() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = builder.new_graph();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let ed3 = builder.new_edge_data();

            let v1 = graph.add_vertex(vd1.clone());
            let v2 = graph.add_vertex(vd2.clone());

            // Normal edge.
            graph.add_or_replace_edge(&v1, &v2, ed1.clone());
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
        fn test_remove_vertex_cleans_edges() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = builder.new_graph();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();
            let ed2 = builder.new_edge_data();
            let ed3 = builder.new_edge_data();

            let v1 = graph.add_vertex(vd1.clone());
            let v2 = graph.add_vertex(vd2.clone());

            // Normal edge.
            graph.add_or_replace_edge(&v1, &v2, ed1.clone());
            // Duplicate edge.
            graph.add_edge(&v1, &v2, ed2.clone());
            // Self edge.
            graph.add_edge(&v1, &v1, ed3.clone());

            graph.remove_vertex(&v1);
            assert_eq!(graph.num_vertices(), 1);
            assert_eq!(graph.num_edges(), 0);
        }

        #[test]
        fn test_edges_out() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = builder.new_graph();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();

            let v1 = graph.add_vertex(vd1);
            let v2 = graph.add_vertex(vd2);
            let e1 = graph.add_or_replace_edge(&v1, &v2, ed1.clone()).0;

            assert_eq!(graph.edges_from(v1).collect::<Vec<_>>(), vec![e1]);
            assert_eq!(graph.num_edges_from(v2), (!graph.is_directed()).into());
        }

        #[test]
        fn test_edges_in() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = builder.new_graph();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();

            let v1 = graph.add_vertex(vd1);
            let v2 = graph.add_vertex(vd2);
            let e1 = graph.add_or_replace_edge(&v1, &v2, ed1.clone()).0;

            assert_eq!(graph.edges_into(v2).collect::<Vec<_>>(), vec![e1]);
            assert_eq!(graph.num_edges_into(v1), (!graph.is_directed()).into());
        }

        #[test]
        fn test_edges_between() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = builder.new_graph();
            let vd1 = builder.new_vertex_data();
            let vd2 = builder.new_vertex_data();
            let ed1 = builder.new_edge_data();

            let v1 = graph.add_vertex(vd1);
            let v2 = graph.add_vertex(vd2);
            let e1 = graph.add_edge(&v1, &v2, ed1);

            assert_eq!(graph.num_edges_between(v1.clone(), v2.clone()), 1);
            assert_eq!(
                graph
                    .edges_between(v1.clone(), v2.clone())
                    .collect::<Vec<_>>(),
                vec![e1.clone()]
            );
            assert_eq!(
                graph.edges_between(v2, v1).collect::<Vec<_>>(),
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
            let mut source = builder.new_graph();
            let v1 = source.add_vertex(builder.new_vertex_data());
            let v2 = source.add_vertex(builder.new_vertex_data());
            let v3 = source.add_vertex(builder.new_vertex_data());
            let e1 = source.add_edge(&v1, &v2, builder.new_edge_data());
            let e2 = source.add_edge(&v2, &v3, builder.new_edge_data());

            let mut target = builder.new_graph();
            let vertex_map = target.copy_from(&source);
            let edge_map = source.make_edge_map(&target, &vertex_map);

            assert_eq!(target.vertex_ids().count(), 3);
            assert_eq!(target.edge_ids().count(), 2);
            assert_eq!(
                source.vertex_data(&v1),
                target.vertex_data(&vertex_map[&v1])
            );
            assert_eq!(
                source.vertex_data(&v2),
                target.vertex_data(&vertex_map[&v2])
            );
            assert_eq!(
                source.vertex_data(&v3),
                target.vertex_data(&vertex_map[&v3])
            );
            assert_eq!(source.edge_data(&e1), target.edge_data(&edge_map[&e1]));
            assert_eq!(source.edge_data(&e2), target.edge_data(&edge_map[&e2]));
        }

        #[test]
        fn test_clear() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut graph = builder.new_graph();
            let v1 = graph.add_vertex(builder.new_vertex_data());
            let v2 = graph.add_vertex(builder.new_vertex_data());
            graph.add_edge(&v1, &v2, builder.new_edge_data());

            assert_eq!(graph.vertex_ids().count(), 2);
            assert_eq!(graph.edge_ids().count(), 1);

            graph.clear();

            assert_eq!(graph.vertex_ids().count(), 0);
            assert_eq!(graph.edge_ids().count(), 0);
        }
    };
}

#[macro_export]
macro_rules! graph_test_copy_from_with {
    ($type:ty, $f:expr, $g:expr) => {
        #[test]
        fn test_copy_from_with() {
            let mut builder = $crate::tests::InternalBuilderImpl::<$type>::new();
            let mut source = builder.new_graph();
            let v1 = source.add_vertex(builder.new_vertex_data());
            let v2 = source.add_vertex(builder.new_vertex_data());
            let v3 = source.add_vertex(builder.new_vertex_data());
            let e1 = source.add_edge(&v1, &v2, builder.new_edge_data());
            let e2 = source.add_edge(&v2, &v3, builder.new_edge_data());

            let mut target = builder.new_graph();
            let f: Box<dyn Fn(&<$type as Graph>::VertexData) -> <$type as Graph>::VertexData> = Box::new($f);
            let g: Box<dyn Fn(&<$type as Graph>::EdgeData) -> <$type as Graph>::EdgeData> = Box::new($g);
            let vertex_map = target.copy_from_with(&source, &f, &g);
            let edge_map = source.make_edge_map(&target, &vertex_map);

            assert_eq!(target.vertex_ids().count(), 3);
            assert_eq!(target.edge_ids().count(), 2);
            assert_eq!(
                f(source.vertex_data(&v1)),
                *target.vertex_data(&vertex_map[&v1])
            );
            assert_eq!(
                f(source.vertex_data(&v2)),
                *target.vertex_data(&vertex_map[&v2])
            );
            assert_eq!(
                f(source.vertex_data(&v3)),
                *target.vertex_data(&vertex_map[&v3])
            );
            assert_eq!(g(source.edge_data(&e1)), *target.edge_data(&edge_map[&e1]));
            assert_eq!(g(source.edge_data(&e2)), *target.edge_data(&edge_map[&e2]));
        }
    };
}
