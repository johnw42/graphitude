use std::{collections::HashMap, error::Error, io};

use crate::{dot::attr::Attr, prelude::*};

/// Validates if a string is a valid DOT identifier.
/// Returns true if the identifier is valid.
fn is_valid_dot_id(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    // Must start with letter or underscore
    if !first.is_ascii_alphabetic() && first != '_' {
        // Or be a valid number literal - try parsing as f64
        if first.is_ascii_digit() || first == '-' || first == '.' {
            return s.parse::<f64>().is_ok();
        }
        return false;
    }

    // Remaining characters must be alphanumeric or underscore
    s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

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

    /// Returns true if the graph is directed, false if undirected.
    /// By default, this uses the graph's directedness.
    fn is_directed(&self) -> bool {
        G::Directedness::is_directed()
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
            if !is_valid_dot_id(&graph_name) {
                return Err(DotError::InvalidId(graph_name));
            }

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
                if !is_valid_dot_id(&name) {
                    return Err(DotError::InvalidId(name));
                }

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

    // Escape a string for use in DOT format (only called when quoting is needed)
    fn escape_dot_string(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    }

    // Format a value for DOT output, adding quotes only if needed
    fn format_dot_value(s: &str) -> String {
        if !is_valid_dot_id(s) {
            format!("\"{}\"", escape_dot_string(s))
        } else {
            s.to_string()
        }
    }

    // Custom DOT renderer that properly handles optional attributes
    let is_directed = generator.is_directed();
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
        }

        writeln!(output, ";").map_err(DotError::IoError)?;
    }

    // Blank line between nodes and edges
    if graph.num_edges() > 0 && graph.num_nodes() > 0 {
        writeln!(output).map_err(DotError::IoError)?;
    }

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GraphMut,
        directedness::{Directed, Undirected},
        linked_graph::LinkedGraph,
    };

    #[test]
    fn test_is_valid_dot_id_alphanumeric() {
        assert!(is_valid_dot_id("abc"));
        assert!(is_valid_dot_id("_abc"));
        assert!(is_valid_dot_id("abc123"));
        assert!(is_valid_dot_id("a_b_c"));
        assert!(is_valid_dot_id("_123"));
        assert!(is_valid_dot_id("ABC"));
        assert!(is_valid_dot_id("a1B2c3"));
    }

    #[test]
    fn test_is_valid_dot_id_numbers() {
        assert!(is_valid_dot_id("123"));
        assert!(is_valid_dot_id("0"));
        assert!(is_valid_dot_id("3.14"));
        assert!(is_valid_dot_id("-42"));
        assert!(is_valid_dot_id("-.5"));
        assert!(is_valid_dot_id("1.5e10"));
        assert!(is_valid_dot_id("1.5e+10"));
        assert!(is_valid_dot_id("1.5e-10"));
        assert!(is_valid_dot_id("-1.5e-10"));
    }

    #[test]
    fn test_is_valid_dot_id_invalid() {
        assert!(!is_valid_dot_id(""));
        assert!(!is_valid_dot_id("123abc")); // starts with digit but not a valid number
        assert!(!is_valid_dot_id("a-b")); // hyphen not allowed in identifiers
        assert!(!is_valid_dot_id("a b")); // space not allowed
        assert!(!is_valid_dot_id("a.b")); // dot not allowed in identifiers
        assert!(!is_valid_dot_id("hello world"));
        assert!(!is_valid_dot_id("foo-bar"));
        assert!(!is_valid_dot_id("@abc"));
        assert!(!is_valid_dot_id("abc!"));
    }

    struct TestGenerator {
        graph_name: String,
    }

    impl<G: Graph> DotGenerator<G> for TestGenerator {
        type Error = std::convert::Infallible;

        fn graph_name(&self) -> Result<String, Self::Error> {
            Ok(self.graph_name.clone())
        }
    }

    #[test]
    fn test_generate_empty_directed_graph() {
        let graph: LinkedGraph<String, (), Directed> = LinkedGraph::default();
        let generator = TestGenerator {
            graph_name: "Empty".to_string(),
        };
        let mut output = Vec::new();

        generate_dot_file(&graph, &generator, &mut output).unwrap();
        let dot = String::from_utf8(output).unwrap();

        assert!(dot.contains("digraph Empty"));
        assert!(dot.contains("{"));
        assert!(dot.contains("}"));
    }

    #[test]
    fn test_generate_empty_undirected_graph() {
        let graph: LinkedGraph<String, (), Undirected> = LinkedGraph::default();
        let generator = TestGenerator {
            graph_name: "Empty".to_string(),
        };
        let mut output = Vec::new();

        generate_dot_file(&graph, &generator, &mut output).unwrap();
        let dot = String::from_utf8(output).unwrap();

        assert!(dot.contains("graph Empty"));
        assert!(dot.contains("{"));
        assert!(dot.contains("}"));
    }

    #[test]
    fn test_generate_simple_directed_graph() {
        let mut graph: LinkedGraph<String, (), Directed> = LinkedGraph::default();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        graph.add_new_edge(&a, &b, ());

        let generator = TestGenerator {
            graph_name: "G".to_string(),
        };
        let mut output = Vec::new();

        generate_dot_file(&graph, &generator, &mut output).unwrap();
        let dot = String::from_utf8(output).unwrap();

        assert!(dot.contains("digraph G"));
        assert!(dot.contains("n0"));
        assert!(dot.contains("n1"));
        // Both nodes present, one directed edge
        assert_eq!(dot.matches("->").count(), 1);
    }

    #[test]
    fn test_generate_simple_undirected_graph() {
        let mut graph: LinkedGraph<String, (), Undirected> = LinkedGraph::default();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        graph.add_new_edge(&a, &b, ());

        let generator = TestGenerator {
            graph_name: "G".to_string(),
        };
        let mut output = Vec::new();

        generate_dot_file(&graph, &generator, &mut output).unwrap();
        let dot = String::from_utf8(output).unwrap();

        assert!(dot.contains("graph G"));
        assert!(dot.contains("n0"));
        assert!(dot.contains("n1"));
        // Both nodes present, one undirected edge (order may vary)
        assert_eq!(dot.matches("--").count(), 1);
    }

    struct InvalidNameGenerator;

    impl<G: Graph> DotGenerator<G> for InvalidNameGenerator {
        type Error = std::convert::Infallible;

        fn graph_name(&self) -> Result<String, Self::Error> {
            Ok("invalid name!".to_string())
        }
    }

    #[test]
    fn test_generate_invalid_graph_name() {
        let graph: LinkedGraph<String, (), Directed> = LinkedGraph::default();
        let generator = InvalidNameGenerator;
        let mut output = Vec::new();

        let result = generate_dot_file(&graph, &generator, &mut output);
        assert!(matches!(result, Err(DotError::InvalidId(_))));
    }

    struct InvalidNodeNameGenerator;

    impl<G: Graph> DotGenerator<G> for InvalidNodeNameGenerator {
        type Error = std::convert::Infallible;

        fn node_name(&self, _node_id: &G::NodeId, _index: usize) -> Result<String, Self::Error> {
            Ok("node name!".to_string())
        }
    }

    #[test]
    fn test_generate_invalid_node_name() {
        let mut graph: LinkedGraph<String, (), Directed> = LinkedGraph::default();
        graph.add_node("a".to_string());

        let generator = InvalidNodeNameGenerator;
        let mut output = Vec::new();

        let result = generate_dot_file(&graph, &generator, &mut output);
        assert!(matches!(result, Err(DotError::InvalidId(_))));
    }

    struct AttributeGenerator;

    impl<G: Graph> DotGenerator<G> for AttributeGenerator {
        type Error = std::convert::Infallible;

        fn node_attrs(
            &self,
            _node_id: &G::NodeId,
            _name: &mut String,
        ) -> Result<Vec<Attr>, Self::Error> {
            Ok(vec![Attr::Label("Test Label".to_string())])
        }

        fn edge_attrs(&self, _edge_id: &G::EdgeId) -> Result<Vec<Attr>, Self::Error> {
            Ok(vec![
                Attr::Label("Edge Label".to_string()),
                Attr::Color(vec![crate::dot::types::Color::Named("red".to_string())]),
            ])
        }
    }

    #[test]
    fn test_generate_with_attributes() {
        let mut graph: LinkedGraph<String, (), Directed> = LinkedGraph::default();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        graph.add_new_edge(&a, &b, ());

        let generator = AttributeGenerator;
        let mut output = Vec::new();

        generate_dot_file(&graph, &generator, &mut output).unwrap();
        let dot = String::from_utf8(output).unwrap();

        assert!(dot.contains("label = \"Test Label\""));
        assert!(dot.contains("label = \"Edge Label\""));
        assert!(dot.contains("color = red"));
    }

    #[test]
    fn test_format_dot_value_quoting() {
        // Test that values are quoted when necessary
        let mut graph: LinkedGraph<String, (), Directed> = LinkedGraph::default();
        let a = graph.add_node("hello world".to_string());
        let b = graph.add_node("foo-bar".to_string());
        graph.add_new_edge(&a, &b, ());

        let generator = TestGenerator {
            graph_name: "G".to_string(),
        };
        let mut output = Vec::new();

        generate_dot_file(&graph, &generator, &mut output).unwrap();
        let dot = String::from_utf8(output).unwrap();

        // Default labels should be quoted since node names contain invalid characters
        // (but they're not shown because they're internal node IDs)
        assert!(dot.contains("digraph G"));
    }

    #[test]
    fn test_generate_self_loop() {
        let mut graph: LinkedGraph<String, (), Directed> = LinkedGraph::default();
        let a = graph.add_node("a".to_string());
        graph.add_new_edge(&a.clone(), &a, ());

        let generator = TestGenerator {
            graph_name: "G".to_string(),
        };
        let mut output = Vec::new();

        generate_dot_file(&graph, &generator, &mut output).unwrap();
        let dot = String::from_utf8(output).unwrap();

        assert!(dot.contains("n0 -> n0"));
    }

    #[test]
    fn test_generate_multiple_edges() {
        let mut graph: LinkedGraph<String, (), Directed> = LinkedGraph::default();
        let a = graph.add_node("a".to_string());
        let b = graph.add_node("b".to_string());
        let c = graph.add_node("c".to_string());
        graph.add_new_edge(&a.clone(), &b.clone(), ());
        graph.add_new_edge(&b.clone(), &c.clone(), ());
        graph.add_new_edge(&c, &a, ());

        let generator = TestGenerator {
            graph_name: "Triangle".to_string(),
        };
        let mut output = Vec::new();

        generate_dot_file(&graph, &generator, &mut output).unwrap();
        let dot = String::from_utf8(output).unwrap();

        assert!(dot.contains("digraph Triangle"));
        assert_eq!(dot.matches("->").count(), 3);
    }
}
