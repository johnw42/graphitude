#![cfg(feature = "dot")]

use std::collections::{HashMap, HashSet};

use graphitude::{
    dot::{
        attr::Attr,
        parser::{GraphBuilder, parse_dot_into_graph},
        renderer::{DotGenerator, generate_dot_file},
    },
    linked_graph::LinkedGraph,
    prelude::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct NodeData {
    id: String,
    attrs: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EdgeData {
    attrs: Vec<(String, String)>,
}

#[derive(Debug)]
struct TestBuilder;

#[derive(Debug)]
struct TestError(String);

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

impl GraphBuilder for TestBuilder {
    type NodeData = NodeData;
    type EdgeData = EdgeData;
    type Error = TestError;

    fn make_node_data(&mut self, id: &str, attrs: &[Attr]) -> Result<Self::NodeData, Self::Error> {
        let attr_vec = attrs
            .iter()
            .map(|attr| (attr.name().to_string(), attr.value()))
            .collect();

        Ok(NodeData {
            id: id.to_string(),
            attrs: attr_vec,
        })
    }

    fn make_edge_data(&mut self, attrs: &[Attr]) -> Result<Self::EdgeData, Self::Error> {
        let attr_vec = attrs
            .iter()
            .map(|attr| (attr.name().to_string(), attr.value()))
            .collect();

        Ok(EdgeData { attrs: attr_vec })
    }

    fn make_implicit_node_data(&mut self, node_id: &str) -> Result<Self::NodeData, Self::Error> {
        Ok(NodeData {
            id: node_id.to_string(),
            attrs: Vec::new(),
        })
    }
}

struct TestDotGenerator<'a, G: Graph> {
    graph: &'a G,
}

impl<'a, G> DotGenerator<G> for TestDotGenerator<'a, G>
where
    G: Graph<NodeData = NodeData, EdgeData = EdgeData>,
{
    type Error = TestError;

    fn graph_name(&self) -> Result<String, Self::Error> {
        Ok("TestGraph".to_string())
    }

    fn node_name(&self, node_id: &G::NodeId, _index: usize) -> Result<String, Self::Error> {
        let data = self.graph.node_data(node_id);
        Ok(data.id.clone())
    }

    fn node_attrs(
        &self,
        node_id: &G::NodeId,
        _name: &mut String,
    ) -> Result<Vec<Attr>, Self::Error> {
        let data = self.graph.node_data(node_id);
        let mut attrs = Vec::new();
        for (k, v) in &data.attrs {
            if let Ok(attr) = Attr::parse(k, v) {
                attrs.push(attr);
            }
        }
        Ok(attrs)
    }

    fn edge_attrs(&self, edge_id: &G::EdgeId) -> Result<Vec<Attr>, Self::Error> {
        let data = self.graph.edge_data(edge_id);
        let mut attrs = Vec::new();
        for (k, v) in &data.attrs {
            if let Ok(attr) = Attr::parse(k, v) {
                attrs.push(attr);
            }
        }
        Ok(attrs)
    }
}

fn normalize_graph_structure<G>(
    graph: &G,
) -> (
    HashSet<String>,
    HashSet<(String, String)>,
    HashMap<String, HashSet<(String, String)>>,
    HashMap<(String, String), HashSet<(String, String)>>,
)
where
    G: Graph<NodeData = NodeData, EdgeData = EdgeData>,
{
    let nodes: HashSet<String> = graph
        .node_ids()
        .map(|id| graph.node_data(&id).id.clone())
        .collect();

    let edges: HashSet<(String, String)> = graph
        .edge_ids()
        .map(|eid| {
            let src = eid.source();
            let dst = eid.target();
            let mut src_id = graph.node_data(&src).id.clone();
            let mut dst_id = graph.node_data(&dst).id.clone();
            // Normalize undirected edges by sorting node IDs
            if src_id > dst_id {
                std::mem::swap(&mut src_id, &mut dst_id);
            }
            (src_id, dst_id)
        })
        .collect();

    let node_attrs: HashMap<String, HashSet<(String, String)>> = graph
        .node_ids()
        .map(|id| {
            let data = graph.node_data(&id);
            let attrs: HashSet<_> = data.attrs.iter().cloned().collect();
            (data.id.clone(), attrs)
        })
        .collect();

    let edge_attrs: HashMap<(String, String), HashSet<(String, String)>> = graph
        .edge_ids()
        .map(|eid| {
            let src = eid.source();
            let dst = eid.target();
            let mut src_id = graph.node_data(&src).id.clone();
            let mut dst_id = graph.node_data(&dst).id.clone();
            // Normalize undirected edges by sorting node IDs
            if src_id > dst_id {
                std::mem::swap(&mut src_id, &mut dst_id);
            }
            let data = graph.edge_data(&eid);
            (
                (src_id, dst_id),
                data.attrs.iter().cloned().collect::<HashSet<_>>(),
            )
        })
        .collect();

    (nodes, edges, node_attrs, edge_attrs)
}

#[test]
fn test_directed_graph_roundtrip() {
    let input_dot = r#"
        digraph TestGraph {
            a [label="Node A", color=red];
            b [label="Node B"];
            c;
            a -> b [weight=5];
            b -> c;
        }
    "#;

    // Parse DOT into graph
    let mut builder = TestBuilder;
    let graph1: LinkedGraph<NodeData, EdgeData, Directed> =
        parse_dot_into_graph(input_dot, &mut builder).expect("Failed to parse DOT");

    // Generate DOT from graph
    let generator = TestDotGenerator { graph: &graph1 };
    let mut output = Vec::new();
    generate_dot_file(&graph1, &generator, &mut output).expect("Failed to generate DOT");
    let generated_dot = String::from_utf8(output).expect("Invalid UTF-8");

    // Parse generated DOT back into graph
    let mut builder2 = TestBuilder;
    let graph2: LinkedGraph<NodeData, EdgeData, Directed> =
        parse_dot_into_graph(&generated_dot, &mut builder2).expect("Failed to parse generated DOT");

    // Compare graph structures (ignoring ordering)
    let (nodes1, edges1, node_attrs1, edge_attrs1) = normalize_graph_structure(&graph1);
    let (nodes2, edges2, node_attrs2, edge_attrs2) = normalize_graph_structure(&graph2);

    assert_eq!(nodes1, nodes2, "Node sets should match");
    assert_eq!(edges1, edges2, "Edge sets should match");
    assert_eq!(node_attrs1, node_attrs2, "Node attributes should match");
    assert_eq!(edge_attrs1, edge_attrs2, "Edge attributes should match");
}

#[test]
fn test_undirected_graph_roundtrip() {
    let input_dot = r#"
        graph TestGraph {
            x [shape=box];
            y [shape=circle];
            z;
            x -- y;
            y -- z [color=blue];
        }
    "#;

    // Parse DOT into graph
    let mut builder = TestBuilder;
    let graph1: LinkedGraph<NodeData, EdgeData, Undirected> =
        parse_dot_into_graph(input_dot, &mut builder).expect("Failed to parse DOT");

    // Generate DOT from graph
    let generator = TestDotGenerator { graph: &graph1 };
    let mut output = Vec::new();
    generate_dot_file(&graph1, &generator, &mut output).expect("Failed to generate DOT");
    let generated_dot = String::from_utf8(output).expect("Invalid UTF-8");

    // Parse generated DOT back into graph
    let mut builder2 = TestBuilder;
    let graph2: LinkedGraph<NodeData, EdgeData, Undirected> =
        parse_dot_into_graph(&generated_dot, &mut builder2).expect("Failed to parse generated DOT");

    // Compare graph structures (ignoring ordering)
    let (nodes1, edges1, node_attrs1, edge_attrs1) = normalize_graph_structure(&graph1);
    let (nodes2, edges2, node_attrs2, edge_attrs2) = normalize_graph_structure(&graph2);

    assert_eq!(nodes1, nodes2, "Node sets should match");
    assert_eq!(edges1, edges2, "Edge sets should match");
    assert_eq!(node_attrs1, node_attrs2, "Node attributes should match");
    assert_eq!(edge_attrs1, edge_attrs2, "Edge attributes should match");
}

#[test]
fn test_empty_graph_roundtrip() {
    let input_dot = r#"digraph Empty {}"#;

    let mut builder = TestBuilder;
    let graph1: LinkedGraph<NodeData, EdgeData, Directed> =
        parse_dot_into_graph(input_dot, &mut builder).expect("Failed to parse DOT");

    let generator = TestDotGenerator { graph: &graph1 };
    let mut output = Vec::new();
    generate_dot_file(&graph1, &generator, &mut output).expect("Failed to generate DOT");
    let generated_dot = String::from_utf8(output).expect("Invalid UTF-8");

    let mut builder2 = TestBuilder;
    let graph2: LinkedGraph<NodeData, EdgeData, Directed> =
        parse_dot_into_graph(&generated_dot, &mut builder2).expect("Failed to parse generated DOT");

    assert_eq!(graph1.num_nodes(), graph2.num_nodes());
    assert_eq!(graph1.num_edges(), graph2.num_edges());
    assert_eq!(graph1.num_nodes(), 0);
    assert_eq!(graph1.num_edges(), 0);
}
