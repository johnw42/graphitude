#![cfg(feature = "bitvec")]

use std::{collections::HashSet, fmt::Debug};

use graphitude::{
    AdjacencyMatrix, EdgeId as EdgeIdTrait, NodeId as NodeIdTrait, SymmetricHashAdjacencyMatrix,
    debug::format_debug_with, graph_test_copy_from_with, graph_tests, prelude::*,
    tests::TestDataBuilder,
};

/// An undirected graph where nodes are identified by strings.  A node's ID
/// is the same as its data.  Edges have no data and are identified by the pair
/// of node IDs they connect.
struct StringGraph {
    nodes: HashSet<NodeId>,
    edges: SymmetricHashAdjacencyMatrix<NodeId, ()>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct NodeId(String);

impl NodeIdTrait for NodeId {}

// Invariant: `EdgeId` always has the smaller `NodeId` first.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
struct EdgeId(NodeId, NodeId);

impl EdgeId {
    fn new(from: NodeId, into: NodeId) -> Self {
        if from <= into {
            EdgeId(from, into)
        } else {
            EdgeId(into, from)
        }
    }
}

impl EdgeIdTrait for EdgeId {
    type NodeId = NodeId;
    type Directedness = Undirected;

    fn source(&self) -> NodeId {
        self.0.clone()
    }

    fn target(&self) -> NodeId {
        self.1.clone()
    }
}

impl StringGraph {
    fn edge_id(
        &self,
        from: <StringGraph as Graph>::NodeId,
        into: <StringGraph as Graph>::NodeId,
    ) -> <StringGraph as Graph>::EdgeId {
        assert!(self.nodes.contains(&from));
        debug_assert!(self.nodes.contains(&into));
        EdgeId::new(from, into)
    }
}

impl Graph for StringGraph {
    type NodeData = String;
    type NodeId = NodeId;
    type EdgeData = ();
    type EdgeId = EdgeId;
    type Directedness = Undirected;
    type EdgeMultiplicity = SingleEdge;

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        &self.nodes.get(id).expect("Node does not exist").0
    }

    fn num_edges_between(&self, from: &Self::NodeId, into: &Self::NodeId) -> usize {
        self.edges
            .get(from.clone(), into.clone())
            .into_iter()
            .count()
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        self.edges
            .get(id.0.clone(), id.1.clone())
            .expect("Edge does not exist")
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.iter().cloned()
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.edges
            .iter()
            .map(|(from, into, _)| EdgeId::new(from, into))
    }

    // Override these methods to provide O(1) validation instead of the default O(n)
    // iteration. This is necessary to make test_deconstruct_large_graph_by_nodes
    // reasonably fast, as it validates all nodes and edges after every compaction.

    fn check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        if self.nodes.contains(id) {
            Ok(())
        } else {
            Err("NodeId not found in graph")
        }
    }

    fn check_valid_edge_id(&self, id: &Self::EdgeId) -> Result<(), &'static str> {
        if self.edges.get(id.0.clone(), id.1.clone()).is_some() {
            Ok(())
        } else {
            Err("EdgeId not found in graph")
        }
    }
}

impl GraphMut for StringGraph {
    fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: SymmetricHashAdjacencyMatrix::new(),
        }
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let id = NodeId(data);
        self.nodes.insert(id.clone());
        id
    }

    fn add_or_replace_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let old_data = self.edges.insert(from.clone(), into.clone(), data);
        (self.edge_id(from.clone(), into.clone()), old_data)
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> String {
        let edges_from = self
            .edges
            .entries_in_row(id.clone())
            .map(|(into, _)| into)
            .collect::<Vec<_>>();
        for into in edges_from {
            self.edges.remove(id.clone(), into);
        }
        self.nodes.remove(id);
        id.0.clone()
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData {
        self.edges
            .remove(id.0.clone(), id.1.clone())
            .expect("Edge does not exist")
    }
}

impl Debug for StringGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug_with(
            self,
            f,
            "StringGraph",
            &mut |nid| format!("{:?}", nid.0),
            &|_| (),
            &|_| (),
        )
    }
}

impl TestDataBuilder for StringGraph {
    type Graph = Self;

    fn new_edge_data(_i: usize) -> () {
        ()
    }

    fn new_node_data(i: usize) -> String {
        format!("v{}", i)
    }
}

graph_tests!(StringGraph);
graph_test_copy_from_with!(StringGraph, |data| format!("{}-copied", data), |_| ());

#[test]
fn test_format_debug_with() {
    let mut graph = StringGraph::new();
    // Add nodes in non-sorted order.
    let n1 = graph.add_node("B".to_string());
    let n2 = graph.add_node("A".to_string());
    graph.add_edge(&n1, &n2, ());

    // Single-line output - nodes show data (not zero-sized), edges don't (zero-sized)
    let output = format!("{:?}", &graph);
    let expected = r#"StringGraph { nodes: ["A", "B"], edges: ["A" -- "B"] }"#;
    assert_eq!(output, expected);

    // Multi-line output.
    let output = format!("{:#?}", &graph);
    let expected = r#"StringGraph {
    nodes: [
        "A",
        "B",
    ],
    edges: [
        "A" -- "B",
    ],
}"#;
    assert_eq!(output, expected);
}

#[test]
fn test_edge_id_ordering() {
    let edge1 = EdgeId::new(NodeId("Z".to_string()), NodeId("A".to_string()));
    let edge2 = EdgeId::new(NodeId("A".to_string()), NodeId("Z".to_string()));
    assert_eq!(edge1, edge2);
    assert_eq!(edge1.0, NodeId("A".to_string()));
}
