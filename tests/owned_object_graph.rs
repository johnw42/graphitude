use derivative::Derivative;

use graphitude::{EdgeId as EdgeIdTrait, Graph, NodeId as NodeIdTrait, directedness::Directed};

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
struct NodeId<N>(*const N);

impl<N> NodeIdTrait for NodeId<N> {}

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    Debug(bound = "")
)]
struct EdgeId<N>(NodeId<N>, NodeId<N>);

impl<N> EdgeIdTrait for EdgeId<N> {
    type NodeId = NodeId<N>;
    type Directedness = Directed;

    fn source(&self) -> NodeId<N> {
        self.0
    }

    fn target(&self) -> NodeId<N> {
        self.1
    }
}

// A graph representation for traversing object graphs using a user-provided neighbor function.
struct OwnedObjectGraph<N, F> {
    neighbors_fn: F,
    root: N,
}

impl<N, F> OwnedObjectGraph<N, F>
where
    F: for<'a> Fn(&'a N) -> Vec<&'a N>,
{
    // Create a new ObjectGraph given an object and a function to get its neighbors.
    fn new(root: N, neighbors_fn: F) -> Self {
        Self { neighbors_fn, root }
    }

    fn root(&self) -> NodeId<N> {
        NodeId(&self.root)
    }

    #[cfg(feature = "pathfinding")]
    fn node_id(&self, v: &N) -> NodeId<N> {
        NodeId(v)
    }
}

impl<N, F> Graph for OwnedObjectGraph<N, F>
where
    F: for<'a> Fn(&'a N) -> Vec<&'a N>,
{
    type NodeId = NodeId<N>;
    type NodeData = N;
    type EdgeData = ();
    type EdgeId = EdgeId<N>;
    type Directedness = Directed;

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        let node_data: &Self::NodeData = self.node_data(&from);
        let items = (self.neighbors_fn)(node_data);
        items.into_iter().map(move |v| EdgeId(*from, NodeId(v)))
    }

    fn node_data(&self, id: &NodeId<N>) -> &Self::NodeData {
        unsafe { &*id.0 }
    }

    fn edge_data(&self, _eid: &Self::EdgeId) -> &Self::EdgeData {
        &()
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.bfs(&self.root())
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.node_ids()
            .flat_map(|from| self.edges_from(&from).collect::<Vec<_>>())
    }
}

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

    let graph = OwnedObjectGraph::new(node1, |node: &Node| node.neighbors.iter().collect());

    let root_id = graph.root();
    assert_eq!(graph.node_data(&root_id).value, 1);

    let successors: Vec<_> = graph.successors(&root_id).into_iter().collect();
    assert_eq!(successors.len(), 1);
    assert_eq!(graph.node_data(&successors[0]).value, 2);
    let second_successors: Vec<_> = graph.successors(&successors[0]).into_iter().collect();
    assert_eq!(second_successors.len(), 1);
    assert_eq!(graph.node_data(&second_successors[0]).value, 3);

    assert!(graph.has_edge(&root_id, &successors[0]));
    assert!(!graph.has_edge(&root_id, &second_successors[0]));
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

    fn values<'a, F: for<'b> Fn(&'b Node<'a>) -> Vec<&'b Node<'a>>>(
        graph: &OwnedObjectGraph<Node<'a>, F>,
        path: &graphitude::path::Path<EdgeId<Node<'a>>>,
    ) -> Vec<i32> {
        path.nodes()
            .map(|nid| graph.node_data(&nid).value)
            .collect()
    }

    let graph = OwnedObjectGraph::new(node1, |node: &Node| node.neighbors.clone());

    let id1 = graph.root();
    let id2 = graph.node_id(&node2);
    let id3 = graph.node_id(&node3);
    let id4 = graph.node_id(&node4);

    let paths = graph.shortest_paths(&id1, |_| 1);
    assert_eq!(paths.len(), 4);
    assert_eq!(paths.get(&id1).unwrap().1, 0);
    assert_eq!(values(&graph, &paths.get(&id1).unwrap().0), vec![1]);
    assert_eq!(paths.get(&id2).unwrap().1, 1);
    assert_eq!(values(&graph, &paths.get(&id2).unwrap().0), vec![1, 2]);
    assert_eq!(paths.get(&id3).unwrap().1, 2);
    assert_eq!(values(&graph, &paths.get(&id3).unwrap().0), vec![1, 2, 3]);
    assert_eq!(paths.get(&id4).unwrap().1, 1);
    assert_eq!(values(&graph, &paths.get(&id4).unwrap().0), vec![1, 4]);
}
