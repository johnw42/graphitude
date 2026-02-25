use crate::{Graph, GraphMut};

/// Generates a large graph with an irregular structure using custom closures
/// for node and edge data generation.
///
/// The graph structure includes:
/// - Cluster 1: Dense cluster (50 nodes, ~60% connectivity)
/// - Cluster 2: Medium cluster (80 nodes, ~30% connectivity)
/// - Cluster 3: Large sparse cluster (150 nodes, ~8% connectivity)
/// - Hub nodes (20 nodes with many connections)
/// - Scattered nodes (100 nodes with few connections)
/// - Bridge nodes connecting clusters (10 nodes)
/// - Long-range connections between random nodes
/// - Self loops
///
/// The resulting graph has approximately 500 nodes and 2000 edges.
///
/// # Arguments
///
/// * `new_node_data` - A closure that takes an index and returns node data
/// * `new_edge_data` - A closure that takes an index and returns edge data
pub fn generate_large_graph<G, FN, FE>(graph: &mut G, mut new_node_data: FN, mut new_edge_data: FE)
where
    G: GraphMut,
    FN: FnMut(usize) -> <G as Graph>::NodeData,
    FE: FnMut(usize) -> <G as Graph>::EdgeData,
{
    let mut node_counter = 0;
    let mut edge_counter = 0;

    // Create an irregular graph with ~500 nodes and ~2000 edges
    // Structure includes: clusters, hubs, sparse regions, and bridges

    let mut all_nodes = Vec::new();

    // Cluster 1: Dense cluster (50 nodes, highly connected)
    let cluster1_start = all_nodes.len();
    for _ in 0..50 {
        let node = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(node);
    }

    // Connect nodes within cluster 1 with ~60% density
    for i in cluster1_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 7 + j * 11) % 10 < 6 {
                graph.add_edge(&all_nodes[i], &all_nodes[j], new_edge_data(edge_counter));
                edge_counter += 1;
            }
        }
    }

    // Cluster 2: Medium cluster (80 nodes, moderately connected)
    let cluster2_start = all_nodes.len();
    for _ in 0..80 {
        let node = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(node);
    }
    // Connect nodes within cluster 2 with ~30% density
    for i in cluster2_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 13 + j * 17) % 10 < 3 {
                graph.add_edge(&all_nodes[i], &all_nodes[j], new_edge_data(edge_counter));
                edge_counter += 1;
            }
        }
    }

    // Cluster 3: Large sparse cluster (150 nodes, sparsely connected)
    let cluster3_start = all_nodes.len();
    for _ in 0..150 {
        let node = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(node);
    }
    // Connect nodes within cluster 3 with ~8% density
    for i in cluster3_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 19 + j * 23) % 100 < 8 {
                graph.add_edge(&all_nodes[i], &all_nodes[j], new_edge_data(edge_counter));
                edge_counter += 1;
            }
        }
    }

    // Add hub nodes (20 nodes with many connections)
    let hubs_start = all_nodes.len();
    for _ in 0..20 {
        let hub = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(hub.clone());

        // Connect each hub to random existing nodes
        #[allow(clippy::needless_range_loop)]
        for i in 0..all_nodes.len() - 1 {
            if (hubs_start * 29 + i * 31) % 7 < 4 {
                graph.add_edge(&hub, &all_nodes[i], new_edge_data(edge_counter));
                edge_counter += 1;
            }
        }
    }

    // Add scattered nodes (100 nodes with few connections)
    let scattered_start = all_nodes.len();
    for _ in 0..100 {
        let node = graph.add_node(new_node_data(node_counter));
        node_counter += 1;
        all_nodes.push(node.clone());

        // Connect to 1-3 random other nodes
        let num_connections = ((scattered_start + all_nodes.len()) % 3) + 1;
        for c in 0..num_connections {
            let target_idx =
                (scattered_start * 37 + all_nodes.len() * 41 + c * 43) % (all_nodes.len() - 1);
            graph.add_edge(&node, &all_nodes[target_idx], new_edge_data(edge_counter));
            edge_counter += 1;
        }
    }

    // Add bridge nodes connecting clusters (10 nodes)
    for i in 0..10 {
        let bridge = graph.add_node(new_node_data(node_counter));
        node_counter += 1;

        // Connect to nodes from different clusters
        let idx1 = (i * 47) % (cluster2_start - cluster1_start) + cluster1_start;
        let idx2 = (i * 53) % (cluster3_start - cluster2_start) + cluster2_start;
        let idx3 = (i * 59) % (hubs_start - cluster3_start) + cluster3_start;

        graph.add_edge(&bridge, &all_nodes[idx1], new_edge_data(edge_counter));
        edge_counter += 1;
        graph.add_edge(&bridge, &all_nodes[idx2], new_edge_data(edge_counter));
        edge_counter += 1;
        graph.add_edge(&bridge, &all_nodes[idx3], new_edge_data(edge_counter));
        edge_counter += 1;

        all_nodes.push(bridge);
    }

    // Add some long-range connections between random nodes
    for i in 0..200 {
        let idx1 = (i * 61) % all_nodes.len();
        let idx2 = (i * 67 + 100) % all_nodes.len();
        if idx1 != idx2 {
            graph.add_edge(
                &all_nodes[idx1],
                &all_nodes[idx2],
                new_edge_data(edge_counter),
            );
            edge_counter += 1;
        }
    }

    // Add reciprocal edge loops between pairs of nodes
    for i in 0..50 {
        let idx1 = (i * 73 + 7) % all_nodes.len();
        let idx2 = (i * 79 + 11) % all_nodes.len();
        if idx1 == idx2 {
            continue;
        }
        graph.add_edge(
            &all_nodes[idx1],
            &all_nodes[idx2],
            new_edge_data(edge_counter),
        );
        edge_counter += 1;
        graph.add_edge(
            &all_nodes[idx2],
            &all_nodes[idx1],
            new_edge_data(edge_counter),
        );
        edge_counter += 1;
    }

    // Add some self loops
    for i in 0..50 {
        let idx = (i * 71) % all_nodes.len();
        graph.add_edge(
            &all_nodes[idx],
            &all_nodes[idx],
            new_edge_data(edge_counter),
        );
        edge_counter += 1;
    }
}
