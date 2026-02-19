//! Example that generates a large graph and exports it to a DOT file.
//!
//! This example demonstrates how to:
//! - Use `generate_large_graph_with` to create a graph with custom data types
//! - Implement a DotGenerator to customize DOT output
//! - Write the graph to a .dot file for visualization with Graphviz
//! - Parse command-line arguments with clap
//! - Write to stdout when no output file is specified
//! - Support different node and edge data types (i32, String, ())

#[cfg(feature = "dot")]
mod inner {
    use std::fs::File;
    use std::io::{self, Write};

    use clap::{Parser, ValueEnum};
    use graphitude::directedness::Directedness;
    use graphitude::edge_multiplicity::EdgeMultiplicity;
    use graphitude::{
        dot::renderer::DotGenerator, linked_graph::LinkedGraph, prelude::*,
        graph_tests::generate_large_graph_with,
    };

    /// Data type selector for CLI
    #[derive(Debug, Clone, Copy, ValueEnum)]
    enum DataType {
        /// 32-bit integer
        I32,
        /// String
        String,
        /// No data (unit type)
        None,
    }

    /// Graph kind selector for CLI
    #[derive(Debug, Clone, Copy, ValueEnum)]
    enum GraphKind {
        Directed,
        Undirected,
    }

    /// Graph implementation selector for CLI
    #[derive(Debug, Clone, Copy, ValueEnum)]
    enum GraphImpl {
        Linked,
        Adjacency,
    }

    /// Actual data stored in graph nodes and edges
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    enum Data {
        /// 32-bit integer value
        I32(i32),
        /// String value
        String(String),
        /// No data
        None,
    }

    impl std::fmt::Display for Data {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Data::I32(v) => write!(f, "{}", v),
                Data::String(s) => write!(f, "{}", s),
                Data::None => write!(f, ""),
            }
        }
    }

    /// Generate a large graph and export it to a DOT file for visualization with Graphviz
    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    struct Args {
        /// Output file path for the DOT file (writes to stdout if not provided)
        #[arg(short, long)]
        output: Option<String>,

        /// Name of the graph in the DOT file
        #[arg(short, long, default_value = "LargeGraph")]
        graph_name: String,

        /// Graph kind (directed or undirected)
        #[arg(long, value_enum, default_value = "directed")]
        graph_kind: GraphKind,

        /// Graph strictness
        #[arg(long, value_enum, default_value = "false")]
        strict: bool,

        /// Node data type
        #[arg(long, value_enum, default_value = "i32")]
        node_type: DataType,

        /// Edge data type
        #[arg(long, value_enum, default_value = "string")]
        edge_type: DataType,

        /// Prefix for edge labels (only used when edge-type is string)
        #[arg(long, default_value = "e")]
        edge_prefix: String,
    }

    /// DOT generator that uses a configurable graph name and has access to the graph.
    struct ConfigurableGenerator<'a, G> {
        graph_name: String,
        graph: &'a G,
    }

    impl<'a, G> DotGenerator<G> for ConfigurableGenerator<'a, G>
    where
        G: Graph,
        G::EdgeData: std::fmt::Display,
    {
        type Error = std::convert::Infallible;

        fn graph_name(&self) -> Result<String, Self::Error> {
            Ok(self.graph_name.clone())
        }

        fn edge_attrs(
            &self,
            edge_id: &<G as Graph>::EdgeId,
        ) -> Result<Vec<graphitude::dot::attr::Attr>, Self::Error> {
            use graphitude::dot::attr::Attr;

            // Get the edge data and create a label attribute
            let edge_data = self.graph.edge_data(edge_id);
            let label = edge_data.to_string();
            if !label.is_empty() {
                Ok(vec![Attr::Label(label)])
            } else {
                Ok(vec![])
            }
        }
    }

    fn write_graph_dot<G>(
        graph: &G,
        graph_name: &str,
        writer: &mut dyn Write,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        G: Graph,
        G::EdgeData: std::fmt::Display,
    {
        let generator = ConfigurableGenerator {
            graph_name: graph_name.to_string(),
            graph,
        };
        let mut buffer = Vec::new();
        Graph::write_dot(graph, &generator, &mut buffer)
            .map_err(|err| -> Box<dyn std::error::Error> { Box::new(err) })?;
        writer.write_all(&buffer)?;
        Ok(())
    }

    fn node_data_for(i: usize, node_type: DataType) -> Data {
        match node_type {
            DataType::I32 => Data::I32(i as i32),
            DataType::String => Data::String(format!("n{}", i)),
            DataType::None => Data::None,
        }
    }

    fn edge_data_for(i: usize, edge_type: DataType, edge_prefix: &str) -> Data {
        match edge_type {
            DataType::I32 => Data::I32(i as i32),
            DataType::String => Data::String(format!("{}{}", edge_prefix, i)),
            DataType::None => Data::None,
        }
    }

    fn build_linked_graph(
        node_type: DataType,
        edge_type: DataType,
        edge_prefix: &str,
        graph_kind: GraphKind,
        strict: bool,
    ) -> LinkedGraph<Data, Data, Directedness, EdgeMultiplicity> {
        let directedness = match graph_kind {
            GraphKind::Directed => Directedness::Directed,
            GraphKind::Undirected => Directedness::Undirected,
        };
        let edge_multiplicity = if strict {
            EdgeMultiplicity::SingleEdge
        } else {
            EdgeMultiplicity::MultipleEdges
        };
        let mut graph = LinkedGraph::new(directedness, edge_multiplicity);
        generate_large_graph_with(
            &mut graph,
            |i| node_data_for(i, node_type),
            |i| edge_data_for(i, edge_type, edge_prefix),
        );
        graph
    }

    fn write_graph_output<G>(graph: &G, args: &Args) -> Result<(), Box<dyn std::error::Error>>
    where
        G: Graph,
        G::EdgeData: std::fmt::Display,
    {
        eprintln!("Graph generated:");
        eprintln!("  Nodes: {}", graph.num_nodes());
        eprintln!("  Edges: {}", graph.num_edges());

        match args.output {
            Some(ref path) => {
                eprintln!("\nWriting to {}...", path);
                let mut file = File::create(path)?;
                write_graph_dot(graph, &args.graph_name, &mut file)?;
                eprintln!("DOT file written successfully!");
            }
            None => {
                eprintln!("\nWriting to stdout...");
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                write_graph_dot(graph, &args.graph_name, &mut handle)?;
                handle.flush()?;
                eprintln!("\nDOT output written to stdout");
            }
        }

        eprintln!("  Graph name: {}", args.graph_name);
        eprintln!("\nThe graph is too large to visualize with dot; try Graphi instead.");

        Ok(())
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args = Args::parse();

        eprintln!("Generating large graph...");

        let graph = build_linked_graph(
            args.node_type,
            args.edge_type,
            &args.edge_prefix,
            args.graph_kind,
            args.strict,
        );
        write_graph_output(&graph, &args)?;

        Ok(())
    }
}

#[cfg(feature = "dot")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    inner::run()
}

#[cfg(not(feature = "dot"))]
fn main() {
    println!("This example requires the 'dot' feature to be enabled.");
    println!("Run with: cargo run --example large_graph_to_dot --features dot");
}
