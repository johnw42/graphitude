#[cfg(test)]
mod debug {
    use graphitude::adjacency_graph::AdjacencyGraph;
    use graphitude::adjacency_matrix::HashStorage;
    use graphitude::directedness::Directed;
    use graphitude::{Graph, GraphMut, MappingResult};

    type TestGraph = AdjacencyGraph<i32, String, Directed, HashStorage>;

    fn new_node_data(i: usize) -> i32 {
        i as i32
    }

    fn new_edge_data(i: usize) -> String {
        format!("e{}", i)
    }

    #[test]
    fn test_large_graph_remove_with_compact() {
        // Simplified version of generate_large_graph
        let mut graph: TestGraph = GraphMut::new();

        // Create 50 nodes
        let mut all_nodes = Vec::new();
        for i in 0..50 {
            let node = graph.add_node(new_node_data(i));
            all_nodes.push(node);
        }

        // Add edges between consecutive nodes
        for i in 0..49 {
            graph.add_edge(&all_nodes[i], &all_nodes[i + 1], new_edge_data(i));
        }

        println!(
            "Initial graph: {} nodes, {} edges",
            graph.num_nodes(),
            graph.num_edges()
        );

        // Collect node IDs
        let mut node_ids = graph.node_ids().collect::<Vec<_>>();
        println!("Collected {} node IDs", node_ids.len());

        let total = node_ids.len();

        // Try the problematic pattern
        for i in 0..total {
            if i % 5 == 0 {
                println!("\nIteration {}: node_ids.len() = {}", i, node_ids.len());
            }
            let idx_to_remove = i % node_ids.len();
            let node_to_remove = node_ids.remove(idx_to_remove);
            graph.remove_node(&node_to_remove);

            if i % 10 == 0 && i > 0 {
                println!("  Compacting at iteration {}...", i);
                println!(
                    "    Before compact: graph has {} nodes, node_ids has {} entries",
                    graph.num_nodes(),
                    node_ids.len()
                );
                let before = node_ids.len();
                let mut remap_count = 0;
                graph.compact_with(
                    Some(|r: MappingResult<_>| match r {
                        MappingResult::Remapped(old_id, new_id) => {
                            remap_count += 1;
                            for j in 0..node_ids.len() {
                                if node_ids[j] == old_id {
                                    node_ids[j] = new_id.clone();
                                }
                            }
                        }
                        _ => {}
                    }),
                    None::<fn(MappingResult<_>)>,
                );
                println!(
                    "    After compact: {} remappings, graph has {} nodes, node_ids has {} entries",
                    remap_count,
                    graph.num_nodes(),
                    node_ids.len()
                );
                assert_eq!(
                    before,
                    node_ids.len(),
                    "node_ids length changed during compact!"
                );
            }
        }

        println!(
            "\nFinal: {} nodes, {} edges",
            graph.num_nodes(),
            graph.num_edges()
        );
        assert_eq!(graph.num_nodes(), 0);
        assert_eq!(graph.num_edges(), 0);
    }
}
