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
    use graphitude::{
        dot::renderer::DotGenerator,
        linked_graph::LinkedGraph,
        prelude::*,
        tests::generate_large_graph_with,
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

    /// Type alias for a large directed graph with Data enum for nodes and edges.
    type LargeGraph = LinkedGraph<Data, Data, Directed>;

    /// DOT generator that uses a configurable graph name and has access to the graph.
    struct ConfigurableGenerator<'a> {
        graph_name: String,
        graph: &'a LargeGraph,
    }

    impl<'a> DotGenerator<LargeGraph> for ConfigurableGenerator<'a> {
        type Error = std::convert::Infallible;

        fn graph_name(&self) -> Result<String, Self::Error> {
            Ok(self.graph_name.clone())
        }

        fn edge_attrs(
            &self,
            edge_id: &<LargeGraph as Graph>::EdgeId,
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

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args = Args::parse();

        eprintln!("Generating large graph...");

        // Create closures based on selected node and edge types
        let node_type = args.node_type;
        let edge_type = args.edge_type;
        let edge_prefix = args.edge_prefix.clone();

        let graph: LargeGraph = generate_large_graph_with(
            move |i| match node_type {
                DataType::I32 => Data::I32(i as i32),
                DataType::String => Data::String(format!("n{}", i)),
                DataType::None => Data::None,
            },
            move |i| match edge_type {
                DataType::I32 => Data::I32(i as i32),
                DataType::String => Data::String(format!("{}{}", edge_prefix, i)),
                DataType::None => Data::None,
            },
        );

        eprintln!("Graph generated:");
        eprintln!("  Nodes: {}", graph.num_nodes());
        eprintln!("  Edges: {}", graph.num_edges());

        let generator = ConfigurableGenerator {
            graph_name: args.graph_name.clone(),
            graph: &graph,
        };

        // Write to file or stdout
        match args.output {
            Some(ref path) => {
                eprintln!("\nWriting to {}...", path);
                let mut file = File::create(path)?;
                graph.write_dot(&generator, &mut file)?;
                eprintln!("DOT file written successfully!");
                eprintln!("  Graph name: {}", args.graph_name);
                eprintln!(
                    "\nYou can visualize it with: dot -Tpng {} -o {}.png",
                    path,
                    path.trim_end_matches(".dot")
                );
            }
            None => {
                eprintln!("\nWriting to stdout...");
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                graph.write_dot(&generator, &mut handle)?;
                handle.flush()?;
                eprintln!("\nDOT output written to stdout");
                eprintln!("  Graph name: {}", args.graph_name);
                eprintln!("\nThe graph is too large to visualize with dot; try Graphi instead.");
            }
        }

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
