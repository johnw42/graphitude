use std::collections::HashMap;
use std::error::Error;
use std::io;

use crate::{
    directedness::Directedness,
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

    struct EdgeInfo {
        attrs: Vec<Attr>,
    }

    struct GraphWrapper<'a, G: Graph> {
        _phantom: std::marker::PhantomData<&'a G>,
        node_info: HashMap<G::NodeId, NodeInfo>,
        edge_info: HashMap<G::EdgeId, EdgeInfo>,
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

            // Pre-generate edge attributes
            let mut edge_info = HashMap::new();
            for edge_id in graph.edge_ids() {
                let attrs = generator
                    .edge_attrs(&edge_id)
                    .map_err(DotError::Generator)?;
                edge_info.insert(edge_id.clone(), EdgeInfo { attrs });
            }

            Ok(Self {
                _phantom: std::marker::PhantomData,
                node_info,
                edge_info,
                graph_name,
            })
        }
    }

    let wrapper = GraphWrapper::new(graph, generator)?;

    // Check if a string needs to be quoted in DOT format
    fn needs_quoting(s: &str) -> bool {
        if s.is_empty() {
            return true;
        }

        // Check if it's a valid unquoted identifier
        // Must start with letter or underscore, contain only alphanumeric or underscore
        let mut chars = s.chars();
        let first = chars.next().unwrap();

        if !first.is_ascii_alphabetic() && first != '_' {
            // Numbers at the start are OK for numeric literals
            if !first.is_ascii_digit() {
                return true;
            }
            // If it starts with a digit, check if it's a valid number
            return !s
                .chars()
                .all(|c| c.is_ascii_digit() || c == '.' || c == '-');
        }

        // Check remaining characters
        !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }

    // Escape a string for use in DOT format (only called when quoting is needed)
    fn escape_dot_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }

    // Format a value for DOT output, adding quotes only if needed
    fn format_dot_value(s: &str) -> String {
        if needs_quoting(s) {
            format!("\"{}\"", escape_dot_string(s))
        } else {
            s.to_string()
        }
    }

    // Custom DOT renderer that properly handles optional attributes
    let is_directed = G::Directedness::is_directed();
    let graph_type = if is_directed { "digraph" } else { "graph" };
    let edge_op = if is_directed { "->" } else { "--" };

    // Write graph header
    writeln!(output, "{} {} {{", graph_type, wrapper.graph_name).map_err(DotError::IoError)?;

    // Write nodes
    for node_id in graph.node_ids() {
        let node_info = wrapper
            .node_info
            .get(&node_id)
            .expect("Node ID should exist in map");
        write!(output, "    {}", node_info.name).map_err(DotError::IoError)?;

        // Write node attributes if any
        if !node_info.attrs.is_empty() {
            write!(output, " [").map_err(DotError::IoError)?;
            for (i, attr) in node_info.attrs.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ").map_err(DotError::IoError)?;
                }
                write!(
                    output,
                    "{} = {}",
                    attr.name(),
                    format_dot_value(&attr.value())
                )
                .map_err(DotError::IoError)?;
            }
            write!(output, "]").map_err(DotError::IoError)?;
        } else {
            // Default label is the node name
            write!(output, " [label = {}]", format_dot_value(&node_info.name))
                .map_err(DotError::IoError)?;
        }

        writeln!(output, ";").map_err(DotError::IoError)?;
    }

    // Blank line between nodes and edges
    writeln!(output).map_err(DotError::IoError)?;

    // Write edges
    for edge_id in graph.edge_ids() {
        let source_id = edge_id.source();
        let target_id = edge_id.target();
        let source_info = wrapper
            .node_info
            .get(&source_id)
            .expect("Source node should exist");
        let target_info = wrapper
            .node_info
            .get(&target_id)
            .expect("Target node should exist");
        let edge_info = wrapper
            .edge_info
            .get(&edge_id)
            .expect("Edge ID should exist in map");

        write!(
            output,
            "    {} {} {}",
            source_info.name, edge_op, target_info.name
        )
        .map_err(DotError::IoError)?;

        // Write edge attributes if any - omit entirely if no attributes
        if !edge_info.attrs.is_empty() {
            write!(output, " [").map_err(DotError::IoError)?;
            for (i, attr) in edge_info.attrs.iter().enumerate() {
                if i > 0 {
                    write!(output, ", ").map_err(DotError::IoError)?;
                }
                write!(
                    output,
                    "{} = {}",
                    attr.name(),
                    format_dot_value(&attr.value())
                )
                .map_err(DotError::IoError)?;
            }
            write!(output, "]").map_err(DotError::IoError)?;
        }

        writeln!(output, ";").map_err(DotError::IoError)?;
    }

    writeln!(output, "}}").map_err(DotError::IoError)
}
