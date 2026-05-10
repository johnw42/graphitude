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
pub fn generate_large_graph<G, FN, FE>(
    graph: &mut G,
    mut new_node_data: FN,
    mut new_edge_data: FE,
    generate_parallel_edges: bool,
) where
    G: GraphMut,
    FN: FnMut(usize) -> <G as Graph>::NodeData,
    FE: FnMut(usize) -> <G as Graph>::EdgeData,
{
    let mut all_nodes = Vec::new();
    let mut edge_counter = 0;

    let mut add_node = |graph: &mut G, all_nodes: &mut Vec<_>| {
        let node = graph.add_node(new_node_data(all_nodes.len()));
        all_nodes.push(node);
    };

    let mut add_edge = |graph: &mut G, i: usize, j: usize, all_nodes: &[_]| {
        if generate_parallel_edges || !graph.has_edge_from_into(&all_nodes[i], &all_nodes[j]) {
            // Add a parallel edge with different data
            graph.add_edge(&all_nodes[i], &all_nodes[j], new_edge_data(edge_counter));
            edge_counter += 1;
        }
    };

    // Create an irregular graph with ~500 nodes and ~2000 edges
    // Structure includes: clusters, hubs, sparse regions, and bridges

    // Cluster 1: Dense cluster (50 nodes, highly connected)
    let cluster1_start = all_nodes.len();
    for _ in 0..50 {
        add_node(graph, &mut all_nodes);
    }

    // Connect nodes within cluster 1 with ~60% density
    for i in cluster1_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 7 + j * 11) % 10 < 6 {
                add_edge(graph, i, j, &all_nodes);
            }
        }
    }

    // Cluster 2: Medium cluster (80 nodes, moderately connected)
    let cluster2_start = all_nodes.len();
    for _ in 0..80 {
        add_node(graph, &mut all_nodes);
    }
    // Connect nodes within cluster 2 with ~30% density
    for i in cluster2_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 13 + j * 17) % 10 < 3 {
                add_edge(graph, i, j, &all_nodes);
            }
        }
    }

    // Cluster 3: Large sparse cluster (150 nodes, sparsely connected)
    let cluster3_start = all_nodes.len();
    for _ in 0..150 {
        add_node(graph, &mut all_nodes);
    }
    // Connect nodes within cluster 3 with ~8% density
    for i in cluster3_start..all_nodes.len() {
        for j in (i + 1)..all_nodes.len() {
            if (i * 19 + j * 23) % 100 < 8 {
                add_edge(graph, i, j, &all_nodes);
            }
        }
    }

    // Add hub nodes (20 nodes with many connections)
    let hubs_start = all_nodes.len();
    for _ in 0..20 {
        add_node(graph, &mut all_nodes);

        // Connect each hub to random existing nodes
        #[allow(clippy::needless_range_loop)]
        for i in 0..all_nodes.len() - 1 {
            if (hubs_start * 29 + i * 31) % 7 < 4 {
                add_edge(graph, hubs_start + i, i, &all_nodes);
            }
        }
    }

    // Add scattered nodes (100 nodes with few connections)
    let scattered_start = all_nodes.len();
    for _ in 0..100 {
        add_node(graph, &mut all_nodes);

        // Connect to 1-3 random other nodes
        let num_connections = ((scattered_start + all_nodes.len()) % 3) + 1;
        for c in 0..num_connections {
            let target_idx =
                (scattered_start * 37 + all_nodes.len() * 41 + c * 43) % (all_nodes.len() - 1);
            add_edge(graph, all_nodes.len() - 1, target_idx, &all_nodes);
        }
    }

    // Add bridge nodes connecting clusters (10 nodes)
    for i in 0..10 {
        let bridge = all_nodes.len();
        add_node(graph, &mut all_nodes);

        // Connect to nodes from different clusters
        let idx1 = (i * 47) % (cluster2_start - cluster1_start) + cluster1_start;
        let idx2 = (i * 53) % (cluster3_start - cluster2_start) + cluster2_start;
        let idx3 = (i * 59) % (hubs_start - cluster3_start) + cluster3_start;

        add_edge(graph, bridge, idx1, &all_nodes);
        add_edge(graph, bridge, idx2, &all_nodes);
        add_edge(graph, bridge, idx3, &all_nodes);
    }

    // Add some long-range connections between random nodes
    for i in 0..200 {
        let idx1 = (i * 61) % all_nodes.len();
        let idx2 = (i * 67 + 100) % all_nodes.len();
        if idx1 != idx2 {
            add_edge(graph, idx1, idx2, &all_nodes);
        }
    }

    // Add reciprocal edge loops between pairs of nodes
    for i in 0..50 {
        let idx1 = (i * 73 + 7) % all_nodes.len();
        let idx2 = (i * 79 + 11) % all_nodes.len();
        if idx1 == idx2 {
            continue;
        }
        add_edge(graph, idx1, idx2, &all_nodes);
        add_edge(graph, idx2, idx1, &all_nodes);
    }

    // Add some self loops
    for i in 0..50 {
        let idx = (i * 71) % all_nodes.len();
        add_edge(graph, idx, idx, &all_nodes);
    }
}
