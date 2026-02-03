use std::collections::HashMap;
use std::error::Error;
use std::io;

use crate::{
    dot::attr::Attr,
    graph::{EdgeId, Graph},
};

/// Errors that can occur during DOT file generation.
#[derive(Debug, thiserror::Error)]
pub enum DotError<E> {
    /// Invalid identifier for DOT format.
    #[error("Invalid DOT identifier: {0}")]
    InvalidId(String),
    /// IO error during rendering.
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    /// Error from the user-provided generator.
    #[error("Generator error: {0}")]
    Generator(#[source] E),
}

/// Trait for generating DOT representations of graphs.  Users can implement
/// this trait to customize node names and attributes in the generated DOT file.
pub trait DotGenerator<G: Graph> {
    type Error: Error + 'static;

    /// Returns the name of the graph to be used in the DOT output.
    /// By default, this returns "G".
    fn graph_name(&self) -> Result<String, Self::Error> {
        Ok("G".to_string())
    }

    /// Returns an identifier for a node to be used in the DOT output.  The
    /// `index` parameter is a zero-based index of the node in the graph's node
    /// IDs iterator, which can be used to generate unique names.
    ///
    /// By default, this returns "n{index}".
    fn node_name(&self, node_id: &G::NodeId, index: usize) -> Result<String, Self::Error> {
        let _ = node_id;
        Ok(format!("n{}", index))
    }

    /// Returns a list of attributes for a given node.  The `name` parameter is
    /// the name returned by `node_name` for the same node.  After this method
    /// has been called for each node, all nodes must have unique names.
    fn node_attrs(&self, node_id: &G::NodeId, name: &mut String) -> Result<Vec<Attr>, Self::Error> {
        let _ = (node_id, name);
        Ok(vec![])
    }

    /// Returns a list of attributes for a given edge.
    fn edge_attrs(&self, edge_id: &G::EdgeId) -> Result<Vec<Attr>, Self::Error> {
        let _ = edge_id;
        Ok(vec![])
    }
}

// Generates a DOT representation for any `Graph` implementation.
#[cfg(feature = "dot")]
pub fn generate_dot_file<G, D>(
    graph: &G,
    generator: &D,
    output: &mut impl io::Write,
) -> Result<(), DotError<D::Error>>
where
    G: Graph,
    D: DotGenerator<G>,
{
    struct NodeInfo {
        name: String,
        attrs: Vec<Attr>,
    }

    struct GraphWrapper<'a, G: Graph> {
        graph: &'a G,
        node_info: HashMap<G::NodeId, NodeInfo>,
        graph_name: String,
    }

    impl<'a, G: Graph> GraphWrapper<'a, G> {
        fn new<D: DotGenerator<G>>(
            graph: &'a G,
            generator: &D,
        ) -> Result<Self, DotError<D::Error>> {
            // Validate graph name
            let graph_name = generator.graph_name().map_err(DotError::Generator)?;
            ::dot::Id::new(graph_name.as_str())
                .map_err(|()| DotError::InvalidId(graph_name.clone()))?;

            // Pre-generate and validate all node names and attributes
            let mut node_info = HashMap::new();
            for (index, node_id) in graph.node_ids().enumerate() {
                let mut name = generator
                    .node_name(&node_id, index)
                    .map_err(DotError::Generator)?;
                let attrs = generator
                    .node_attrs(&node_id, &mut name)
                    .map_err(DotError::Generator)?;

                // Validate the node name is a valid DOT identifier
                ::dot::Id::new(name.as_str()).map_err(|()| DotError::InvalidId(name.clone()))?;

                node_info.insert(node_id.clone(), NodeInfo { name, attrs });
            }

            Ok(Self {
                graph,
                node_info,
                graph_name,
            })
        }
    }

    impl<'a, G> ::dot::Labeller<'a, G::NodeId, G::EdgeId> for GraphWrapper<'a, G>
    where
        G: Graph,
    {
        fn graph_id(&'a self) -> ::dot::Id<'a> {
            // Safe to unwrap since we validated this in new()
            ::dot::Id::new(self.graph_name.as_str()).expect("Graph name was pre-validated")
        }

        fn node_id(&'a self, n: &G::NodeId) -> ::dot::Id<'a> {
            let info = self.node_info.get(n).expect("Node ID should exist in map");
            // Safe to unwrap since we validated all names in new()
            ::dot::Id::new(info.name.as_str()).expect("Node name was pre-validated")
        }

        fn node_label(&'a self, n: &G::NodeId) -> ::dot::LabelText<'a> {
            let info = self.node_info.get(n).expect("Node ID should exist in map");

            // Build label from attributes if present
            if info.attrs.is_empty() {
                ::dot::LabelText::LabelStr(info.name.clone().into())
            } else {
                let attr_strs: Vec<String> = info
                    .attrs
                    .iter()
                    .map(|a| format!("{}={}", a.name(), a.value()))
                    .collect();
                ::dot::LabelText::LabelStr(attr_strs.join(", ").into())
            }
        }

        fn edge_label(&'a self, _e: &G::EdgeId) -> ::dot::LabelText<'a> {
            // Note: edge attributes are not pre-generated since they don't affect validation
            ::dot::LabelText::LabelStr("".into())
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

    let wrapper = GraphWrapper::new(graph, generator)?;
    ::dot::render(&wrapper, output).map_err(DotError::IoError)
}
