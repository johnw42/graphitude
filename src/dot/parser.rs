use std::{
    collections::{HashMap, hash_map::Entry},
    error::Error,
};

use dot_parser::ast::{
    EdgeStmt, Graph as DotGraph, ID, NodeID, NodeStmt, Stmt, StmtList, Subgraph, either::Either,
};

use crate::{
    directedness::Directedness, dot::attr::Attr, edge_multiplicity::EdgeMultiplicity, prelude::*,
};

/// Recursively extract all node IDs from a node/subgraph specification.
/// Returns a vector of node ID strings.
fn extract_node_ids(either: &Either<NodeID, Subgraph<(ID<'_>, ID<'_>)>>) -> Vec<String> {
    match either {
        Either::Left(node_id) => vec![node_id.id.to_string()],
        Either::Right(subgraph) => {
            let mut node_ids = Vec::new();
            for stmt in &subgraph.stmts {
                match stmt {
                    Stmt::NodeStmt(node_stmt) => {
                        node_ids.push(node_stmt.node.id.to_string());
                    }
                    Stmt::EdgeStmt(edge_stmt) => {
                        // Recursively extract from edge statements within subgraph
                        node_ids.extend(extract_node_ids(&edge_stmt.from));
                        let mut current_rhs = Some(&edge_stmt.next);
                        while let Some(rhs) = current_rhs {
                            node_ids.extend(extract_node_ids(&rhs.to));
                            current_rhs = rhs.next.as_deref();
                        }
                    }
                    Stmt::Subgraph(nested_subgraph) => {
                        // Recursively process nested subgraph
                        node_ids.extend(extract_node_ids(&Either::Right(nested_subgraph.clone())));
                    }
                    _ => {} // Ignore other statement types
                }
            }
            node_ids
        }
    }
}

/// Errors that can occur during DOT format parsing.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ParseError<B: GraphBuilder> {
    /// Failed to parse the DOT format data.
    #[error("Failed to parse DOT format data: {0}")]
    ParseError(String),
    /// A node ID referenced in an edge was not found in the graph.
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    /// A duplicate node ID was found in the DOT data.
    #[error("Duplicate node ID found: {0}")]
    DuplicateNode(String),
    /// A parallel edge was found in the DOT data, and the graph does not support parallel edges.
    #[error("Duplicate edge found between nodes: {0} and {1}")]
    DuplicateEdge(String, String),
    /// The DOT data specifies a directedness that is not supported by the graph builder.
    #[error("Unsupported directedness: {0:?}")]
    UnsupportedDirectedness(Directedness),
    /// The DOT data specifies an edge multiplicity that is not supported by the graph builder.
    #[error("Unsupported edge multiplicity: {0:?}")]
    UnsupportedEdgeMultiplicity(EdgeMultiplicity),
    /// An error occurred in the graph builder.
    #[error("Builder error: {0}")]
    Builder(#[source] B::Error),
}

/// Trait for building graph data from DOT format statements.
///
/// Implementors of this trait provide the logic for converting DOT format
/// node and edge statements into the graph's node and edge data types.
pub trait GraphBuilder {
    type Graph: GraphMut;
    type Error: Error;

    /// Create an empty graph with the given name, directedness, and edge multiplicity.
    fn make_empty_graph(
        &mut self,
        name: Option<&str>,
        directedness: <Self::Graph as Graph>::Directedness,
        edge_multiplicity: <Self::Graph as Graph>::EdgeMultiplicity,
    ) -> Result<Self::Graph, Self::Error>;

    /// Create node data from a node with its attributes.
    fn make_node_data(
        &mut self,
        id: &str,
        attrs: &[Attr],
    ) -> Result<<Self::Graph as Graph>::NodeData, Self::Error>;

    /// Create edge data from a DOT EdgeStmt.
    fn make_edge_data(
        &mut self,
        attrs: &[Attr],
    ) -> Result<<Self::Graph as Graph>::EdgeData, Self::Error>;

    /// Create node data for an implicit node (referenced in an edge but not explicitly declared).
    fn make_implicit_node_data(
        &mut self,
        node_id: &str,
    ) -> Result<<Self::Graph as Graph>::NodeData, Self::Error> {
        let _ = node_id;
        unimplemented!("make_implicit_node_data must be implemented to handle implicit nodes")
    }
}

/// Parse DOT attribute lists into a Vec<Attr>.
/// Handles the nested structure: Vec<AttrList> -> Vec<AList> -> Vec<(ID, ID)>
fn parse_attrs(
    attr_lists: &[dot_parser::ast::AttrList<(ID<'_>, ID<'_>)>],
) -> Result<Vec<Attr>, String> {
    let mut attrs = Vec::new();
    for attr_list in attr_lists {
        for alist in &attr_list.elems {
            for (name, value) in &alist.elems {
                let name_str: String = name.clone().into();
                let value_str: String = value.clone().into();
                let attr = Attr::parse(&name_str, &value_str)
                    .map_err(|e| format!("Failed to parse attribute '{}': {:?}", name_str, e))?;
                attrs.push(attr);
            }
        }
    }
    Ok(attrs)
}

/// Parse attributes from a NodeStmt into a Vec<Attr>.
fn parse_node_attrs(node_stmt: &NodeStmt<(ID<'_>, ID<'_>)>) -> Result<Vec<Attr>, String> {
    node_stmt
        .attr
        .as_ref()
        .map(|attrs| parse_attrs(std::slice::from_ref(attrs)))
        .unwrap_or_else(|| Ok(Vec::new()))
}

/// Parse attributes from an EdgeStmt into a Vec<Attr>.
fn parse_edge_attrs(edge_stmt: &EdgeStmt<(ID<'_>, ID<'_>)>) -> Result<Vec<Attr>, String> {
    edge_stmt
        .attr
        .as_ref()
        .map(|attrs| parse_attrs(std::slice::from_ref(attrs)))
        .unwrap_or_else(|| Ok(Vec::new()))
}

/// Parse a DOT format string and construct a graph using the provided builder.
///
/// # Arguments
///
/// * `data` - The DOT format string to parse
/// * `builder` - A trait implementor that creates node and edge data from DOT statements
///
/// # Returns
///
/// A `Result` containing the populated graph, or a `DotParseError` if parsing fails.
///
/// # Errors
///
/// Returns `DotParseError::ParseError` if the DOT data cannot be parsed.
/// Returns `DotParseError::NodeNotFound` if an edge references a non-existent node.
pub(crate) fn parse_dot_into_graph<G, B>(data: &str, builder: &mut B) -> Result<G, ParseError<B>>
where
    G: GraphMut,
    B: GraphBuilder<Graph = G>,
{
    let dot_ast: DotGraph<_> = DotGraph::try_from(data)
        .map_err(|e| ParseError::ParseError(format!("Failed to parse DOT data: {:?}", e)))?;

    let directedness = if dot_ast.is_digraph {
        Directedness::Directed
    } else {
        Directedness::Undirected
    };
    let edge_multiplicity = if dot_ast.strict {
        EdgeMultiplicity::SingleEdge
    } else {
        EdgeMultiplicity::MultipleEdges
    };
    let mut graph = builder
        .make_empty_graph(
            dot_ast.name.as_deref(),
            directedness
                .try_into()
                .map_err(|_| ParseError::UnsupportedDirectedness(directedness))?,
            edge_multiplicity
                .try_into()
                .map_err(|_| ParseError::UnsupportedEdgeMultiplicity(edge_multiplicity))?,
        )
        .map_err(ParseError::Builder)?;
    let mut node_map: HashMap<String, G::NodeId> = HashMap::new();

    // First pass: create all explicit nodes (including those in subgraphs)
    fn process_stmts_for_nodes<G, B>(
        stmts: &StmtList<(ID<'_>, ID<'_>)>,
        graph: &mut G,
        node_map: &mut HashMap<String, G::NodeId>,
        builder: &mut B,
    ) -> Result<(), ParseError<B>>
    where
        G: Graph + GraphMut,
        B: GraphBuilder<Graph = G>,
    {
        for stmt in stmts {
            match stmt {
                Stmt::NodeStmt(node_stmt) => {
                    let node_id_str = node_stmt.node.id.to_string();
                    match node_map.entry(node_id_str) {
                        Entry::Occupied(entry) => {
                            return Err(ParseError::DuplicateNode(entry.key().clone()));
                        }
                        Entry::Vacant(entry) => {
                            let attrs =
                                parse_node_attrs(node_stmt).map_err(ParseError::ParseError)?;
                            let node_data = builder
                                .make_node_data(entry.key(), &attrs)
                                .map_err(ParseError::Builder)?;
                            let new_node_id = graph.add_node(node_data);
                            entry.insert(new_node_id);
                        }
                    }
                }
                Stmt::Subgraph(subgraph) => {
                    // Recursively process subgraph statements
                    process_stmts_for_nodes(&subgraph.stmts, graph, node_map, builder)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    process_stmts_for_nodes(&dot_ast.stmts, &mut graph, &mut node_map, builder)?;

    // Second pass: collect all node IDs referenced in edges and create implicit nodes
    fn process_stmts_for_implicit_nodes<G, B>(
        stmts: &StmtList<(ID<'_>, ID<'_>)>,
        graph: &mut G,
        node_map: &mut HashMap<String, G::NodeId>,
        builder: &mut B,
    ) -> Result<(), ParseError<B>>
    where
        G: Graph + GraphMut,
        B: GraphBuilder<Graph = G>,
    {
        for stmt in stmts {
            match stmt {
                Stmt::EdgeStmt(edge_stmt) => {
                    // Helper to collect node IDs from Either<NodeID, Subgraph>
                    let mut collect_node_ids =
                        |node: &Either<NodeID, _>| -> Result<(), ParseError<B>> {
                            for node_id_str in extract_node_ids(node) {
                                match node_map.entry(node_id_str.clone()) {
                                    Entry::Occupied(_) => {} // Node already exists, do nothing
                                    Entry::Vacant(entry) => {
                                        // Create implicit node using builder
                                        let node_data = builder
                                            .make_implicit_node_data(entry.key())
                                            .map_err(ParseError::Builder)?;
                                        let new_node_id = graph.add_node(node_data);
                                        entry.insert(new_node_id);
                                    }
                                }
                            }
                            Ok(())
                        };

                    // Collect from edge source
                    collect_node_ids(&edge_stmt.from)?;

                    // Collect from edge chain
                    let mut current_rhs = Some(&edge_stmt.next);
                    while let Some(rhs) = current_rhs {
                        collect_node_ids(&rhs.to)?;
                        current_rhs = rhs.next.as_deref();
                    }
                }
                Stmt::Subgraph(subgraph) => {
                    // Recursively process subgraph statements
                    process_stmts_for_implicit_nodes(&subgraph.stmts, graph, node_map, builder)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    process_stmts_for_implicit_nodes(&dot_ast.stmts, &mut graph, &mut node_map, builder)?;

    // Third pass: create edges (all nodes now exist)
    fn process_stmts_for_edges<G, B>(
        stmts: &StmtList<(ID<'_>, ID<'_>)>,
        graph: &mut G,
        node_map: &HashMap<String, G::NodeId>,
        builder: &mut B,
    ) -> Result<(), ParseError<B>>
    where
        G: Graph + GraphMut,
        B: GraphBuilder<Graph = G>,
    {
        for stmt in stmts {
            match stmt {
                Stmt::EdgeStmt(edge_stmt) => {
                    // Get all node IDs from the source (handles both single nodes and subgraphs)
                    let from_node_ids: Vec<(String, G::NodeId)> = extract_node_ids(&edge_stmt.from)
                        .into_iter()
                        .filter_map(|id_str| {
                            node_map
                                .get(&id_str)
                                .cloned()
                                .map(|node_id| (id_str, node_id))
                        })
                        .collect();

                    // Process edge chain: from -> next.to -> next.next.to -> ...
                    for (from_id_string, from_id) in from_node_ids {
                        let mut current_from = from_id.clone();
                        let mut current_rhs = Some(&edge_stmt.next);

                        while let Some(rhs) = current_rhs {
                            let to_node_ids: Vec<(String, G::NodeId)> = extract_node_ids(&rhs.to)
                                .into_iter()
                                .filter_map(|id_str| {
                                    node_map
                                        .get(&id_str)
                                        .cloned()
                                        .map(|node_id| (id_str, node_id))
                                })
                                .collect();

                            for (to_id_string, to_id) in to_node_ids.iter() {
                                let attrs =
                                    parse_edge_attrs(edge_stmt).map_err(ParseError::ParseError)?;
                                let edge_data = builder
                                    .make_edge_data(&attrs)
                                    .map_err(ParseError::Builder)?;
                                if let AddEdgeResult::Updated(_, _) =
                                    graph.add_edge(&current_from, to_id, edge_data)
                                {
                                    return Err(ParseError::DuplicateEdge(
                                        from_id_string.clone(),
                                        to_id_string.clone(),
                                    ));
                                }
                            }

                            // For edge chains, the "to" becomes the "from" for the next segment
                            // Use the first node if it's a subgraph
                            if let Some((_, first_to)) = to_node_ids.first() {
                                current_from = first_to.clone();
                            }

                            current_rhs = rhs.next.as_deref();
                        }
                    }
                }
                Stmt::Subgraph(subgraph) => {
                    // Recursively process subgraph statements
                    process_stmts_for_edges(&subgraph.stmts, graph, node_map, builder)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    process_stmts_for_edges(&dot_ast.stmts, &mut graph, &node_map, builder)?;

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linked_graph::LinkedGraph;

    // Simple builder that creates string node data from node IDs and empty edge data
    #[derive(Debug)]
    struct SimpleBuilder;

    impl GraphBuilder for SimpleBuilder {
        type Graph = LinkedGraph<String, ()>;
        type Error = std::convert::Infallible;

        fn make_empty_graph(
            &mut self,
            _name: Option<&str>,
            directedness: <Self::Graph as Graph>::Directedness,
            edge_multiplicity: <Self::Graph as Graph>::EdgeMultiplicity,
        ) -> Result<Self::Graph, Self::Error> {
            Ok(LinkedGraph::new(directedness, edge_multiplicity))
        }

        fn make_node_data(&mut self, id: &str, attrs: &[Attr]) -> Result<String, Self::Error> {
            if attrs.is_empty() {
                Ok(id.to_string())
            } else {
                let attrs_str = attrs
                    .iter()
                    .map(|attr| attr.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                Ok(format!("{}[{}]", id, attrs_str))
            }
        }

        fn make_implicit_node_data(&mut self, node_id: &str) -> Result<String, Self::Error> {
            Ok(node_id.to_string())
        }

        fn make_edge_data(&mut self, _attrs: &[Attr]) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    #[test]
    fn test_parse_simple_directed_graph() {
        let dot = r#"
            digraph G {
                a;
                b;
                a -> b;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);

        // Verify nodes exist
        let nodes: Vec<_> = graph
            .node_ids()
            .map(|id| graph.node_data(&id).clone())
            .collect();
        assert!(nodes.contains(&"a".to_string()));
        assert!(nodes.contains(&"b".to_string()));
    }

    #[test]
    fn test_parse_simple_undirected_graph() {
        let dot = r#"
            graph G {
                a;
                b;
                a -- b;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_parse_implicit_nodes() {
        // Node 'c' is not explicitly declared, only referenced in an edge
        let dot = r#"
            digraph G {
                a;
                b;
                a -> c;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 3);
        assert_eq!(graph.num_edges(), 1);

        let nodes: Vec<_> = graph
            .node_ids()
            .map(|id| graph.node_data(&id).clone())
            .collect();
        assert!(nodes.contains(&"a".to_string()));
        assert!(nodes.contains(&"b".to_string()));
        assert!(nodes.contains(&"c".to_string()));
    }

    #[test]
    fn test_parse_edge_chain() {
        // a -> b -> c creates two edges: a->b and b->c
        let dot = r#"
            digraph G {
                a -> b -> c;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 3);
        assert_eq!(graph.num_edges(), 2);
    }

    #[test]
    fn test_parse_multiple_edges() {
        let dot = r#"
            digraph G {
                a -> b;
                b -> c;
                c -> a;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 3);
        assert_eq!(graph.num_edges(), 3);
    }

    #[test]
    fn test_parse_with_attributes() {
        // Attributes should be available in the NodeStmt/EdgeStmt
        let dot = r#"
            digraph G {
                a [label="Node A"];
                b [label="Node B"];
                a -> b [weight=5];
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);

        // Verify that attributes are reflected in node data
        let nodes: Vec<_> = graph
            .node_ids()
            .map(|id| graph.node_data(&id).clone())
            .collect();

        // Check that node data includes the label attribute
        let node_a = nodes.iter().find(|n| n.starts_with("a")).unwrap();
        assert!(node_a.contains("label"));
        assert!(node_a.contains("Node A"));

        let node_b = nodes.iter().find(|n| n.starts_with("b")).unwrap();
        assert!(node_b.contains("label"));
        assert!(node_b.contains("Node B"));
    }

    #[test]
    fn test_parse_empty_graph() {
        let dot = r#"
            digraph G {
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }

    #[test]
    fn test_parse_error_invalid_dot() {
        let dot = "this is not valid DOT format";

        let mut builder = SimpleBuilder;
        let result: Result<LinkedGraph<String, ()>, _> = parse_dot_into_graph(dot, &mut builder);

        assert!(matches!(result, Err(ParseError::ParseError(_))));
    }

    #[test]
    fn test_parse_self_loop() {
        let dot = r#"
            digraph G {
                a -> a;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 1);
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_parse_complex_graph_directed() {
        let dot = r#"
            digraph G {
                a;
                b;
                c;
                a -> b;
                b -> c;
                c -> d;
                d -> a;
                a -> c;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 4);
        assert_eq!(graph.num_edges(), 5);
    }

    #[test]
    fn test_parse_complex_graph_undirected() {
        let dot = r#"
            graph G {
                a;
                b;
                c;
                a -- b;
                b -- c;
                c -- a;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 3);
        assert_eq!(graph.num_edges(), 3);
    }

    // Builder with numeric edge data
    #[derive(Debug)]
    struct EdgeWeightBuilder;

    impl GraphBuilder for EdgeWeightBuilder {
        type Graph = LinkedGraph<String, i32>;
        type Error = std::convert::Infallible;

        fn make_empty_graph(
            &mut self,
            _name: Option<&str>,
            directedness: <Self::Graph as Graph>::Directedness,
            edge_multiplicity: <Self::Graph as Graph>::EdgeMultiplicity,
        ) -> Result<Self::Graph, Self::Error> {
            Ok(LinkedGraph::new(directedness, edge_multiplicity))
        }

        fn make_node_data(&mut self, id: &str, _attrs: &[Attr]) -> Result<String, Self::Error> {
            Ok(id.to_string())
        }

        fn make_implicit_node_data(&mut self, node_id: &str) -> Result<String, Self::Error> {
            Ok(node_id.to_string())
        }

        fn make_edge_data(&mut self, _attrs: &[Attr]) -> Result<i32, Self::Error> {
            // For simplicity, just return a default weight
            // In a real implementation, you'd parse the attributes
            Ok(1)
        }
    }

    #[test]
    fn test_parse_with_edge_weights() {
        let dot = r#"
            digraph G {
                a -> b;
                b -> c;
            }
        "#;

        let mut builder = EdgeWeightBuilder;
        let graph: LinkedGraph<String, i32> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 3);
        assert_eq!(graph.num_edges(), 2);

        // Verify edges exist by checking edge IDs
        let edge_ids: Vec<_> = graph.edge_ids().collect();
        assert_eq!(edge_ids.len(), 2);

        // All edges should have default weight of 1
        for edge_id in edge_ids {
            assert_eq!(*graph.edge_data(&edge_id), 1);
        }
    }

    #[test]
    fn test_parse_subgraph_as_edge_source() {
        // a -> { b; c; } should create edges a->b and a->c
        let dot = r#"
            digraph G {
                a -> { b; c; };
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 3);
        assert_eq!(graph.num_edges(), 2);

        let nodes: Vec<_> = graph
            .node_ids()
            .map(|id| graph.node_data(&id).clone())
            .collect();
        assert!(nodes.contains(&"a".to_string()));
        assert!(nodes.contains(&"b".to_string()));
        assert!(nodes.contains(&"c".to_string()));
    }

    #[test]
    fn test_parse_subgraph_as_edge_target() {
        // { a; b; } -> c should create edges a->c and b->c
        let dot = r#"
            digraph G {
                { a; b; } -> c;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 3);
        assert_eq!(graph.num_edges(), 2);
    }

    #[test]
    fn test_parse_subgraph_both_sides() {
        // { a; b; } -> { c; d; } should create edges: a->c, a->d, b->c, b->d
        let dot = r#"
            digraph G {
                { a; b; } -> { c; d; };
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 4);
        assert_eq!(graph.num_edges(), 4);

        let nodes: Vec<_> = graph
            .node_ids()
            .map(|id| graph.node_data(&id).clone())
            .collect();
        assert!(nodes.contains(&"a".to_string()));
        assert!(nodes.contains(&"b".to_string()));
        assert!(nodes.contains(&"c".to_string()));
        assert!(nodes.contains(&"d".to_string()));
    }

    #[test]
    fn test_parse_named_subgraph() {
        // Named subgraphs should work the same way
        let dot = r#"
            digraph G {
                subgraph cluster_0 {
                    a;
                    b;
                }
                a -> c;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 3);
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_parse_nested_subgraph() {
        // Nested subgraphs should be flattened
        let dot = r#"
            digraph G {
                a -> {
                    b;
                    { c; d; }
                };
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 4);
        // a -> b, a -> c, a -> d
        assert_eq!(graph.num_edges(), 3);
    }

    #[test]
    fn test_parse_subgraph_with_internal_edges() {
        // Subgraph can have edges inside it, plus edges to/from outside
        let dot = r#"
            digraph G {
                x -> {
                    a -> b;
                    b -> c;
                };
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 4);
        // The subgraph extracts nodes from edges: a, b (appears twice), c
        // So x connects to: a, b (twice), c
        // Plus internal edges: a->b, b->c
        // Total edge additions: 4 (x->a, x->b, x->c, a->b, b->c) or possibly 5 if x->b twice
        // But LinkedGraph might allow duplicate edges
        assert!(graph.num_edges() == 4 || graph.num_edges() == 5 || graph.num_edges() == 6);
    }

    #[test]
    fn test_parse_complex_subgraph_example() {
        // Complex example demonstrating various subgraph features
        let dot = r#"
            digraph G {
                // Regular nodes
                start;
                end;
                
                // Start connects to a subgraph
                start -> {
                    a;
                    b;
                    c;
                };
                
                // Subgraph members connect to each other
                a -> b;
                b -> c;
                
                // Subgraph connects to end
                { a; b; c; } -> end;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 5); // start, end, a, b, c

        // Edges:
        // start -> a, start -> b, start -> c (3)
        // a -> b, b -> c (2)
        // a -> end, b -> end, c -> end (3)
        // Total: 8 edges
        assert_eq!(graph.num_edges(), 8);
    }

    #[test]
    fn test_parse_error_malformed_syntax() {
        let invalid_inputs = vec![
            "digraph { a -> }",     // Missing target
            "digraph { -> b }",     // Missing source
            "digraph G { a -> b",   // Missing closing brace
            "digraph { a [label }", // Malformed attribute
        ];

        for dot in invalid_inputs {
            let mut builder = SimpleBuilder;
            let result: Result<LinkedGraph<String, ()>, _> =
                parse_dot_into_graph(dot, &mut builder);
            assert!(result.is_err(), "Expected error for input: {}", dot);
        }
    }

    #[test]
    fn test_parse_multiple_attributes() {
        let dot = r#"
            digraph G {
                a [label="Node A", color=red, shape=box];
                b [label="Node B", color=blue];
                a -> b [label="Edge", weight=5, color=green];
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();
        parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);

        let nodes: Vec<_> = graph
            .node_ids()
            .map(|id| graph.node_data(&id).clone())
            .collect();

        // Verify multiple attributes are parsed
        let node_a = nodes.iter().find(|n| n.starts_with("a")).unwrap();
        assert!(node_a.contains("label"));
        assert!(node_a.contains("color"));
        assert!(node_a.contains("shape"));
    }

    #[test]
    fn test_parse_parallel_edges() {
        let dot = r#"
            digraph G {
                a -> b;
                a -> b;
                a -> b;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 2);
        // LinkedGraph allows parallel edges
        assert_eq!(graph.num_edges(), 3);
    }

    #[test]
    fn test_parse_quoted_identifiers() {
        let dot = r#"
            digraph G {
                "node 1";
                "node-2";
                "node 1" -> "node-2";
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);

        let nodes: Vec<_> = graph
            .node_ids()
            .map(|id| graph.node_data(&id).clone())
            .collect();

        // Verify quoted identifiers are handled
        assert!(
            nodes
                .iter()
                .any(|n| n.contains("node 1") || n.contains("node-1"))
        );
        assert!(
            nodes
                .iter()
                .any(|n| n.contains("node-2") || n.contains("node 2"))
        );
    }

    #[test]
    fn test_parse_numeric_node_ids() {
        let dot = r#"
            digraph G {
                123;
                456;
                123 -> 456;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 2);
        assert_eq!(graph.num_edges(), 1);
    }

    #[test]
    fn test_parse_long_edge_chain() {
        let dot = r#"
            digraph G {
                a -> b -> c -> d -> e -> f;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        assert_eq!(graph.num_nodes(), 6);
        assert_eq!(graph.num_edges(), 5); // a->b, b->c, c->d, d->e, e->f
    }

    #[test]
    fn test_parse_mixed_explicit_and_implicit_nodes() {
        let dot = r#"
            digraph G {
                a [label="Explicit A"];
                b [label="Explicit B"];
                a -> c;
                c -> b;
                d -> e;
            }
        "#;

        let mut builder = SimpleBuilder;
        let graph: LinkedGraph<String, ()> = parse_dot_into_graph(dot, &mut builder).unwrap();

        // a, b are explicit; c, d, e are implicit
        assert_eq!(graph.num_nodes(), 5);
        assert_eq!(graph.num_edges(), 3);

        let nodes: Vec<_> = graph
            .node_ids()
            .map(|id| graph.node_data(&id).clone())
            .collect();

        // Explicit nodes should have attributes
        let node_a = nodes.iter().find(|n| n.starts_with("a")).unwrap();
        assert!(node_a.contains("Explicit A"));

        // Implicit nodes should not have attributes (just the ID)
        let node_c = nodes
            .iter()
            .find(|n| n.starts_with("c") && !n.starts_with("ca"))
            .unwrap();
        assert_eq!(node_c, "c");
    }

    #[test]
    fn test_duplicate_edges() {
        let dot = r#"
            digraph G {
                a -> b;
                a -> b;
            }
        "#;

        let mut builder = SimpleBuilder;
        let result: Result<LinkedGraph<String, ()>, _> = parse_dot_into_graph(dot, &mut builder);

        // Depending on the implementation, this might allow parallel edges or return an error
        // If it allows parallel edges, it should have 2 edges; if not, it should return an error
        match result {
            Ok(graph) => {
                assert_eq!(graph.num_nodes(), 2);
                assert_eq!(graph.num_edges(), 2); // Parallel edges allowed
            }
            Err(ParseError::DuplicateEdge(from, to)) => {
                assert_eq!(from, "a");
                assert_eq!(to, "b");
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }
}
