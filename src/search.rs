use std::collections::{HashSet, VecDeque};

use super::{EdgeId, Graph};

const DEFAULT_HASH_SET_CAPACITY: usize = 64;

pub struct BfsIterator<'g, G: Graph> {
    graph: &'g G,
    visited: HashSet<G::NodeId>,
    queue: VecDeque<G::NodeId>,
}

impl<'g, G> BfsIterator<'g, G>
where
    G: Graph,
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
    G: Graph,
{
    type Item = G::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(nid) = self.queue.pop_front() {
            if self.visited.contains(&nid) {
                continue;
            }
            self.visited.insert(nid.clone());
            for eid in self.graph.edges_from(nid.clone()) {
                let neighbor = eid.target();
                if !self.visited.contains(&neighbor) {
                    self.queue.push_back(neighbor);
                }
            }
            return Some(nid);
        }
        None
    }
}

pub struct BfsIteratorWithPaths<'g, G: Graph> {
    graph: &'g G,
    visited: HashSet<G::NodeId>,
    queue: VecDeque<Vec<G::NodeId>>,
}

impl<'g, G> BfsIteratorWithPaths<'g, G>
where
    G: Graph,
{
    pub fn new(graph: &'g G, start: Vec<G::NodeId>) -> Self {
        Self {
            graph,
            visited: HashSet::with_capacity(DEFAULT_HASH_SET_CAPACITY),
            queue: start.into_iter().map(|v| vec![v]).collect(),
        }
    }
}

impl<'g, G> Iterator for BfsIteratorWithPaths<'g, G>
where
    G: Graph,
{
    type Item = Vec<G::NodeId>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.queue.pop_front() {
            let nid = path.last().unwrap().clone();
            if self.visited.insert(nid.clone()) {
                for eid in self.graph.edges_from(nid.clone()) {
                    let neighbor = eid.target();
                    if !self.visited.contains(&neighbor) {
                        let mut new_path = path.clone();
                        new_path.push(neighbor);
                        self.queue.push_back(new_path);
                    }
                }
                return Some(path);
            }
        }
        None
    }
}

pub struct DfsIterator<'g, G: Graph> {
    graph: &'g G,
    visited: HashSet<G::NodeId>,
    stack: Vec<G::NodeId>,
}

impl<'g, G> DfsIterator<'g, G>
where
    G: Graph,
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
    G: Graph,
{
    type Item = G::NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(nid) = self.stack.pop() {
            if self.visited.insert(nid.clone()) {
                let mut successors = self.graph.successors(nid.clone()).collect::<Vec<_>>();
                successors.reverse();
                self.stack.extend(successors);
                return Some(nid);
            }
        }
        None
    }
}

pub struct DfsIteratorWithPaths<'g, G: Graph> {
    graph: &'g G,
    visited: HashSet<G::NodeId>,
    stack: Vec<Vec<G::NodeId>>,
}

impl<'g, G> DfsIteratorWithPaths<'g, G>
where
    G: Graph,
{
    pub fn new(graph: &'g G, start: Vec<G::NodeId>) -> Self {
        let mut stack = start.into_iter().map(|v| vec![v]).collect::<Vec<_>>();
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
    G: Graph,
{
    type Item = Vec<G::NodeId>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.stack.pop() {
            let nid = path.last().unwrap().clone();
            if self.visited.insert(nid.clone()) {
                let successors = self.graph.successors(nid.clone()).collect::<Vec<_>>();
                for successor in successors.into_iter().rev() {
                    let mut new_path = path.clone();
                    new_path.push(successor);
                    self.stack.push(new_path);
                }
                return Some(path);
            }
        }
        None
    }
}

#[cfg(test)]
#[cfg(feature = "bitvec")]
mod tests {
    use crate::{GraphMut, adjacency_graph::AdjacencyGraph};

    use super::*;

    fn create_simple_graph() -> AdjacencyGraph<usize, ()> {
        let mut graph = AdjacencyGraph::new();
        let n0 = graph.add_node(0);
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        let n3 = graph.add_node(3);
        graph.add_edge(n0, n1, ());
        graph.add_edge(n0, n2, ());
        graph.add_edge(n1, n3, ());
        graph
    }

    fn create_cyclic_graph() -> AdjacencyGraph<usize, ()> {
        let mut graph = AdjacencyGraph::new();
        let n0 = graph.add_node(0);
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);
        graph.add_edge(n0, n1, ());
        graph.add_edge(n1, n2, ());
        graph.add_edge(n2, n0, ());
        graph
    }

    #[test]
    fn test_bfs_simple_graph() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert!(visited[0] == nodes[0]);
        assert!(visited[1] == nodes[1] || visited[1] == nodes[2]);
        assert!(visited[2] == nodes[2] || visited[2] == nodes[1]);
        assert!(visited[3] == nodes[3]);
    }

    #[test]
    fn test_bfs_visits_all_reachable() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: HashSet<_> = BfsIterator::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert!(visited.contains(&nodes[0]));
        assert!(visited.contains(&nodes[1]));
        assert!(visited.contains(&nodes[2]));
        assert!(visited.contains(&nodes[3]));
    }

    #[test]
    fn test_bfs_empty_start() {
        let graph = create_simple_graph();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![]).collect();
        assert_eq!(visited.len(), 0);
    }

    #[test]
    fn test_bfs_multiple_start_nodes() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: HashSet<_> = BfsIterator::new(&graph, vec![nodes[0], nodes[1]]).collect();
        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn test_bfs_handles_cycles() {
        let graph = create_cyclic_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(visited.len(), 3);
    }

    #[test]
    fn test_dfs_simple_graph() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: HashSet<_> = DfsIterator::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(
            visited,
            HashSet::from([nodes[0], nodes[1], nodes[3], nodes[2]])
        );
    }

    #[test]
    fn test_dfs_visits_all_reachable() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: HashSet<_> = DfsIterator::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert!(visited.contains(&nodes[0]));
        assert!(visited.contains(&nodes[1]));
        assert!(visited.contains(&nodes[2]));
        assert!(visited.contains(&nodes[3]));
    }

    #[test]
    fn test_dfs_empty_start() {
        let graph = create_simple_graph();
        let visited: Vec<_> = DfsIterator::new(&graph, vec![]).collect();
        assert_eq!(visited.len(), 0);
    }

    #[test]
    fn test_dfs_multiple_start_nodes() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: HashSet<_> = DfsIterator::new(&graph, vec![nodes[0], nodes[1]]).collect();
        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn test_dfs_handles_cycles() {
        let graph = create_cyclic_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: Vec<_> = DfsIterator::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(visited.len(), 3);
    }

    #[test]
    fn test_bfs_dfs_visit_same_nodes() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let bfs_visited: HashSet<_> = BfsIterator::new(&graph, vec![nodes[0]]).collect();
        let dfs_visited: HashSet<_> = DfsIterator::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(bfs_visited, dfs_visited);
    }

    #[test]
    fn test_bfs_wth_paths() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: HashSet<_> = BfsIteratorWithPaths::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert_eq!(
            visited,
            HashSet::from([
                vec![nodes[0]],
                vec![nodes[0], nodes[1]],
                vec![nodes[0], nodes[2]],
                vec![nodes[0], nodes[1], nodes[3]],
            ])
        );
    }

    #[test]
    fn test_dfs_wth_paths() {
        let graph = create_simple_graph();
        let nodes: Vec<_> = graph.node_ids().collect();
        let visited: HashSet<_> = DfsIteratorWithPaths::new(&graph, vec![nodes[0]]).collect();
        assert_eq!(
            visited,
            HashSet::from([
                vec![nodes[0]],
                vec![nodes[0], nodes[2]],
                vec![nodes[0], nodes[1]],
                vec![nodes[0], nodes[1], nodes[3]],
            ])
        );
    }
}
