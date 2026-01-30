use std::collections::HashMap;

use jrw_graph::{
    Directed, EdgeId as EdgeIdTrait, Graph, GraphMut, NodeId as NodeIdTrait,
    graph_test_copy_from_with, graph_tests, tests::TestDataBuilder,
};

struct StringGraph {
    nodes: HashMap<NodeId, Node>,
    next_node_id: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct NodeId(usize);

impl NodeIdTrait for NodeId {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct EdgeId(NodeId, NodeId);

impl EdgeIdTrait for EdgeId {
    type NodeId = NodeId;
    type Directedness = Directed;

    fn source(&self) -> NodeId {
        self.0
    }

    fn target(&self) -> NodeId {
        self.1
    }
}

struct Node {
    data: String,
    edges_out: Vec<Edge>,
}

struct Edge {
    target: NodeId,
    data: String,
}

impl StringGraph {
    fn node(&self, id: &NodeId) -> &Node {
        self.nodes.get(id).expect("Invalid node ID")
    }

    fn edge(&self, id: &EdgeId) -> &Edge {
        self.nodes
            .get(&id.0)
            .expect("Invalid edge ID")
            .edges_out
            .iter()
            .find(|e| e.target == id.1)
            .expect("Invalid edge ID")
    }
}

impl Graph for StringGraph {
    type NodeData = String;
    type NodeId = NodeId;
    type EdgeData = String;
    type EdgeId = EdgeId;
    type Directedness = Directed;

    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData {
        &self.node(&id).data
    }

    fn edge_data(&self, id: Self::EdgeId) -> &Self::EdgeData {
        &self.edge(&id).data
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.keys().cloned()
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.nodes.iter().flat_map(|(from_id, node)| {
            node.edges_out
                .iter()
                .map(move |edge| EdgeId(from_id.clone(), edge.target.clone()))
        })
    }
}

impl GraphMut for StringGraph {
    fn new() -> Self {
        StringGraph {
            nodes: HashMap::new(),
            next_node_id: 0,
        }
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        self.nodes.insert(
            id,
            Node {
                data,
                edges_out: Vec::new(),
            },
        );
        id
    }

    fn add_or_replace_edge(
        &mut self,
        from: Self::NodeId,
        to: Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        assert!(self.nodes.contains_key(&to), "Invalid 'to' node ID");
        self.nodes
            .get_mut(&from)
            .expect("Invalid 'from' node ID")
            .edges_out
            .push(Edge {
                target: to.clone(),
                data: data,
            });
        (EdgeId(from, to), None)
    }

    fn remove_node(&mut self, id: Self::NodeId) -> Self::NodeData {
        for node in self.nodes.values_mut() {
            node.edges_out.retain(|e| e.target != id);
        }
        self.nodes.remove(&id).expect("Invalid node ID").data
    }

    fn remove_edge(&mut self, EdgeId(from, to): Self::EdgeId) -> Self::EdgeData {
        let node = self.nodes.get_mut(&from).expect("Invalid 'from' node ID");
        let pos = node
            .edges_out
            .iter()
            .position(|e| e.target == to)
            .expect("Invalid 'to' node ID");
        let data = node.edges_out.remove(pos).data;
        data
    }
}

impl TestDataBuilder for StringGraph {
    type Graph = Self;

    fn new_edge_data(i: usize) -> String {
        format!("e{}", i)
    }

    fn new_node_data(i: usize) -> String {
        format!("v{}", i)
    }
}

graph_tests!(StringGraph);
graph_test_copy_from_with!(
    StringGraph,
    |data| format!("{}-copied", data),
    |data| format!("{}-copied", data)
);
