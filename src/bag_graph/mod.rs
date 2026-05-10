use crate::{
    bag::{Bag, BagKey},
    copier::GraphCopier,
    end_pair::EndPair,
    format_debug::format_debug,
    map_collector::MapCollector,
    prelude::*,
};
use derivative::Derivative;
use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

mod ids;

pub use ids::{BagGraphEdgeId, BagGraphNodeId};

struct Node<G: Graph> {
    data: G::NodeData,
    edges_out: Vec<BagKey>,
    // Only maintained for directed graphs, since for undirected graphs
    // edges_out is sufficient to find all edges.
    edges_in: Vec<BagKey>,
}

struct Edge<G: Graph> {
    data: G::EdgeData,
    ends: <G::Directedness as Directedness>::EndPair<BagKey>,
}

impl<G: Graph> Edge<G> {
    fn new(data: G::EdgeData, from: BagKey, into: BagKey) -> Self {
        Self {
            data,
            ends: G::Directedness::make_pair(from, into),
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
#[derivative(Default(bound = "D: Default"))]
pub struct BagGraph<N, E, D>
where
    D: Directedness,
{
    nodes: Bag<Node<Self>>,
    edges: Bag<Edge<Self>>,
    directedness: PhantomData<D>,
}

impl<N, E, D> BagGraph<N, E, D>
where
    D: Directedness,
{
    fn node(&self, id: &BagGraphNodeId<Self>) -> &Node<Self> {
        &self.nodes[id.key()]
    }

    fn node_mut(&mut self, id: &BagGraphNodeId<Self>) -> &mut Node<Self> {
        &mut self.nodes[id.key()]
    }

    fn edge(&self, id: &BagGraphEdgeId<Self>) -> &Edge<Self> {
        &self.edges[id.key()]
    }

    fn edge_mut(&mut self, id: &BagGraphEdgeId<Self>) -> &mut Edge<Self> {
        &mut self.edges[id.key()]
    }
}

impl<N, E, D> Graph for BagGraph<N, E, D>
where
    D: Directedness,
{
    type NodeId = BagGraphNodeId<Self>;
    type NodeData = N;
    type EdgeId = BagGraphEdgeId<Self>;
    type EdgeData = E;
    type Directedness = D;
    type EdgeMultiplicity = MultipleEdges;

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        &self.node(id).data
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.keys().map(BagGraphNodeId::new)
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        &self.edge(id).data
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.edges.keys().map(BagGraphEdgeId::new)
    }

    fn edge_ends(
        &self,
        id: &Self::EdgeId,
    ) -> <Self::Directedness as Directedness>::EndPair<Self::NodeId> {
        let edge = self.edge(id);
        let (from_key, into_key) = edge.ends.values();
        D::make_pair(
            BagGraphNodeId::new(*from_key),
            BagGraphNodeId::new(*into_key),
        )
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.node(from)
            .edges_out
            .iter()
            .copied()
            .map(BagGraphEdgeId::new)
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.node(into)
            .edges_in
            .iter()
            .copied()
            .map(BagGraphEdgeId::new)
            .chain(self.edges_from(into).take_while(|_| !self.is_directed()))
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        let expected_ends = D::make_pair(from.key(), into.key());
        self.node(from)
            .edges_out
            .iter()
            .filter_map(move |edge_key| {
                let edge = &self.edges[*edge_key];
                let matches = edge.ends == expected_ends;
                matches.then(|| BagGraphEdgeId::new(*edge_key))
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

impl<N, E, D> GraphMut for BagGraph<N, E, D>
where
    D: Directedness,
{
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
        BagGraphNodeId::new(node_key)
    }

    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<(Self::EdgeId, Self::EdgeData)>) {
        let ends = D::make_pair(from.key(), into.key());

        if !self.allows_parallel_edges() {
            debug_assert!(self.num_edges_from_into(from, into) <= 1);
            if let Some(edge_key) = self.nodes[from.key()]
                .edges_out
                .iter_mut()
                .find(|edge_key| self.edges[**edge_key].ends == ends)
            {
                let mut old_data = data;
                std::mem::swap(&mut self.edges[*edge_key].data, &mut old_data);
                let edge_id = BagGraphEdgeId {
                    key: *edge_key,
                    phantom: PhantomData,
                };
                return (edge_id.clone(), Some((edge_id, old_data)));
            }
            debug_assert_eq!(self.num_edges_from_into(from, into), 0);
        }

        let (from, into) = ends.values();

        let edge_key = self
            .edges
            .insert(Edge::new(data, from.clone(), into.clone()));

        let eid = BagGraphEdgeId::new(edge_key);

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

        (eid, None)
    }

    fn remove_node(&mut self, nid: &Self::NodeId) -> N {
        let node_key = nid.key();
        let node = self.nodes.remove(node_key).expect("NodeId is invalid");
        let is_directed = self.is_directed();

        if is_directed {
            for edge_key in &node.edges_out {
                let edge = &self.edges[*edge_key];
                let &other_node_key = edge.ends.other_value(&node_key).into_inner();
                if other_node_key != node_key {
                    let other_node = &mut self.nodes[other_node_key];
                    other_node.edges_out.retain(|key| *key != *edge_key);
                    self.edges.remove(*edge_key);
                }
            }
            for edge_key in &node.edges_in {
                let edge = &self.edges[*edge_key];
                let &other_node_key = edge.ends.other_value(&node_key).into_inner();
                if other_node_key != node_key {
                    let other_node = &mut self.nodes[other_node_key];
                    other_node.edges_out.retain(|key| *key != *edge_key);
                }
                self.edges.remove(*edge_key);
            }
        } else {
            for edge_key in &node.edges_out {
                let edge = &self.edges[*edge_key];
                let &other_node_key = edge.ends.other_value(&node_key).into_inner();
                if other_node_key != node_key {
                    let other_node = &mut self.nodes[other_node_key];
                    other_node.edges_out.retain(|key| *key != *edge_key);
                }
                self.edges.remove(*edge_key);
            }
        }

        node.data
    }

    fn remove_edge(&mut self, eid: &Self::EdgeId) -> Self::EdgeData {
        let edge_key = eid.key();
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
            edge.ends = edge.ends.clone().map(|node_key| node_map[&node_key]);
        }
        if let Some(node_map_collector) = node_map_collector {
            for (old_key, new_key) in node_map {
                node_map_collector
                    .insert(BagGraphNodeId::new(old_key), BagGraphNodeId::new(new_key));
            }
        }
        if let Some(edge_map_collector) = edge_map_collector {
            for (old_key, new_key) in edge_map {
                edge_map_collector
                    .insert(BagGraphEdgeId::new(old_key), BagGraphEdgeId::new(new_key));
            }
        }
    }
}

impl<N, E, D> Clone for BagGraph<N, E, D>
where
    N: Clone,
    E: Clone,
    D: Directedness,
{
    fn clone(&self) -> Self {
        GraphCopier::new(self).clone_nodes().clone_edges().copy()
    }
}

impl<N, E, D> Debug for BagGraph<N, E, D>
where
    N: Debug,
    E: Debug,
    D: Directedness,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "BagGraph")
    }
}
