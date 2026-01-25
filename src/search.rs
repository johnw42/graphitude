use std::collections::{HashSet, VecDeque};

use super::Graph;

const DEFAULT_HASH_SET_CAPACITY: usize = 64;

pub struct BfsIterator<'g, G: Graph> {
    graph: &'g G,
    visited: HashSet<G::VertexId>,
    queue: VecDeque<G::VertexId>,
}

impl<'g, G> BfsIterator<'g, G>
where
    G: Graph,
{
    pub fn new(graph: &'g G, start: Vec<G::VertexId>) -> Self {
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
    type Item = G::VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(vid) = self.queue.pop_front() {
            if self.visited.contains(&vid) {
                continue;
            }
            self.visited.insert(vid.clone());
            for eid in self.graph.edges_from(vid.clone()) {
                let neighbor = self.graph.edge_target(eid);
                if !self.visited.contains(&neighbor) {
                    self.queue.push_back(neighbor);
                }
            }
            return Some(vid);
        }
        None
    }
}

pub struct BfsIteratorWithPaths<'g, G: Graph> {
    graph: &'g G,
    visited: HashSet<G::VertexId>,
    queue: VecDeque<Vec<G::VertexId>>,
}

impl<'g, G> BfsIteratorWithPaths<'g, G>
where
    G: Graph,
{
    pub fn new(graph: &'g G, start: Vec<G::VertexId>) -> Self {
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
    type Item = Vec<G::VertexId>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.queue.pop_front() {
            let vid = path.last().unwrap().clone();
            if self.visited.insert(vid.clone()) {
                for eid in self.graph.edges_from(vid.clone()) {
                    let neighbor = self.graph.edge_target(eid);
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
    visited: HashSet<G::VertexId>,
    stack: Vec<G::VertexId>,
}

impl<'g, G> DfsIterator<'g, G>
where
    G: Graph,
{
    pub fn new(graph: &'g G, start: Vec<G::VertexId>) -> Self {
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
    type Item = G::VertexId;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(vid) = self.stack.pop() {
            if self.visited.insert(vid.clone()) {
                let mut successors = self.graph.successors(vid.clone()).collect::<Vec<_>>();
                successors.reverse();
                self.stack.extend(successors);
                return Some(vid);
            }
        }
        None
    }
}

pub struct DfsIteratorWithPaths<'g, G: Graph> {
    graph: &'g G,
    visited: HashSet<G::VertexId>,
    stack: Vec<Vec<G::VertexId>>,
}

impl<'g, G> DfsIteratorWithPaths<'g, G>
where
    G: Graph,
{
    pub fn new(graph: &'g G, start: Vec<G::VertexId>) -> Self {
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
    type Item = Vec<G::VertexId>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(path) = self.stack.pop() {
            let vid = path.last().unwrap().clone();
            if self.visited.insert(vid.clone()) {
                let successors = self.graph.successors(vid.clone()).collect::<Vec<_>>();
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
mod tests {
    use crate::{GraphMut, linked_graph::LinkedGraph};

    use super::*;

    fn create_simple_graph() -> LinkedGraph<usize, ()> {
        let mut graph = LinkedGraph::new();
        let v0 = graph.add_vertex(0);
        let v1 = graph.add_vertex(1);
        let v2 = graph.add_vertex(2);
        let v3 = graph.add_vertex(3);
        graph.add_edge(v0, v1, ());
        graph.add_edge(v0, v2, ());
        graph.add_edge(v1, v3, ());
        graph.add_edge(v2, v3, ());
        graph
    }

    fn create_cyclic_graph() -> LinkedGraph<usize, ()> {
        let mut graph = LinkedGraph::new();
        let v0 = graph.add_vertex(0);
        let v1 = graph.add_vertex(1);
        let v2 = graph.add_vertex(2);
        graph.add_edge(v0, v1, ());
        graph.add_edge(v1, v2, ());
        graph.add_edge(v2, v0, ());
        graph
    }

    #[test]
    fn test_bfs_simple_graph() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert_eq!(
            visited,
            vec![vertices[0], vertices[1], vertices[2], vertices[3]]
        );
    }

    #[test]
    fn test_bfs_visits_all_reachable() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: HashSet<_> = BfsIterator::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert!(visited.contains(&vertices[0]));
        assert!(visited.contains(&vertices[1]));
        assert!(visited.contains(&vertices[2]));
        assert!(visited.contains(&vertices[3]));
    }

    #[test]
    fn test_bfs_empty_start() {
        let graph = create_simple_graph();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![]).collect();
        assert_eq!(visited.len(), 0);
    }

    #[test]
    fn test_bfs_multiple_start_vertices() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: HashSet<_> =
            BfsIterator::new(&graph, vec![vertices[0], vertices[1]]).collect();
        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn test_bfs_handles_cycles() {
        let graph = create_cyclic_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: Vec<_> = BfsIterator::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(visited.len(), 3);
    }

    #[test]
    fn test_dfs_simple_graph() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: Vec<_> = DfsIterator::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert_eq!(
            visited,
            vec![vertices[0], vertices[1], vertices[3], vertices[2]]
        );
    }

    #[test]
    fn test_dfs_visits_all_reachable() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: HashSet<_> = DfsIterator::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert!(visited.contains(&vertices[0]));
        assert!(visited.contains(&vertices[1]));
        assert!(visited.contains(&vertices[2]));
        assert!(visited.contains(&vertices[3]));
    }

    #[test]
    fn test_dfs_empty_start() {
        let graph = create_simple_graph();
        let visited: Vec<_> = DfsIterator::new(&graph, vec![]).collect();
        assert_eq!(visited.len(), 0);
    }

    #[test]
    fn test_dfs_multiple_start_vertices() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: HashSet<_> =
            DfsIterator::new(&graph, vec![vertices[0], vertices[1]]).collect();
        assert_eq!(visited.len(), 4);
    }

    #[test]
    fn test_dfs_handles_cycles() {
        let graph = create_cyclic_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: Vec<_> = DfsIterator::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(visited.len(), 3);
    }

    #[test]
    fn test_bfs_dfs_visit_same_vertices() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let bfs_visited: HashSet<_> = BfsIterator::new(&graph, vec![vertices[0]]).collect();
        let dfs_visited: HashSet<_> = DfsIterator::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(bfs_visited, dfs_visited);
    }

    #[test]
    fn test_bfs_wth_paths() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: Vec<_> = BfsIteratorWithPaths::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert_eq!(
            visited,
            vec![
                vec![vertices[0]],
                vec![vertices[0], vertices[1]],
                vec![vertices[0], vertices[2]],
                vec![vertices[0], vertices[1], vertices[3]],
            ]
        );
    }

    #[test]
    fn test_dfs_wth_paths() {
        let graph = create_simple_graph();
        let vertices: Vec<_> = graph.vertex_ids().collect();
        let visited: Vec<_> = DfsIteratorWithPaths::new(&graph, vec![vertices[0]]).collect();
        assert_eq!(visited.len(), 4);
        assert_eq!(
            visited,
            vec![
                vec![vertices[0]],
                vec![vertices[0], vertices[1]],
                vec![vertices[0], vertices[1], vertices[3]],
                vec![vertices[0], vertices[2]]
            ]
        );
    }
}
