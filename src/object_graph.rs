use std::{fmt::Debug, marker::PhantomData, mem::transmute};

use derivative::Derivative;

use crate::directedness::Directed;

use super::Graph;

/// Node identifier for [`ObjectGraph`].
///
/// Contains a raw pointer to the node data with a lifetime tied to the graph.
#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    Debug(bound = ""),
    PartialOrd(bound = ""),
    Ord(bound = "")
)]
pub struct NodeId<'g, N>(*const N, PhantomData<&'g N>);

impl<'a, N> crate::graph::NodeId for NodeId<'a, N> {}

impl<'a, N> From<&'a N> for NodeId<'a, N> {
    fn from(v: &'a N) -> Self {
        NodeId(v as *const N, PhantomData)
    }
}

/// Edge identifier for [`ObjectGraph`].
///
/// Represented as a tuple of source and target node IDs.
pub type EdgeId<'g, N> = (NodeId<'g, N>, NodeId<'g, N>);

/// A graph representation for traversing object graphs using a user-provided neighbor function.
pub struct ObjectGraph<'a, N, F> {
    neighbors_fn: F,
    roots: Vec<&'a N>,
}

impl<'a, N: Debug, F> ObjectGraph<'a, N, F>
where
    F: Fn(&'a N) -> Vec<&'a N>,
{
    /// Create a new ObjectGraph given an object and a function to get its neighbors.
    pub fn new(root: &'a N, neighbors_fn: F) -> Self {
        Self::new_multi(vec![root], neighbors_fn)
    }

    /// Create a new ObjectGraph given an object and a function to get its neighbors.
    pub fn new_multi(roots: Vec<&'a N>, neighbors_fn: F) -> Self {
        Self {
            neighbors_fn,
            roots,
        }
    }

    /// Get the NodeId of the root node.
    pub fn roots(&self) -> impl Iterator<Item = NodeId<'a, N>> {
        self.roots.iter().cloned().map(NodeId::from)
    }

    /// Get the NodeId for a given node reference.
    ///
    /// # Safety
    /// This function is unsafe because it creates a NodeId from a reference.
    /// The caller must ensure that the reference is to a valid node in the
    /// graph.
    pub unsafe fn node_id(&self, v: &'a N) -> NodeId<'a, N> {
        NodeId::from(v)
    }

    fn neighbors(&self, id: &NodeId<'a, N>) -> Vec<<Self as Graph>::NodeId> {
        let v = self.node_data(id);
        (self.neighbors_fn)(v)
            .iter()
            .map(|&neighbor| NodeId::from(neighbor))
            .collect()
    }

    fn make_edge_id(&self, from: &NodeId<'a, N>, to: &NodeId<'a, N>) -> EdgeId<'a, N> {
        (*from, *to)
    }
}

impl<'a, N: Debug> crate::graph::EdgeId for (NodeId<'a, N>, NodeId<'a, N>) {
    type NodeId = NodeId<'a, N>;
    type Directedness = Directed;

    fn source(&self) -> NodeId<'a, N> {
        self.0
    }

    fn target(&self) -> NodeId<'a, N> {
        self.1
    }
}

impl<'d, N: Debug, F> Graph for ObjectGraph<'d, N, F>
where
    F: Fn(&'d N) -> Vec<&'d N>,
{
    type NodeId = NodeId<'d, N>;
    type NodeData = &'d N;
    type EdgeId = (Self::NodeId, Self::NodeId);
    type EdgeData = ();
    type Directedness = Directed;

    fn node_data(&self, id: &NodeId<N>) -> &<Self as Graph>::NodeData {
        unsafe { transmute::<&*const N, &&'d N>(&id.0) }
    }

    fn edge_data(&self, (from, to): &<Self as Graph>::EdgeId) -> &<Self as Graph>::EdgeData {
        let neighbors = (self.neighbors_fn)(self.node_data(from));
        neighbors
            .iter()
            .position(|&v| NodeId::from(v) == *to)
            .map(|_| &())
            .expect("Edge does not exist")
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.neighbors(&from)
            .into_iter()
            .map(move |to| self.make_edge_id(&from, &to))
    }

    fn node_ids(&self) -> impl Iterator<Item = <Self as Graph>::NodeId> {
        self.bfs_multi(self.roots().collect())
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.node_ids()
            .flat_map(|from| self.edges_from(&from).collect::<Vec<_>>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_graph() {
        #[derive(Debug)]
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
        assert_eq!(graph.node_data(&root_id).value, 1);

        let neighbors: Vec<_> = graph.neighbors(&root_id).into_iter().collect();
        assert_eq!(neighbors.len(), 1);
        assert_eq!(graph.node_data(&neighbors[0]).value, 2);

        let second_neighbors: Vec<_> = graph.neighbors(&neighbors[0]).into_iter().collect();
        assert_eq!(second_neighbors.len(), 1);
        assert_eq!(graph.node_data(&second_neighbors[0]).value, 3);

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
        let id2 = unsafe { graph.node_id(&node2) };
        let id3 = unsafe { graph.node_id(&node3) };
        let id4 = unsafe { graph.node_id(&node4) };

        let paths = graph.shortest_paths(&id1, |_| 1);

        let values = |id| {
            paths
                .get(id)
                .unwrap()
                .0
                .nodes()
                .map(|nid| graph.node_data(&nid).value)
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
