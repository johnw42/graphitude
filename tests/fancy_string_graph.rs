use std::collections::HashMap;

use jrw_graph::{
    Directed, Graph, GraphMut, graph_test_copy_from_with, graph_tests, tests::TestDataBuilder,
};

struct StringGraph {
    nodes: HashMap<NodeId, Node>,
    next_node_id: usize,
}

type NodeId = usize;
type EdgeId = (NodeId, NodeId);

struct Node {
    data: String,
    edges_out: Vec<Edge>,
}

struct Edge {
    target: NodeId,
    data: String,
}

impl StringGraph {
    fn new() -> Self {
        StringGraph {
            nodes: HashMap::new(),
            next_node_id: 0,
        }
    }

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

    fn node_data(&self, id: Self::NodeId<'_>) -> &Self::NodeData {
        &self.node(&id).data
    }

    fn edge_data(&self, id: Self::EdgeId<'_>) -> &Self::EdgeData {
        &self.edge(&id).data
    }

    fn edge_source(&self, id: Self::EdgeId<'_>) -> Self::NodeId<'_> {
        id.0
    }

    fn edge_target(&self, id: Self::EdgeId<'_>) -> Self::NodeId<'_> {
        id.1
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId<'_>> {
        self.nodes.keys().cloned()
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId<'_>> {
        self.nodes.iter().flat_map(|(from_id, node)| {
            node
                .edges_out
                .iter()
                .map(move |edge| (from_id.clone(), edge.target.clone()))
        })
    }

    fn edge_ends(&self, eid: Self::EdgeId<'_>) -> (Self::NodeId<'_>, Self::NodeId<'_>) {
        eid
    }
}

impl GraphMut for StringGraph {
    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId<'_> {
        let id = self.next_node_id;
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
        from: Self::NodeId<'_>,
        to: Self::NodeId<'_>,
        data: Self::EdgeData,
    ) -> (Self::EdgeId<'_>, Option<Self::EdgeData>) {
        assert!(self.nodes.contains_key(&to), "Invalid 'to' node ID");
        self.nodes
            .get_mut(&from)
            .expect("Invalid 'from' node ID")
            .edges_out
            .push(Edge {
                target: to.clone(),
                data: data,
            });
        ((from, to), None)
    }

    fn remove_node(&mut self, id: Self::NodeId<'_>) -> Self::NodeData {
        self.nodes
            .remove(&id)
            .map(|v| v.data)
            .expect("Invalid node ID")
    }

    fn remove_edge(&mut self, (from, to): Self::EdgeId<'_>) -> Option<Self::EdgeData> {
        let node = self.nodes.get_mut(&from)?;
        if let Some(pos) = node.edges_out.iter().position(|e| e.target == to) {
            let edge = node.edges_out.remove(pos);
            Some(edge.data)
        } else {
            None
        }
    }
}

impl TestDataBuilder for StringGraph {
    type Graph = Self;

    fn new_graph() -> Self::Graph {
        Self::new()
    }

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
