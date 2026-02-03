use std::collections::HashMap;
use std::fmt::Debug;

use crate::graph::{EdgeId, Graph};

// Generates a DOT representation for any `Graph` implementation.
#[cfg(feature = "dot")]
pub fn generate_dot_file<G>(graph: &G) -> Vec<u8>
where
    G: Graph,
    G::NodeData: Debug,
    G::EdgeData: Debug,
{
    struct GraphWrapper<'a, G: Graph> {
        graph: &'a G,
        node_id_map: HashMap<G::NodeId, usize>,
    }

    impl<'a, G: Graph> GraphWrapper<'a, G> {
        fn new(graph: &'a G) -> Self {
            let node_id_map = graph
                .node_ids()
                .enumerate()
                .map(|(i, nid)| (nid, i))
                .collect();
            Self { graph, node_id_map }
        }
    }

    impl<'a, G> ::dot::Labeller<'a, G::NodeId, G::EdgeId> for GraphWrapper<'a, G>
    where
        G: Graph,
        G::NodeData: Debug,
        G::EdgeData: Debug,
    {
        fn graph_id(&'a self) -> ::dot::Id<'a> {
            ::dot::Id::new("G").unwrap()
        }

        fn node_id(&'a self, n: &G::NodeId) -> ::dot::Id<'a> {
            let idx = self.node_id_map.get(n).unwrap();
            ::dot::Id::new(format!("n{}", idx)).unwrap()
        }

        fn node_label(&'a self, n: &G::NodeId) -> ::dot::LabelText<'a> {
            let data = self.graph.node_data(n);
            ::dot::LabelText::LabelStr(format!("{:?}", data).into())
        }

        fn edge_label(&'a self, e: &G::EdgeId) -> ::dot::LabelText<'a> {
            let data = self.graph.edge_data(e);
            ::dot::LabelText::LabelStr(format!("{:?}", data).into())
        }
    }

    impl<'a, G> ::dot::GraphWalk<'a, G::NodeId, G::EdgeId> for GraphWrapper<'a, G>
    where
        G: Graph,
    {
        fn nodes(&'a self) -> ::dot::Nodes<'a, G::NodeId> {
            self.graph.node_ids().collect::<Vec<_>>().into()
        }

        fn edges(&'a self) -> ::dot::Edges<'a, G::EdgeId> {
            self.graph.edge_ids().collect::<Vec<_>>().into()
        }

        fn source(&'a self, edge: &G::EdgeId) -> G::NodeId {
            edge.source()
        }

        fn target(&'a self, edge: &G::EdgeId) -> G::NodeId {
            edge.target()
        }
    }

    let wrapper = GraphWrapper::new(graph);
    let mut output = Vec::new();

    if graph.is_directed() {
        ::dot::render(&wrapper, &mut output).unwrap();
    } else {
        ::dot::render(&wrapper, &mut output).unwrap();
    }

    output
}
