#![cfg(feature = "pathfinding")]

use crate::{Graph, end_pair::EndPair as _, path::Path};
use std::{
    collections::{HashMap, HashSet},
    ops::Add,
};

pub fn shortest_paths<'g, G, C: Default + Ord + Copy + Add<Output = C>>(
    graph: &'g G,
    start: &G::NodeId,
    distance_fn: impl Fn(&G::EdgeId) -> C,
) -> HashMap<G::NodeId, (Path<'g, G>, C)>
where
    G: Graph + ?Sized,
{
    // Find shortest paths using Dijkstra's algorithm.

    let mut distances: HashMap<G::NodeId, C> = HashMap::new();
    let mut predecessors: HashMap<G::NodeId, (G::EdgeId, G::NodeId)> = HashMap::new();
    let mut unvisited: HashSet<G::NodeId> = graph.nodes().collect();

    distances.insert(start.clone(), C::default());

    while !unvisited.is_empty() {
        // Find unvisited node with minimum distance
        let current = unvisited
            .iter()
            .filter_map(|node| distances.get(node).map(|&dist| (node.clone(), dist)))
            .min_by_key(|(_, dist)| *dist);

        let (current_node, current_dist) = match current {
            Some(pair) => pair,
            None => break, // No more reachable nodes
        };

        unvisited.remove(&current_node);

        // Update distances to neighbors
        for edge_id in graph.edges_from(&current_node) {
            let ends = graph.edge_ends(&edge_id);
            let neighbor = ends.other_value(&current_node).into_inner();
            if unvisited.contains(&neighbor) {
                let edge_distance = distance_fn(&edge_id);
                let new_dist = current_dist + edge_distance;

                let should_update = distances
                    .get(&neighbor)
                    .is_none_or(|&old_dist| new_dist < old_dist);

                if should_update {
                    distances.insert(neighbor.clone(), new_dist);
                    predecessors.insert(neighbor.clone(), (edge_id, current_node.clone()));
                }
            }
        }
    }

    // Build paths from predecessors
    let mut result: HashMap<G::NodeId, (Path<'g, G>, C)> = HashMap::new();
    for (node, &dist) in &distances {
        if node == start {
            result.insert(
                start.clone(),
                (Path::new(graph, start.clone()), C::default()),
            );
        } else {
            let mut current = node.clone();

            let mut path_edges = Vec::new();
            while let Some(pred) = predecessors.get(&current) {
                path_edges.push(pred.0.clone());
                current = pred.1.clone();
            }

            let mut path = Path::new(graph, start.clone());
            for edge_id in path_edges.iter().rev() {
                path.add_edge(edge_id.clone());
            }

            result.insert(node.clone(), (path, dist));
        }
    }

    result
}
