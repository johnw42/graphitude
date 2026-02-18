use std::collections::{HashSet, VecDeque};

use crate::path::Path;

use super::prelude::*;

const DEFAULT_HASH_SET_CAPACITY: usize = 64;

/// Iterator for breadth-first search traversal of a graph.
///
/// Visits nodes in breadth-first order starting from one or more root nodes.
/// Each node is visited at most once.
pub struct BfsIterator<'g, G: Graph + ?Sized> {
    graph: &'g G,
    visited: HashSet<G::NodeId>,
    queue: VecDeque<G::NodeId>,
}

impl<'g, G> BfsIterator<'g, G>
where
    G: Graph + ?Sized,
{
    pub fn new(graph: &'g G, start: Vec<G::NodeId>) -> Self {
        Self {
            graph,
            visited: HashSet::with_capacity(DEFAULT_HASH_SET_CAPACITY),
            queue: start.into(),
        }
    }
}

impl<'g, G> Iterator for BfsIterator<'g, G>
where
    G: Graph + ?Sized,
{
    type Item = G::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(nid) = self.queue.pop_front() {
            if self.visited.contains(&nid) {
                continue;
            }
            self.visited.insert(nid.clone());
            for eid in self.graph.edges_from(&nid) {
                let neighbor = eid.other_end(&nid);
                if !self.visited.contains(&neighbor) {
                    self.queue.push_back(neighbor);
                }
            }
            return Some(nid);
        }
        None
    }
}

/// Iterator for breadth-first search traversal that yields paths to each node.
///
/// Visits nodes in breadth-first order and yields the path from a root to each visited node.
/// Each node is visited at most once, and the first path found is returned.
pub struct BfsIteratorWithPaths<'g, G: Graph + ?Sized> {
    graph: &'g G,
    visited: HashSet<G::NodeId>,
    queue: VecDeque<Path<G::EdgeId>>,
}

impl<'g, G> BfsIteratorWithPaths<'g, G>
where
    G: Graph + ?Sized,
{
    pub fn new(graph: &'g G, start: Vec<G::NodeId>) -> Self {
        Self {
            graph,
            visited: HashSet::with_capacity(DEFAULT_HASH_SET_CAPACITY),
            queue: start.into_iter().map(Path::new).collect(),
        }
    }
}

impl<'g, G> Iterator for BfsIteratorWithPaths<'g, G>
where
    G: Graph + ?Sized,
{
    type Item = Path<G::EdgeId>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.queue.pop_front() {
            let nid = path.last_node().clone();
            if self.visited.insert(nid.clone()) {
                for eid in self.graph.edges_from(&nid) {
                    let neighbor = eid.other_end(&nid);
                    if !self.visited.contains(&neighbor) {
                        let mut new_path = path.clone();
                        new_path.add_edge_and_node(eid, neighbor);
                        self.queue.push_back(new_path);
                    }
                }
                return Some(path);
            }
        }
        None
    }
}

/// Iterator for depth-first search traversal of a graph.
///
/// Visits nodes in depth-first order starting from one or more root nodes.
/// Each node is visited at most once.
pub struct DfsIterator<'g, G: Graph + ?Sized> {
    graph: &'g G,
    visited: HashSet<G::NodeId>,
    stack: Vec<G::NodeId>,
}

impl<'g, G> DfsIterator<'g, G>
where
    G: Graph + ?Sized,
{
    pub fn new(graph: &'g G, start: Vec<G::NodeId>) -> Self {
        let mut stack = start;
        stack.reverse();
        Self {
            graph,
            visited: HashSet::with_capacity(DEFAULT_HASH_SET_CAPACITY),
            stack,
        }
    }
}

impl<'g, G> Iterator for DfsIterator<'g, G>
where
    G: Graph + ?Sized,
{
    type Item = G::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(nid) = self.stack.pop() {
            if self.visited.insert(nid.clone()) {
                let mut successors = self.graph.successors(&nid).collect::<Vec<_>>();
                successors.reverse();
                self.stack.extend(successors);
                return Some(nid);
            }
        }
        None
    }
}

/// Iterator for depth-first search traversal that yields paths to each node.
///
/// Visits nodes in depth-first order and yields the path from a root to each visited node.
/// Each node is visited at most once, and the first path found is returned.
pub struct DfsIteratorWithPaths<'g, G: Graph + ?Sized> {
    graph: &'g G,
    visited: HashSet<G::NodeId>,
    stack: Vec<Path<G::EdgeId>>,
}

impl<'g, G> DfsIteratorWithPaths<'g, G>
where
    G: Graph + ?Sized,
{
    pub fn new(graph: &'g G, start: Vec<G::NodeId>) -> Self {
        let mut stack = start.into_iter().map(Path::new).collect::<Vec<_>>();
        stack.reverse();
        Self {
            graph,
            visited: HashSet::with_capacity(DEFAULT_HASH_SET_CAPACITY),
            stack,
        }
    }
}

impl<'g, G> Iterator for DfsIteratorWithPaths<'g, G>
where
    G: Graph + ?Sized,
{
    type Item = Path<G::EdgeId>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.stack.pop() {
            let nid = path.last_node().clone();
            if self.visited.insert(nid.clone()) {
                let edges = self.graph.edges_from(&nid).collect::<Vec<_>>();
                for eid in edges.into_iter().rev() {
                    let mut new_path = path.clone();
                    new_path.add_edge_and_node(eid.clone(), eid.other_end(&nid));
                    self.stack.push(new_path);
                }
                return Some(path);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{Directed, GraphMut, LinkedGraph, MultipleEdges};

    use super::*;

    type TestGraph = LinkedGraph<usize, (), Directed, MultipleEdges>;

    fn create_simple_graph() -> (
        TestGraph,
        Vec<<TestGraph as Graph>::NodeId>,
        Vec<<TestGraph as Graph>::EdgeId>,
    ) {
        let mut graph = TestGraph::default();
        let n0 = graph.add_node(0);
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let n3 = graph.add_node(3);
        let edges = vec![
            graph.add_edge(&n0, &n1, ()).unwrap(),
            graph.add_edge(&n0, &n2, ()).unwrap(),
            graph.add_edge(&n1, &n3, ()).unwrap(),
        ];
        (graph, vec![n0, n1, n2, n3], edges)
    }

    fn create_cyclic_graph() -> (
        TestGraph,
        Vec<<TestGraph as Graph>::NodeId>,
        Vec<<TestGraph as Graph>::EdgeId>,
    ) {
        let mut graph = TestGraph::default();
        let n0 = graph.add_node(0);
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let edges = vec![
            graph.add_edge(&n0, &n1, ()).unwrap(),
            graph.add_edge(&n1, &n2, ()).unwrap(),
            graph.add_edge(&n2, &n0, ()).unwrap(),
        ];
        (graph, vec![n0, n1, n2], edges)
    }

    #[test]
    fn test_bfs_simple_graph() {
        let (graph, nodes, _) = create_simple_graph();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(visited.len(), 4);
        assert!(visited[0] == nodes[0]);
        assert!(visited[1] == nodes[1] || visited[1] == nodes[2]);
        assert!(visited[2] == nodes[2] || visited[2] == nodes[1]);
        assert!(visited[3] == nodes[3]);
    }

    #[test]
    fn test_bfs_visits_all_reachable() {
        let (graph, nodes, _) = create_simple_graph();
        let visited: HashSet<_> = BfsIterator::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(visited.len(), 4);
        assert!(visited.contains(&nodes[0]));
        assert!(visited.contains(&nodes[1]));
        assert!(visited.contains(&nodes[2]));
        assert!(visited.contains(&nodes[3]));
    }

    #[test]
    fn test_bfs_empty_start() {
        let (graph, _, _) = create_simple_graph();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![]).collect();
        assert_eq!(visited.len(), 0);
    }

    #[test]
    fn test_bfs_multiple_start_nodes() {
        let (graph, nodes, _) = create_simple_graph();
        let visited: HashSet<_> =
            BfsIterator::new(&graph, vec![nodes[0].clone(), nodes[1].clone()]).collect();
        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn test_bfs_handles_cycles() {
        let (graph, nodes, _) = create_cyclic_graph();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(visited.len(), 3);
    }

    #[test]
    fn test_dfs_simple_graph() {
        let (graph, nodes, _) = create_simple_graph();
        let visited: HashSet<_> = DfsIterator::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(
            visited,
            HashSet::from([
                nodes[0].clone(),
                nodes[1].clone(),
                nodes[3].clone(),
                nodes[2].clone()
            ])
        );
    }

    #[test]
    fn test_dfs_visits_all_reachable() {
        let (graph, nodes, _) = create_simple_graph();
        let visited: HashSet<_> = DfsIterator::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(visited.len(), 4);
        assert!(visited.contains(&nodes[0]));
        assert!(visited.contains(&nodes[1]));
        assert!(visited.contains(&nodes[2]));
        assert!(visited.contains(&nodes[3]));
    }

    #[test]
    fn test_dfs_empty_start() {
        let (graph, _, _) = create_simple_graph();
        let visited: Vec<_> = DfsIterator::new(&graph, vec![]).collect();
        assert_eq!(visited.len(), 0);
    }

    #[test]
    fn test_dfs_multiple_start_nodes() {
        let (graph, nodes, _) = create_simple_graph();
        let visited: HashSet<_> =
            DfsIterator::new(&graph, vec![nodes[0].clone(), nodes[1].clone()]).collect();
        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn test_dfs_handles_cycles() {
        let (graph, nodes, _) = create_cyclic_graph();
        let visited: Vec<_> = DfsIterator::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(visited.len(), 3);
    }

    #[test]
    fn test_bfs_dfs_visit_same_nodes() {
        let (graph, nodes, _) = create_simple_graph();
        let bfs_visited: HashSet<_> = BfsIterator::new(&graph, vec![nodes[0].clone()]).collect();
        let dfs_visited: HashSet<_> = DfsIterator::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(bfs_visited, dfs_visited);
    }

    #[test]
    fn test_bfs_wth_paths() {
        let (graph, nodes, edges) = create_simple_graph();
        let paths: Vec<_> = BfsIteratorWithPaths::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(paths.len(), 4);
        assert_eq!(
            paths,
            vec![
                Path::new(nodes[0].clone()),
                Path::from_edges(nodes[0].clone(), vec![edges[0].clone()]),
                Path::from_edges(nodes[0].clone(), vec![edges[1].clone()]),
                Path::from_edges(nodes[0].clone(), vec![edges[0].clone(), edges[2].clone()]),
            ]
        );
    }

    #[test]
    fn test_dfs_wth_paths() {
        let (graph, nodes, edges) = create_simple_graph();
        let visited: Vec<_> = DfsIteratorWithPaths::new(&graph, vec![nodes[0].clone()]).collect();
        assert_eq!(
            visited,
            vec![
                Path::new(nodes[0].clone()),
                Path::from_edges(nodes[0].clone(), vec![edges[0].clone()]),
                Path::from_edges(nodes[0].clone(), vec![edges[0].clone(), edges[2].clone()]),
                Path::from_edges(nodes[0].clone(), vec![edges[1].clone()]),
            ]
        );
    }
}
