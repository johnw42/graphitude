use std::{fmt::Debug, hash::Hash, marker::PhantomData, mem::transmute};

use crate::graph::Directed;

use super::Graph;

pub struct VertexId<'g, V>(*const V, PhantomData<&'g V>);

impl<'g, V> PartialEq for VertexId<'g, V> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<'g, V> Eq for VertexId<'g, V> {}

impl<'g, V> Clone for VertexId<'g, V> {
    fn clone(&self) -> Self {
        VertexId(self.0, self.1)
    }
}

impl<'g, V> Copy for VertexId<'g, V> {}

impl<'g, V> Hash for VertexId<'g, V> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<'g, V> Debug for VertexId<'g, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VertexId({:?})", self.0)
    }
}

impl<'a, V> From<&'a V> for VertexId<'a, V> {
    fn from(v: &'a V) -> Self {
        VertexId(v as *const V, PhantomData)
    }
}

pub type EdgeId<'g, V> = (VertexId<'g, V>, VertexId<'g, V>);

/// A graph representation for traversing object graphs using a user-provided neighbor function.
pub struct ObjectGraph<'a, V, F> {
    neighbors_fn: F,
    roots: Vec<&'a V>,
}

impl<'a, V, F> ObjectGraph<'a, V, F>
where
    F: Fn(&'a V) -> Vec<&'a V>,
{
    /// Create a new ObjectGraph given an object and a function to get its neighbors.
    pub fn new(root: &'a V, neighbors_fn: F) -> Self {
        Self::new_multi(vec![root], neighbors_fn)
    }

    /// Create a new ObjectGraph given an object and a function to get its neighbors.
    pub fn new_multi(roots: Vec<&'a V>, neighbors_fn: F) -> Self {
        Self {
            neighbors_fn,
            roots,
        }
    }

    /// Get the VertexId of the root vertex.
    pub fn roots(&self) -> impl Iterator<Item = VertexId<'a, V>> {
        self.roots.iter().cloned().map(VertexId::from)
    }

    /// Get the VertexId for a given vertex reference.
    ///
    /// # Safety
    /// This function is unsafe because it creates a VertexId from a reference.
    /// The caller must ensure that the reference is to a valid vertex in the
    /// graph.
    pub unsafe fn vertex_id(&self, v: &'a V) -> VertexId<'a, V> {
        VertexId::from(v)
    }

    fn neighbors(&self, id: &VertexId<'a, V>) -> Vec<<Self as Graph>::VertexId> {
        let v = self.vertex_data(id);
        (self.neighbors_fn)(v)
            .iter()
            .map(|&neighbor| VertexId::from(neighbor))
            .collect()
    }

    fn make_edge_id(&self, from: VertexId<'a, V>, to: VertexId<'a, V>) -> EdgeId<'a, V> {
        (from, to)
    }
}

impl<'a, V, F> Graph for ObjectGraph<'a, V, F>
where
    F: Fn(&'a V) -> Vec<&'a V>,
{
    type VertexId = VertexId<'a, V>;
    type VertexData = &'a V;
    type EdgeId = (Self::VertexId, Self::VertexId);
    type EdgeData = ();
    type Directedness = Directed;

    fn vertex_data(&self, id: &VertexId<V>) -> &<Self as Graph>::VertexData {
        unsafe { transmute::<&*const V, &&'a V>(&id.0) }
    }

    fn edge_data(&self, (from, to): &<Self as Graph>::EdgeId) -> &<Self as Graph>::EdgeData {
        let neighbors = (self.neighbors_fn)(self.vertex_data(from));
        neighbors
            .iter()
            .position(|&v| VertexId::from(v) == *to)
            .map(|_| &())
            .expect("Edge does not exist")
    }

    fn edges_from<'b>(&'b self, from: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> + 'b {
        let neighbors = self.neighbors(&from);
        neighbors
            .into_iter()
            .map(move |to| self.make_edge_id(from, to))
    }

    fn vertex_ids(&self) -> impl Iterator<Item = <Self as Graph>::VertexId> {
        self.bfs_multi(self.roots().collect())
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::VertexId, Self::VertexId) {
        eid
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

        let root_id = graph.roots().next().unwrap();
        assert_eq!(graph.vertex_data(&root_id).value, 1);

        let neighbors: Vec<_> = graph.neighbors(&root_id).into_iter().collect();
        assert_eq!(neighbors.len(), 1);
        assert_eq!(graph.vertex_data(&neighbors[0]).value, 2);

        let second_neighbors: Vec<_> = graph.neighbors(&neighbors[0]).into_iter().collect();
        assert_eq!(second_neighbors.len(), 1);
        assert_eq!(graph.vertex_data(&second_neighbors[0]).value, 3);

        assert!(graph.has_edge(root_id, neighbors[0]));
        assert!(!graph.has_edge(root_id, second_neighbors[0]));
    }

    #[cfg(feature = "pathfinding")]
    #[test]
    fn test_shortest_paths() {
        struct Node<'a> {
            value: i32,
            neighbors: Vec<&'a Node<'a>>,
        }

        //     1
        //    /|
        //   2 |
        //  / \|
        // 3   4
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

        let id1 = graph.roots().next().unwrap();
        let id2 = unsafe { graph.vertex_id(&node2) };
        let id3 = unsafe { graph.vertex_id(&node3) };
        let id4 = unsafe { graph.vertex_id(&node4) };

        let paths = graph.shortest_paths(&id1, |_| 1);

        let values = |id| {
            paths
                .get(id)
                .unwrap()
                .0
                .iter()
                .map(|vid| graph.vertex_data(vid).value)
                .collect::<Vec<_>>()
        };
        assert_eq!(paths.len(), 4);

        assert_eq!(paths.get(&id1).unwrap().1, 0);
        assert_eq!(values(&id1), vec![1]);
        assert_eq!(paths.get(&id2).unwrap().1, 1);
        assert_eq!(values(&id2), vec![1, 2]);
        assert_eq!(paths.get(&id3).unwrap().1, 2);
        assert_eq!(values(&id3), vec![1, 2, 3]);
        assert_eq!(paths.get(&id4).unwrap().1, 1);
        assert_eq!(values(&id4), vec![1, 4]);

        (&node1, &node2, &node3, &node4);
    }
}
