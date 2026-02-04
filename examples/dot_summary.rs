//! Example that reads a DOT file (or stdin), validates it, and prints a summary.
//!
//! Usage:
//!   cargo run --example dot_summary --features dot -- path/to/graph.dot
//!   cat path/to/graph.dot | cargo run --example dot_summary --features dot
//!   cargo run --example dot_summary --features dot -- -

#[cfg(feature = "dot")]
mod inner {
    use std::fs;
    use std::io::{self, Read};
    use std::process;

    use clap::Parser;
    use graphitude::{
        EdgeId, Graph, GraphMut,
        directedness::{Directed, Undirected},
        dot::{
            attr::Attr,
            parser::{GraphBuilder, parse_dot_into_graph},
        },
        linked_graph::LinkedGraph,
    };

    /// Read a DOT file or stdin, validate it, and print a summary.
    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    struct Args {
        /// Input DOT file path (use '-' or omit to read from stdin)
        input: Option<String>,

        /// Max number of node IDs to print in the summary
        #[arg(long, default_value_t = 10)]
        sample_nodes: usize,
    }

    pub fn run() {
        let args = Args::parse();

        let input = match args.input.as_deref() {
            Some("-") | None => read_stdin_or_exit(),
            Some(path) => read_file_or_exit(path),
        };

        let directed = match detect_directedness(&input) {
            Some(value) => value,
            None => {
                eprintln!("Could not determine graph type. Expected 'digraph' or 'graph' keyword.");
                process::exit(2);
            }
        };

        let mut builder = AttributeBuilder::default();

        if directed {
            let graph: LinkedGraph<NodeInfo, EdgeInfo, Directed> =
                parse_or_exit(&input, &mut builder, "directed");
            print_summary_with_attrs("directed (digraph)", &graph, args.sample_nodes);
        } else {
            let graph: LinkedGraph<NodeInfo, EdgeInfo, Undirected> =
                parse_or_exit(&input, &mut builder, "undirected");
            print_summary_with_attrs("undirected (graph)", &graph, args.sample_nodes);
        }
    }

    fn read_stdin_or_exit() -> String {
        let mut buffer = String::new();
        if let Err(err) = io::stdin().read_to_string(&mut buffer) {
            eprintln!("Failed to read stdin: {err}");
            process::exit(1);
        }
        buffer
    }

    fn read_file_or_exit(path: &str) -> String {
        match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("Failed to read '{path}': {err}");
                process::exit(1);
            }
        }
    }

    fn detect_directedness(data: &str) -> Option<bool> {
        for raw in data.split_whitespace() {
            let token = raw.trim_matches(|c: char| !c.is_ascii_alphanumeric());
            if token.eq_ignore_ascii_case("digraph") {
                return Some(true);
            }
            if token.eq_ignore_ascii_case("graph") {
                return Some(false);
            }
        }
        None
    }

    fn parse_or_exit<G, B>(data: &str, builder: &mut B, label: &str) -> G
    where
        G: Graph + GraphMut,
        B: GraphBuilder<NodeData = G::NodeData, EdgeData = G::EdgeData>,
    {
        match parse_dot_into_graph(data, builder) {
            Ok(graph) => graph,
            Err(err) => {
                eprintln!("Invalid {label} DOT input: {err}");
                process::exit(1);
            }
        }
    }

    fn print_summary_with_attrs<G: Graph<NodeData = NodeInfo, EdgeData = EdgeInfo>>(
        kind: &str,
        graph: &G,
        sample_nodes: usize,
    ) {
        println!("DOT file parsed successfully.");
        println!("Graph kind: {kind}");
        println!("Nodes: {}", graph.num_nodes());
        println!("Edges: {}", graph.num_edges());

        if sample_nodes == 0 || graph.num_nodes() == 0 {
            return;
        }

        let nodes: Vec<String> = graph
            .node_ids()
            .take(sample_nodes)
            .map(|id| {
                let data = graph.node_data(&id).clone();
                format!("{}", data)
            })
            .collect();

        println!("\nSample nodes (up to {sample_nodes}):");
        for node in nodes {
            println!("  {node}");
        }

        // Show some edge information
        let edge_count = graph.num_edges().min(sample_nodes);
        if edge_count > 0 {
            println!("\nSample edges (up to {sample_nodes}):");
            for edge_id in graph.edge_ids().take(edge_count) {
                let src = edge_id.source();
                let dst = edge_id.target();
                let src_id = &graph.node_data(&src).id;
                let dst_id = &graph.node_data(&dst).id;
                let edge_data = graph.edge_data(&edge_id).clone();
                println!("  {} -> {}: {}", src_id, dst_id, edge_data);
            }
        }
    }

    #[derive(Debug, Default, Clone)]
    struct NodeInfo {
        id: String,
        label: Option<String>,
        color: Option<String>,
        other_attrs: Vec<(String, String)>,
    }

    impl std::fmt::Display for NodeInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.id)?;
            if let Some(label) = &self.label {
                write!(f, " [label=\"{}\"]", label)?;
            }
            if let Some(color) = &self.color {
                write!(f, " [color={}]", color)?;
            }
            if !self.other_attrs.is_empty() {
                write!(f, " [")?;
                for (i, (k, v)) in self.other_attrs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}={}", k, v)?;
                }
                write!(f, "]")?;
            }
            Ok(())
        }
    }

    #[derive(Debug, Default, Clone)]
    struct EdgeInfo {
        label: Option<String>,
        color: Option<String>,
        weight: Option<String>,
        other_attrs: Vec<(String, String)>,
    }

    impl std::fmt::Display for EdgeInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut parts = Vec::new();
            if let Some(label) = &self.label {
                parts.push(format!("label=\"{}\"", label));
            }
            if let Some(color) = &self.color {
                parts.push(format!("color={}", color));
            }
            if let Some(weight) = &self.weight {
                parts.push(format!("weight={}", weight));
            }
            for (k, v) in &self.other_attrs {
                parts.push(format!("{}={}", k, v));
            }
            if parts.is_empty() {
                write!(f, "(no attributes)")
            } else {
                write!(f, "{}", parts.join(", "))
            }
        }
    }

    #[derive(Debug, Default)]
    struct AttributeBuilder;

    impl GraphBuilder for AttributeBuilder {
        type NodeData = NodeInfo;
        type EdgeData = EdgeInfo;
        type Error = std::convert::Infallible;

        fn make_node_data(
            &mut self,
            id: &str,
            attrs: &[Attr],
        ) -> Result<Self::NodeData, Self::Error> {
            let mut node = NodeInfo {
                id: id.to_string(),
                label: None,
                color: None,
                other_attrs: Vec::new(),
            };

            for attr in attrs {
                let key = attr.name().to_string();
                let value = attr.value();

                match key.as_str() {
                    "label" => node.label = Some(value),
                    "color" => node.color = Some(value),
                    _ => node.other_attrs.push((key, value)),
                }
            }

            Ok(node)
        }

        fn make_edge_data(&mut self, attrs: &[Attr]) -> Result<Self::EdgeData, Self::Error> {
            let mut edge = EdgeInfo::default();

            for attr in attrs {
                let key = attr.name().to_string();
                let value = attr.value();

                match key.as_str() {
                    "label" => edge.label = Some(value),
                    "color" => edge.color = Some(value),
                    "weight" => edge.weight = Some(value),
                    _ => edge.other_attrs.push((key, value)),
                }
            }

            Ok(edge)
        }

        fn make_implicit_node_data(
            &mut self,
            node_id: &str,
        ) -> Result<Self::NodeData, Self::Error> {
            Ok(NodeInfo {
                id: node_id.to_string(),
                label: None,
                color: None,
                other_attrs: Vec::new(),
            })
        }
    }
}

#[cfg(feature = "dot")]
fn main() {
    inner::run();
}

#[cfg(not(feature = "dot"))]
fn main() {
    println!("This example requires the 'dot' feature to be enabled.");
    println!("Run with: cargo run --example dot_summary --features dot");
}
