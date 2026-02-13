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
    use graphitude::directedness::Directedness;
    use graphitude::edge_multiplicity::EdgeMultiplicity;
    use graphitude::{
        dot::{attr::Attr, parser::GraphBuilder},
        linked_graph::LinkedGraph,
        prelude::*,
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

        let mut builder = AttributeBuilder;

        let graph = parse_or_exit(&input, &mut builder);
        print_summary_with_attrs(&graph, args.sample_nodes);
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

    fn parse_or_exit<B>(data: &str, builder: &mut B) -> LinkedGraph<NodeInfo, EdgeInfo>
    where
        B: GraphBuilder<Graph = LinkedGraph<NodeInfo, EdgeInfo>>,
    {
        match LinkedGraph::from_dot_string(data, builder) {
            Ok(graph) => graph,
            Err(err) => {
                eprintln!("Invalid DOT input: {err}");
                process::exit(1);
            }
        }
    }

    fn print_summary_with_attrs<G: Graph<NodeData = NodeInfo, EdgeData = EdgeInfo>>(
        graph: &G,
        sample_nodes: usize,
    ) {
        println!("DOT file parsed successfully.");
        println!(
            "Graph kind: {}",
            if graph.is_directed() {
                "directed (digraph)"
            } else {
                "undirected (graph)"
            }
        );
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
        attrs: Vec<(String, String)>,
    }

    impl std::fmt::Display for NodeInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.id)?;
            if !self.attrs.is_empty() {
                write!(f, " [")?;
                for (i, (k, v)) in self.attrs.iter().enumerate() {
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
        attrs: Vec<(String, String)>,
    }

    impl std::fmt::Display for EdgeInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.attrs.is_empty() {
                write!(f, "(no attributes)")
            } else {
                let parts: Vec<String> = self
                    .attrs
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                write!(f, "{}", parts.join(", "))
            }
        }
    }

    #[derive(Debug, Default)]
    struct AttributeBuilder;

    impl GraphBuilder for AttributeBuilder {
        type Graph = LinkedGraph<NodeInfo, EdgeInfo>;
        type Error = std::convert::Infallible;

        fn make_empty_graph(
            &mut self,
            _name: Option<&str>,
            directedness: Directedness,
            edge_multiplicity: EdgeMultiplicity,
        ) -> Result<Self::Graph, Self::Error> {
            Ok(LinkedGraph::new(directedness, edge_multiplicity))
        }

        fn make_node_data(&mut self, id: &str, attrs: &[Attr]) -> Result<NodeInfo, Self::Error> {
            let node_attrs: Vec<(String, String)> = attrs
                .iter()
                .map(|attr| (attr.name().to_string(), attr.value()))
                .collect();

            Ok(NodeInfo {
                id: id.to_string(),
                attrs: node_attrs,
            })
        }

        fn make_edge_data(&mut self, attrs: &[Attr]) -> Result<EdgeInfo, Self::Error> {
            let edge_attrs: Vec<(String, String)> = attrs
                .iter()
                .map(|attr| (attr.name().to_string(), attr.value()))
                .collect();

            Ok(EdgeInfo { attrs: edge_attrs })
        }

        fn make_implicit_node_data(&mut self, node_id: &str) -> Result<NodeInfo, Self::Error> {
            Ok(NodeInfo {
                id: node_id.to_string(),
                attrs: Vec::new(),
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
