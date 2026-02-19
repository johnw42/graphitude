use std::{collections::HashMap, fmt::Debug};

use derivative::Derivative;
use graphitude::{
    EdgeIdTrait, NodeIdTrait, format_debug::format_debug, graph_tests,
    graph_tests::TestDataBuilder, prelude::*,
};

#[derive(Default, Derivative)]
#[derivative(Clone(bound = ""))]
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

    fn directedness(&self) -> Self::Directedness {
        Directed
    }

    fn left(&self) -> NodeId {
        self.0
    }

    fn right(&self) -> NodeId {
        self.1
    }
}

#[derive(Clone, Debug)]
struct Node {
    data: String,
    edges_out: Vec<Edge>,
}

#[derive(Clone, Debug)]
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

    fn node_mut(&mut self, id: &NodeId) -> &mut Node {
        self.nodes.get_mut(id).expect("Invalid node ID")
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

    fn edge_mut(&mut self, id: &EdgeId) -> &mut Edge {
        self.nodes
            .get_mut(&id.0)
            .expect("Invalid edge ID")
            .edges_out
            .iter_mut()
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

    fn directedness(&self) -> Self::Directedness {
        Directed
    }

    fn edge_multiplicity(&self) -> Self::EdgeMultiplicity {
        MultipleEdges
    }

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
                .map(move |edge| EdgeId(*from_id, edge.target, edge.index))
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

    fn is_very_slow(&self) -> bool {
        true
    }
}

impl GraphMut for StringGraph {
    fn new(_directedness: Self::Directedness, _edge_multiplicity: Self::EdgeMultiplicity) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }

    fn node_data_mut(&mut self, id: &Self::NodeId) -> &mut Self::NodeData {
        &mut self.node_mut(id).data
    }

    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData {
        &mut self.edge_mut(id).data
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

    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        to: &Self::NodeId,
        data: Self::EdgeData,
    ) -> AddEdgeResult<Self::EdgeId, Self::EdgeData> {
        assert!(self.nodes.contains_key(to), "Invalid 'to' node ID");
        let edge_index = self.next_edge_id;
        self.next_edge_id += 1;
        self.nodes
            .get_mut(from)
            .expect("Invalid 'from' node ID")
            .edges_out
            .push(Edge {
                target: *to,
                data,
                index: edge_index,
            });
        AddEdgeResult::Added(EdgeId(*from, *to, edge_index))
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

        node.edges_out.remove(pos).data
    }
}

impl Debug for StringGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "StringGraph")
    }
}

#[derive(Default)]
struct StringGraphBuilder;

impl TestDataBuilder for StringGraphBuilder {
    type Graph = StringGraph;

    fn new_graph(&self) -> Self::Graph {
        StringGraph::default()
    }

    fn new_edge_data(&self, i: usize) -> String {
        format!("e{}", i)
    }

    fn new_node_data(&self, i: usize) -> String {
        format!("v{}", i)
    }
}

graph_tests!(
    tests,
    StringGraphBuilder,
    StringGraphBuilder,
    |data| format!("{}-copied", data),
    |data| format!("{}-copied", data)
);
