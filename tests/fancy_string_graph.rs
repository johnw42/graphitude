use std::collections::HashMap;

use graphitude::{
    EdgeId as EdgeIdTrait,
    NodeId as NodeIdTrait,
    graph_test_copy_from_with,
    graph_tests,
    prelude::*,
    tests::TestDataBuilder,
};

struct StringGraph {
    nodes: HashMap<NodeId, Node>,
    next_node_id: usize,
    next_edge_id: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct NodeId(usize);

impl NodeIdTrait for NodeId {}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct EdgeId(NodeId, NodeId, usize);

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
    // The index is necessary to uniquely identify edges between the same pair of nodes.
    index: usize,
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
            .find(|e| e.target == id.1 && e.index == id.2)
            .expect("Invalid edge ID")
    }
}

impl Graph for StringGraph {
    type NodeData = String;
    type NodeId = NodeId;
    type EdgeData = String;
    type EdgeId = EdgeId;
    type Directedness = Directed;
    type EdgeMultiplicity = MultipleEdges;

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        &self.node(id).data
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        &self.edge(id).data
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.keys().cloned()
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.nodes.iter().flat_map(|(from_id, node)| {
            node.edges_out
                .iter()
                .map(move |edge| EdgeId(from_id.clone(), edge.target.clone(), edge.index))
        })
    }

    // Override these methods to provide O(1) or O(degree) validation instead of the
    // default O(n) iteration. This is necessary to make test_deconstruct_large_graph_by_nodes
    // reasonably fast, as it validates all nodes and edges after every compaction.

    fn check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        if self.nodes.contains_key(id) {
            Ok(())
        } else {
            Err("NodeId not found in graph")
        }
    }

    fn check_valid_edge_id(&self, id: &Self::EdgeId) -> Result<(), &'static str> {
        if let Some(node) = self.nodes.get(&id.0) {
            if node
                .edges_out
                .iter()
                .any(|e| e.target == id.1 && e.index == id.2)
            {
                Ok(())
            } else {
                Err("EdgeId not found in graph")
            }
        } else {
            Err("EdgeId not found in graph")
        }
    }
}

impl GraphMut for StringGraph {
    fn new() -> Self {
        StringGraph {
            nodes: HashMap::new(),
            next_node_id: 0,
            next_edge_id: 0,
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
        from: &Self::NodeId,
        to: &Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        assert!(self.nodes.contains_key(to), "Invalid 'to' node ID");
        let edge_index = self.next_edge_id;
        self.next_edge_id += 1;
        self.nodes
            .get_mut(from)
            .expect("Invalid 'from' node ID")
            .edges_out
            .push(Edge {
                target: to.clone(),
                data: data,
                index: edge_index,
            });
        (EdgeId(*from, *to, edge_index), None)
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData {
        for node in self.nodes.values_mut() {
            node.edges_out.retain(|e| e.target != *id);
        }
        self.nodes.remove(id).expect("Invalid node ID").data
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData {
        let EdgeId(from, to, index) = id;
        let node = self.nodes.get_mut(from).expect("Invalid 'from' node ID");
        let pos = node
            .edges_out
            .iter()
            .position(|e| e.target == *to && e.index == *index)
            .expect("Invalid edge ID");
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
