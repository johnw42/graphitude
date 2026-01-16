use std::hash::Hash;

use super::Graph;

// A graph representation for traversing object graphs using a user-provided neighbor function.
pub struct ObjectGraph<'a, V: 'a, F>
where
    F: Fn(&'a V) -> Vec<&'a V>,
{
    neighbors_fn: F,
    root: &'a V,
}

impl<'a, V, F> ObjectGraph<'a, V, F>
where
    F: Fn(&'a V) -> Vec<&'a V>,
{
    // Create a new ObjectGraph given an object and a function to get its neighbors.
    pub fn new(root: &'a V, neighbors_fn: F) -> Self {
        Self { neighbors_fn, root }
    }

    pub fn root(&self) -> VertexId<'a, V> {
        VertexId(self.root)
    }
}

#[derive(Debug)]
pub struct VertexId<'a, V>(&'a V);

impl<'a, V> PartialEq for VertexId<'a, V> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'a, V> Eq for VertexId<'a, V> {}

impl<'a, V> Clone for VertexId<'a, V> {
    fn clone(&self) -> Self {
        VertexId(self.0)
    }
}

impl<'a, V> Copy for VertexId<'a, V> {}

impl<'a, V> Hash for VertexId<'a, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as *const V).hash(state);
    }
}

impl<'a, V> From<&'a V> for VertexId<'a, V> {
    fn from(value: &'a V) -> Self {
        VertexId(value)
    }
}

impl<'a, V, F> Graph for ObjectGraph<'a, V, F>
where
    F: Fn(&'a V) -> Vec<&'a V>,
{
    type VertexId = VertexId<'a, V>;
    type VertexData = &'a V;
    type EdgeData = ();

    fn neighbors(&self, from: &Self::VertexId) -> impl IntoIterator<Item = Self::VertexId> {
        (self.neighbors_fn)(&self.vertex_data(from))
            .into_iter()
            .map(|v| VertexId(v))
    }

    fn vertex_data(&self, id: &Self::VertexId) -> Self::VertexData {
        id.0
    }

    fn edge_data(&self, from: &Self::VertexId, to: &Self::VertexId) -> Option<Self::EdgeData> {
        let neighbors = (self.neighbors_fn)(&self.vertex_data(from));
        neighbors
            .iter()
            .position(|&v| VertexId(v) == *to)
            .map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_graph() {
        struct Node {
            value: i32,
            neighbors: Vec<Node>,
        }

        let node3 = Node {
            value: 3,
            neighbors: vec![],
        };
        let node2 = Node {
            value: 2,
            neighbors: vec![node3],
        };
        let node1 = Node {
            value: 1,
            neighbors: vec![node2],
        };

        let graph = ObjectGraph::new(&node1, |node: &Node| node.neighbors.iter().collect());

        let root_id = graph.root();
        assert_eq!(graph.vertex_data(&root_id).value, 1);

        let neighbors: Vec<_> = graph.neighbors(&root_id).into_iter().collect();
        assert_eq!(neighbors.len(), 1);
        assert_eq!(graph.vertex_data(&neighbors[0]).value, 2);

        let second_neighbors: Vec<_> = graph.neighbors(&neighbors[0]).into_iter().collect();
        assert_eq!(second_neighbors.len(), 1);
        assert_eq!(graph.vertex_data(&second_neighbors[0]).value, 3);

        assert!(graph.has_edge(&root_id, &neighbors[0]));
        assert!(!graph.has_edge(&root_id, &second_neighbors[0]));
    }

    #[cfg(feature = "pathfinding")]
    #[test]
    fn test_shortest_paths() {
        #[derive(Debug)]
        struct Node<'a> {
            value: i32,
            neighbors: Vec<&'a Node<'a>>,
        }

        //            1
        //           /|
        //          2 |
        //         / \|
        //        3   4
        let node4 = Node {
            value: 4,
            neighbors: vec![],
        };
        let node3 = Node {
            value: 3,
            neighbors: vec![],
        };
        let node2 = Node {
            value: 2,
            neighbors: vec![&node3, &node4],
        };
        let node1 = Node {
            value: 1,
            neighbors: vec![&node2, &node4],
        };

        let graph = ObjectGraph::new(&node1, |node: &Node| node.neighbors.clone());

        let root_id = graph.root();

        let paths = graph.shortest_paths(&root_id, |_from, _to| 1);
        assert_eq!(paths.len(), 4);
        assert_eq!(paths.get(&graph.root()).unwrap().1, 0); // cost to self is 0
        assert_eq!(paths.get(&graph.root()).unwrap().0.len(), 1); // path to self is just self
    }
}
