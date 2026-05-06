use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use crate::{
    bag::{Bag, BagKey},
    coordinate_pair::CoordinatePair,
    copier::GraphCopier,
    directedness::Directedness,
    edge_multiplicity::EdgeMultiplicity,
    format_debug::format_debug,
    graph_id::GraphId,
    graph_traits::AddEdgeResult,
    map_collector::MapCollector,
    prelude::*,
    util::OtherValue,
};

mod edge_id;
mod node_id;

use derivative::Derivative;
pub use edge_id::EdgeId;
pub use node_id::NodeId;

struct Node<G: Graph> {
    data: G::NodeData,
    edges_out: Vec<BagKey>,
    // Only maintained for directed graphs, since for undirected graphs
    // edges_out is sufficient to find all edges.
    edges_in: Vec<BagKey>,
}

struct Edge<G: Graph> {
    data: G::EdgeData,
    ends: CoordinatePair<BagKey, G::Directedness>,
}

impl<G: Graph> Edge<G> {
    fn new(data: G::EdgeData, from: BagKey, into: BagKey, directedness: G::Directedness) -> Self {
        Self {
            data,
            ends: CoordinatePair::new(from, into, directedness),
        }
    }
}

/// A graph representation using linked node and edge nodes.  Nodes and edges
/// are stored in insertion order.  Nodes and edge IDs remain valid until the
/// node or edge is removed.
///
/// # Type Parameters
/// * `N` - The type of data stored in nodes
/// * `E` - The type of data stored in edges
/// * `D` - The directedness ([`Directed`] or [`Undirected`](crate::Undirected))
#[derive(Derivative)]
#[derivative(Default(bound = "D: Default, M: Default"))]
pub struct BagGraph<N, E, D = Directedness, M = EdgeMultiplicity>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    nodes: Bag<Node<Self>>,
    edges: Bag<Edge<Self>>,
    id: GraphId,
    directedness: D,
    edge_multiplicity: M,
}

impl<N, E, D, M> BagGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn node_id(&self, key: BagKey) -> NodeId<Self> {
        NodeId {
            key,
            graph_id: self.id.clone(),
            graph: PhantomData,
        }
    }

    fn edge_id(&self, edge_key: BagKey) -> EdgeId<Self> {
        EdgeId {
            edge_key,
            node_keys: self.edges[edge_key].ends.clone(),
            graph_id: self.id.clone(),
            directedness: self.directedness,
        }
    }

    fn node(&self, id: &NodeId<Self>) -> &Node<Self> {
        &self.nodes[id.key]
    }

    /// Gets a mutable reference to the node with the given identifier.
    ///
    /// SAFETY: Caller must ensure that no other references to the node exist,
    /// and the graph outlives the returned reference.
    fn node_mut(&mut self, id: &NodeId<Self>) -> &mut Node<Self> {
        &mut self.nodes[id.key]
    }

    fn edge(&self, id: &EdgeId<Self>) -> &Edge<Self> {
        &self.edges[id.edge_key]
    }

    fn edge_mut(&mut self, id: &EdgeId<Self>) -> &mut Edge<Self> {
        &mut self.edges[id.edge_key]
    }
}

impl<N, E, D, M> Graph for BagGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    type NodeId = NodeId<Self>;
    type NodeData = N;
    type EdgeId = EdgeId<Self>;
    type EdgeData = E;
    type Directedness = D;
    type EdgeMultiplicity = M;

    fn directedness(&self) -> Self::Directedness {
        self.directedness
    }

    fn edge_multiplicity(&self) -> Self::EdgeMultiplicity {
        self.edge_multiplicity
    }

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        &self.node(id).data
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.keys().map(|node| self.node_id(node))
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        &self.edge(id).data
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.edges.pairs().map(|(edge_key, edge)| EdgeId {
            edge_key,
            node_keys: edge.ends.clone(),
            graph_id: self.id.clone(),
            directedness: self.directedness,
        })
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.node(from)
            .edges_out
            .iter()
            .map(|edge_key| self.edge_id(*edge_key))
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.node(into)
            .edges_in
            .iter()
            .map(|edge_key| self.edge_id(*edge_key))
            .chain(self.edges_from(into).take_while(|_| !self.is_directed()))
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        let expected_ends = CoordinatePair::new(from.key, into.key, self.directedness());
        self.node(from)
            .edges_out
            .iter()
            .filter_map(move |edge_key| {
                let edge = self.edge(&self.edge_id(*edge_key));
                let matches = edge.ends == expected_ends;
                matches.then(|| self.edge_id(*edge_key))
            })
    }

    fn has_edge_from_into(&self, from: &Self::NodeId, into: &Self::NodeId) -> bool {
        self.edges_from_into(from, into).next().is_some()
    }

    fn num_edges_into(&self, into: &Self::NodeId) -> usize {
        if self.is_directed() {
            self.node(into).edges_in.len()
        } else {
            self.node(into).edges_out.len()
        }
    }

    fn num_edges_from(&self, from: &Self::NodeId) -> usize {
        self.node(from).edges_out.len()
    }
}

impl<N, E, D, M> GraphMut for BagGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn new(directedness: D, edge_multiplicity: M) -> Self {
        Self {
            nodes: Bag::new(),
            edges: Bag::new(),
            id: GraphId::default(),
            directedness,
            edge_multiplicity,
        }
    }

    fn node_data_mut(&mut self, id: &Self::NodeId) -> &mut Self::NodeData {
        &mut self.node_mut(id).data
    }

    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData {
        &mut self.edge_mut(id).data
    }

    fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let node_key = self.nodes.insert(Node {
            data,
            edges_out: Vec::new(),
            edges_in: Vec::new(),
        });
        self.node_id(node_key)
    }

    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> AddEdgeResult<Self::EdgeId, Self::EdgeData> {
        let ends = self.directedness.coordinate_pair((from.key, into.key));

        if !self.allows_parallel_edges() {
            debug_assert!(self.num_edges_from_into(from, into) <= 1);
            if let Some(edge_key) = self.nodes[from.key]
                .edges_out
                .iter_mut()
                .find(|edge_key| self.edges[**edge_key].ends == ends)
            {
                let mut old_data = data;
                std::mem::swap(&mut self.edges[*edge_key].data, &mut old_data);
                return AddEdgeResult::Updated(
                    EdgeId {
                        edge_key: *edge_key,
                        node_keys: ends,
                        graph_id: self.id.clone(),
                        directedness: self.directedness,
                    },
                    old_data,
                );
            }
            debug_assert_eq!(self.num_edges_from_into(from, into), 0);
        }

        let (from, into) = ends.values();

        let edge_key = self.edges.insert(Edge::new(
            data,
            from.clone(),
            into.clone(),
            self.directedness(),
        ));

        let eid = self.edge_id(edge_key);

        self.nodes[*from].edges_out.push(edge_key);

        if self.is_directed() {
            // For directed graphs, add to the "into" node's edges_in.  We don't
            // maintain edges_in for undirected graphs since it's redundant with
            // edges_out.
            self.nodes[*into].edges_in.push(edge_key);
        } else if from != into {
            // For undirected graphs (non-self-loop), add to the other node's edges_out
            self.nodes[*into].edges_out.push(edge_key);
        }

        AddEdgeResult::Added(eid)
    }

    fn remove_node(&mut self, nid: &Self::NodeId) -> N {
        let node_key = nid.key;
        let node = self.nodes.remove(node_key).expect("NodeId is invalid");
        let is_directed = self.is_directed();

        // Remove outgoing edges from other nodes
        for edge_key in &node.edges_out {
            // For undirected graphs, the "other" node could be either edge.from or edge.into
            let edge = &self.edges[*edge_key];
            match edge.ends.other_value(&node_key) {
                OtherValue::First(other_node_key) | OtherValue::Second(other_node_key) => {
                    let other_node = &mut self.nodes[*other_node_key];
                    if is_directed {
                        // For directed graphs, remove from edges_in
                        other_node.edges_in.retain(|key| *key != *edge_key);
                    } else {
                        // For undirected graphs, remove from edges_out
                        other_node.edges_out.retain(|key| *key != *edge_key);
                    }
                }
                OtherValue::Both(_) => {}
            };
        }

        if is_directed {
            // For directed graphs, also remove incoming edges from source nodes' edges_out
            for edge_key in &node.edges_in {
                let edge = &self.edges[*edge_key];
                let from_key = edge.ends.first();
                if *from_key != node_key {
                    let from_node = &mut self.nodes[*from_key];
                    from_node.edges_out.retain(|key| *key != *edge_key);
                }
            }
        }

        node.data
    }

    fn remove_edge(&mut self, eid: &Self::EdgeId) -> Self::EdgeData {
        let edge_key = eid.edge_key;
        let edge = self.edges.remove(edge_key).expect("EdgeId is invalid");
        let (from_key, into_key) = edge.ends.values();

        // Remove from source node's edges_out
        let from_node = &mut self.nodes[*from_key];
        from_node.edges_out.retain(|&key| key != edge_key);

        if self.is_directed() {
            // For directed graphs, remove from target node's edges_in
            let to_node = &mut self.nodes[*into_key];
            to_node.edges_in.retain(|&key| key != edge_key);
        } else if from_key != into_key {
            // For undirected graphs (non-self-loop), remove from target node's edges_out
            let to_node = &mut self.nodes[*into_key];
            to_node.edges_out.retain(|&key| key != edge_key);
        }

        edge.data
    }

    fn compact(
        &mut self,
        node_map_collector: Option<&mut dyn MapCollector<Self::NodeId>>,
        edge_map_collector: Option<&mut dyn MapCollector<Self::EdgeId>>,
    ) {
        let mut node_map = HashMap::with_capacity(self.nodes.len());
        let mut edge_map = HashMap::with_capacity(self.edges.len());
        self.nodes.compact(Some(&mut node_map));
        self.edges.compact(Some(&mut edge_map));
        for node in self.nodes.iter_mut() {
            node.edges_out = node
                .edges_out
                .iter()
                .map(|edge_key| edge_map[edge_key])
                .collect();
            node.edges_in = node
                .edges_in
                .iter()
                .map(|edge_key| edge_map[edge_key])
                .collect();
        }
        for edge in self.edges.iter_mut() {
            edge.ends = edge.ends.map(|node_key| node_map[&node_key]);
        }
        if let Some(node_map_collector) = node_map_collector {
            for (old_key, new_key) in node_map {
                node_map_collector.insert(self.node_id(old_key), self.node_id(new_key));
            }
        }
        if let Some(edge_map_collector) = edge_map_collector {
            for (old_key, new_key) in edge_map {
                edge_map_collector.insert(self.edge_id(old_key), self.edge_id(new_key));
            }
        }
    }
}

impl<N, E, D, M> Clone for BagGraph<N, E, D, M>
where
    N: Clone,
    E: Clone,
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn clone(&self) -> Self {
        GraphCopier::new(self).clone_nodes().clone_edges().copy()
    }
}

impl<N, E, D, M> Debug for BagGraph<N, E, D, M>
where
    N: Debug,
    E: Debug,
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "BagGraph")
    }
}
