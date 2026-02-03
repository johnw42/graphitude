use std::collections::HashMap;

use crate::{
    dot::attr::Attr,
    graph::{EdgeId, Graph},
};

/// Trait for generating DOT representations of graphs.  Users can implement
/// this trait to customize node names and attributes in the generated DOT file.
pub trait DotGenerator<G: Graph> {
    /// Returns the name of the graph to be used in the DOT output.
    /// By default, this returns "G".
    fn graph_name(&self) -> String {
        "G".to_string()
    }

    /// Returns an identifier for a node to be used in the DOT output.  The
    /// `index` parameter is a zero-based index of the node in the graph's node
    /// IDs iterator, which can be used to generate unique names.
    ///
    /// By default, this returns "n{index}".
    fn node_name(&self, node_id: &G::NodeId, index: usize) -> String {
        let _ = node_id;
        format!("n{}", index)
    }

    /// Returns a list of attributes for a given node.  The `name` parameter is
    /// the name returned by `node_name` for the same node.  After this method
    /// has been called for each node, all nodes must have unique names.
    fn node_attrs(&self, node_id: &G::NodeId, name: &mut String) -> Vec<Attr> {
        let _ = (node_id, name);
        vec![]
    }

    /// Returns a list of attributes for a given edge.
    fn edge_attrs(&self, edge_id: &G::EdgeId) -> Vec<Attr> {
        let _ = edge_id;
        vec![]
    }
}

// Generates a DOT representation for any `Graph` implementation.
#[cfg(feature = "dot")]
pub fn generate_dot_file<G, D>(graph: &G, generator: &D) -> Vec<u8>
where
    G: Graph,
    D: DotGenerator<G>,
{
    struct GraphWrapper<'a, G: Graph, D: DotGenerator<G>> {
        graph: &'a G,
        generator: &'a D,
        node_id_map: HashMap<G::NodeId, usize>,
    }

    impl<'a, G: Graph, D: DotGenerator<G>> GraphWrapper<'a, G, D> {
        fn new(graph: &'a G, generator: &'a D) -> Self {
            let node_id_map = graph
                .node_ids()
                .enumerate()
                .map(|(i, nid)| (nid, i))
                .collect();
            Self {
                graph,
                generator,
                node_id_map,
            }
        }
    }

    impl<'a, G, D> ::dot::Labeller<'a, G::NodeId, G::EdgeId> for GraphWrapper<'a, G, D>
    where
        G: Graph,
        D: DotGenerator<G>,
    {
        fn graph_id(&'a self) -> ::dot::Id<'a> {
            ::dot::Id::new(self.generator.graph_name()).unwrap()
        }

        fn node_id(&'a self, n: &G::NodeId) -> ::dot::Id<'a> {
            let idx = self.node_id_map.get(n).unwrap();
            let mut name = self.generator.node_name(n, *idx);
            // Ensure node_attrs has been called to allow name modification
            let _attrs = self.generator.node_attrs(n, &mut name);
            ::dot::Id::new(name).unwrap()
        }

        fn node_label(&'a self, n: &G::NodeId) -> ::dot::LabelText<'a> {
            let idx = self.node_id_map.get(n).unwrap();
            let mut name = self.generator.node_name(n, *idx);
            let attrs = self.generator.node_attrs(n, &mut name);

            // Build label from attributes if present
            if attrs.is_empty() {
                ::dot::LabelText::LabelStr(name.into())
            } else {
                let attr_strs: Vec<String> = attrs
                    .iter()
                    .map(|a| format!("{}={}", a.name(), a.value()))
                    .collect();
                ::dot::LabelText::LabelStr(attr_strs.join(", ").into())
            }
        }

        fn edge_label(&'a self, e: &G::EdgeId) -> ::dot::LabelText<'a> {
            let attrs = self.generator.edge_attrs(e);
            if attrs.is_empty() {
                ::dot::LabelText::LabelStr("".into())
            } else {
                let attr_strs: Vec<String> = attrs
                    .iter()
                    .map(|a| format!("{}={}", a.name(), a.value()))
                    .collect();
                ::dot::LabelText::LabelStr(attr_strs.join(", ").into())
            }
        }
    }

    impl<'a, G, D> ::dot::GraphWalk<'a, G::NodeId, G::EdgeId> for GraphWrapper<'a, G, D>
    where
        G: Graph,
        D: DotGenerator<G>,
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

    let wrapper = GraphWrapper::new(graph, generator);
    let mut output = Vec::new();

    ::dot::render(&wrapper, &mut output).unwrap();

    output
}
