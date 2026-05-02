use std::{collections::HashMap, fmt::Debug};

use graphitude::{
    EdgeIdTrait, NodeIdTrait,
    format_debug::format_debug,
    graph_id::{GraphId, GraphIdClone},
    graph_test_suite,
    graph_tests::{GraphTests, TestDataBuilder},
    prelude::*,
};

#[derive(Default)]
struct StringGraph {
    nodes: HashMap<usize, Node>,
    next_node_id: usize,
    next_edge_id: usize,
    graph_id: GraphId,
}

impl Clone for StringGraph {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            next_node_id: self.next_node_id,
            next_edge_id: self.next_edge_id,
            graph_id: GraphId::default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
struct NodeId {
    index: usize,
    graph_id: GraphIdClone,
}

impl NodeIdTrait for NodeId {}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct EdgeId {
    source: NodeId,
    target: NodeId,
    index: usize,
}

impl EdgeIdTrait for EdgeId {
    type NodeId = NodeId;
    type Directedness = Directed;

    fn directedness(&self) -> Self::Directedness {
        Directed
    }

    fn left(&self) -> NodeId {
        self.source
    }

    fn right(&self) -> NodeId {
        self.target
    }
}

#[derive(Clone, Debug)]
struct Node {
    data: String,
    edges_out: Vec<Edge>,
}

#[derive(Clone, Debug)]
struct Edge {
    target: usize,
    data: String,
    // The index is necessary to uniquely identify edges between the same pair of nodes.
    index: usize,
}

impl StringGraph {
    fn node(&self, id: &NodeId) -> &Node {
        self.nodes.get(&id.index).expect("Invalid node ID")
    }

    fn node_mut(&mut self, id: &NodeId) -> &mut Node {
        self.nodes.get_mut(&id.index).expect("Invalid node ID")
    }

    fn edge(&self, id: &EdgeId) -> &Edge {
        self.nodes
            .get(&id.source.index)
            .expect("Invalid edge ID")
            .edges_out
            .iter()
            .find(|e| e.target == id.target.index && e.index == id.index)
            .expect("Invalid edge ID")
    }

    fn edge_mut(&mut self, id: &EdgeId) -> &mut Edge {
        self.nodes
            .get_mut(&id.source.index)
            .expect("Invalid edge ID")
            .edges_out
            .iter_mut()
            .find(|e| e.target == id.target.index && e.index == id.index)
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
        self.nodes.keys().map(|index| NodeId {
            index: *index,
            graph_id: self.graph_id.clone(),
        })
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.nodes.iter().flat_map(move |(from_id, node)| {
            node.edges_out.iter().map(move |edge| EdgeId {
                source: NodeId {
                    index: *from_id,
                    graph_id: self.graph_id.clone(),
                },
                target: NodeId {
                    index: edge.target,
                    graph_id: self.graph_id.clone(),
                },
                index: edge.index,
            })
        })
    }

    // Override these methods to provide O(1) validation instead of the default
    // O(n) iteration. This is necessary to make
    // test_deconstruct_large_graph_by_nodes reasonably fast, as it validates
    // all nodes and edges after every compaction.

    fn check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        if self.graph_id != id.graph_id {
            return Err("NodeId does not belong to this graph");
        }
        if !self.nodes.contains_key(&id.index) {
            return Err("NodeId not found in graph");
        }
        Ok(())
    }

    fn check_valid_edge_id(&self, id: &Self::EdgeId) -> Result<(), &'static str> {
        self.check_valid_node_id(&id.source)?;
        self.check_valid_node_id(&id.target)?;
        Ok(())
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
        let id = NodeId {
            index: self.next_node_id,
            graph_id: self.graph_id.clone(),
        };
        self.next_node_id += 1;
        self.nodes.insert(
            id.index,
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
        assert!(self.nodes.contains_key(&to.index), "Invalid 'to' node ID");
        let edge_index = self.next_edge_id;
        self.next_edge_id += 1;
        self.nodes
            .get_mut(&from.index)
            .expect("Invalid 'from' node ID")
            .edges_out
            .push(Edge {
                target: to.index,
                data,
                index: edge_index,
            });
        AddEdgeResult::Added(EdgeId {
            source: *from,
            target: *to,
            index: edge_index,
        })
    }

    fn remove_node(&mut self, id: &Self::NodeId) -> Self::NodeData {
        for node in self.nodes.values_mut() {
            node.edges_out.retain(|e| e.target != id.index);
        }
        self.nodes.remove(&id.index).expect("Invalid node ID").data
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Self::EdgeData {
        let EdgeId {
            source: from,
            target: to,
            index,
        } = id;
        let node = self
            .nodes
            .get_mut(&from.index)
            .expect("Invalid 'from' node ID");
        let pos = node
            .edges_out
            .iter()
            .position(|e| e.target == to.index && e.index == *index)
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

graph_test_suite!(
    tests: GraphTests<StringGraphBuilder> = GraphTests::new(
        StringGraphBuilder,
        |data| format!("{}-copied", data),
        |data| format!("{}-copied", data)
    )
);
