use std::{cell::UnsafeCell, fmt::Debug, marker::PhantomData, sync::Arc};

use crate::{
    debug::format_debug, directedness::Directedness, edge_ends::EdgeEnds,
    edge_multiplicity::EdgeMultiplicity, graph::AddEdgeResult, graph_id::GraphId, prelude::*,
    util::OtherValue,
};

mod edge_id;
mod node_id;

pub use edge_id::EdgeId;
pub use node_id::NodeId;

struct Node<N, E, D: DirectednessTrait> {
    data: N,
    edges_out: Vec<Arc<Edge<N, E, D>>>,
    // Only maintained for directed graphs, since for undirected graphs
    // edges_out is sufficient to find all edges.
    edges_in: Vec<EdgeId<N, E, D>>,
    directedness: PhantomData<D>,
}

struct Edge<N, E, D: DirectednessTrait> {
    data: UnsafeCell<E>,
    ends: EdgeEnds<NodeId<N, E, D>, D>,
    directedness: PhantomData<D>,
}

/// A graph representation using linked node and edge nodes.  Nodes and edges
/// are stored in insertion order.  Nodes and edge IDs remain valid until the
/// node or edge is removed.
///
/// # Type Parameters
/// * `N` - The type of data stored in nodes
/// * `E` - The type of data stored in edges
/// * `D` - The directedness ([`Directed`] or [`Undirected`](crate::Undirected))
pub struct LinkedGraph<N, E, D = Directedness, M = EdgeMultiplicity>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    nodes: Vec<Arc<Node<N, E, D>>>,
    id: GraphId,
    directedness: D,
    edge_multiplicity: M,
}

impl<N, E, D, M> LinkedGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    pub fn new(directedness: D, edge_multiplicity: M) -> Self {
        Self {
            nodes: Vec::new(),
            id: GraphId::new(),
            directedness,
            edge_multiplicity,
        }
    }

    fn node_id(&self, ptr: &Arc<Node<N, E, D>>) -> NodeId<N, E, D> {
        NodeId {
            ptr: Arc::downgrade(ptr),
            graph_id: self.id,
            directedness: PhantomData,
        }
    }

    fn edge_id(&self, ptr: &Arc<Edge<N, E, D>>) -> EdgeId<N, E, D> {
        EdgeId {
            ptr: Arc::downgrade(ptr),
            graph_id: self.id,
            directedness: self.directedness,
        }
    }

    fn node(&self, id: &NodeId<N, E, D>) -> &Node<N, E, D> {
        self.assert_valid_node_id(id);
        let id = id.ptr.upgrade().expect("NodeId is dangling");
        // SAFETY: We have checked that the NodeId is valid.  This method is only used internally
        // where we have &self, so the graph outlives the returned reference.
        unsafe { &*Arc::as_ptr(&id) }
    }

    /// Gets a mutable reference to the node with the given identifier.
    ///
    /// SAFETY: Caller must ensure that no other references to the node exist,
    /// and the graph outlives the returned reference.
    fn node_mut<'a>(&mut self, id: &NodeId<N, E, D>) -> &'a mut Node<N, E, D> {
        self.assert_valid_node_id(id);
        let id = id.ptr.upgrade().expect("NodeId is dangling");

        // SAFETY: We have checked that the NodeId is valid.  This method is only used internally
        // where we have &mut self, so no other references to the nodes can exist.
        unsafe { &mut *(Arc::as_ptr(&id) as *mut _) }
    }

    fn edge(&self, id: &EdgeId<N, E, D>) -> &Edge<N, E, D> {
        self.assert_valid_edge_id(id);
        let id = id.ptr.upgrade().expect("EdgeId is dangling");
        // SAFETY: We have checked that the EdgeId is valid.  This method is only used internally
        // where we have &self, so the graph outlives the returned reference.
        unsafe { &*Arc::as_ptr(&id) }
    }

    fn edge_mut(&mut self, id: &EdgeId<N, E, D>) -> &mut Edge<N, E, D> {
        self.assert_valid_edge_id(id);
        let id = id.ptr.upgrade().expect("EdgeId is dangling");
        // SAFETY: We have checked that the EdgeId is valid.  This method is only used internally
        // where we have &mut self, so no other references to the edges can exist.
        unsafe { &mut *(Arc::as_ptr(&id) as *mut _) }
    }
}

impl<N, E, D, M> Graph for LinkedGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    type NodeId = NodeId<N, E, D>;
    type NodeData = N;
    type EdgeId = EdgeId<N, E, D>;
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
        self.nodes.iter().map(|node| self.node_id(node))
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        let edge = self.edge(id);
        // SAFETY: There can be no mutable references to the data, the graph
        // owns all its data, and there are no mutable references to the graph.
        unsafe { &*edge.data.get() }
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        if self.is_directed() {
            // For directed graphs, just iterate normally
            self.nodes
                .iter()
                .flat_map(|node| node.edges_out.iter().map(|edge| self.edge_id(edge)))
                .collect::<Vec<_>>()
                .into_iter()
        } else {
            // For undirected graphs, deduplicate by Arc pointer address
            use std::collections::HashSet;
            let mut seen = HashSet::new();
            self.nodes
                .iter()
                .flat_map(|node| node.edges_out.iter())
                .filter(move |edge| seen.insert(Arc::as_ptr(edge)))
                .map(|edge| self.edge_id(edge))
                .collect::<Vec<_>>()
                .into_iter()
        }
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.node(from)
            .edges_out
            .iter()
            .map(|edge| self.edge_id(edge))
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        if self.is_directed() {
            // For directed graphs, use edges_in.  We need to collect to avoid
            // borrowing issues with self.node() below since edges_in contains
            // EdgeIds which borrow self.
            #[allow(clippy::unnecessary_to_owned)]
            self.node(into).edges_in.to_vec().into_iter()
        } else {
            // For undirected graphs, edges_into is the same as edges_from
            // since edges appear in both nodes' edges_out lists
            self.node(into)
                .edges_out
                .iter()
                .map(|edge| self.edge_id(edge))
                .collect::<Vec<_>>()
                .into_iter()
        }
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.node(from).edges_out.iter().filter_map(move |edge| {
            let (edge_source, edge_target) = edge.ends.values();
            let matches = if self.is_directed() {
                edge_source == from && edge_target == into
            } else {
                (edge_source == from && edge_target == into)
                    || (edge_source == into && edge_target == from)
            };
            matches.then(|| self.edge_id(edge))
        })
    }

    fn has_edge_from_into(&self, from: &Self::NodeId, into: &Self::NodeId) -> bool {
        self.node(from).edges_out.iter().any(|edge| {
            let (edge_source, edge_target) = edge.ends.values();
            if self.is_directed() {
                edge_source == from && edge_target == into
            } else {
                (edge_source == from && edge_target == into)
                    || (edge_source == into && edge_target == from)
            }
        })
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

    fn check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        if self.id != id.graph_id {
            return Err("NodeId graph_id does not match graph");
        }
        if id.ptr.upgrade().is_none() {
            return Err("NodeId is dangling");
        }
        Ok(())
    }

    fn maybe_check_valid_node_id(&self, id: &Self::NodeId) -> Result<(), &'static str> {
        #[cfg(not(feature = "unchecked"))]
        {
            self.check_valid_node_id(id)
        }
        #[cfg(feature = "unchecked")]
        {
            let _ = id;
            Ok(())
        }
    }

    fn check_valid_edge_id(&self, id: &Self::EdgeId) -> Result<(), &'static str> {
        if self.id != id.graph_id {
            return Err("EdgeId graph_id does not match graph");
        }
        if id.ptr.upgrade().is_none() {
            return Err("EdgeId is dangling");
        }
        Ok(())
    }

    fn maybe_check_valid_edge_id(&self, _id: &Self::EdgeId) -> Result<(), &'static str> {
        #[cfg(not(feature = "unchecked"))]
        {
            self.check_valid_edge_id(_id)
        }
        #[cfg(feature = "unchecked")]
        {
            Ok(())
        }
    }
}

impl<N, E, D, M> Default for LinkedGraph<N, E, D, M>
where
    D: DirectednessTrait + Default,
    M: EdgeMultiplicityTrait + Default,
{
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            id: GraphId::new(),
            directedness: D::default(),
            edge_multiplicity: M::default(),
        }
    }
}

impl<N, E, D, M> GraphMut for LinkedGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn node_data_mut(&mut self, id: &Self::NodeId) -> &mut Self::NodeData {
        &mut self.node_mut(id).data
    }

    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData {
        let edge = self.edge_mut(id);
        // SAFETY: There can be no mutable references to the data, the graph
        // owns all its data, and there are no mutable references to the graph.
        unsafe { &mut *edge.data.get() }
    }

    fn clear(&mut self) {
        self.nodes.clear();
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let node = Arc::new(Node {
            data,
            edges_out: Vec::new(),
            edges_in: Vec::new(),
            directedness: PhantomData,
        });
        let nid = self.node_id(&node);
        self.nodes.push(node);
        nid
    }

    fn add_edge(
        &mut self,
        from: &Self::NodeId,
        into: &Self::NodeId,
        data: Self::EdgeData,
    ) -> AddEdgeResult<Self::EdgeId, Self::EdgeData> {
        if !self.allows_parallel_edges() {
            debug_assert!(self.edges_from_into(from, into).count() <= 1);
            if let Some(edge) =
                self.node_mut(from).edges_out.iter_mut().find(|edge| {
                    edge.ends == self.directedness().make_pair(from.clone(), into.clone())
                })
            {
                let mut old_data = data;
                // SAFETY: There can be no mutable references to the data, the graph
                // owns all its data, and we have &mut self, so no other references
                // to the graph or edge data can exist.
                std::mem::swap(unsafe { &mut *edge.data.get() }, &mut old_data);
                return AddEdgeResult::Updated(old_data);
            }
            debug_assert_eq!(self.edges_from_into(from, into).count(), 0);
        }

        let ends = self.directedness().make_pair(from.clone(), into.clone());
        let (from, into) = ends.clone().into_values();

        let edge = Arc::new(Edge {
            data: UnsafeCell::new(data),
            ends,
            directedness: PhantomData,
        });
        let eid = self.edge_id(&edge);

        // Always add to the sorted "from" node's edges_out
        self.node_mut(&from).edges_out.push(edge.clone());

        if self.is_directed() {
            // For directed graphs, add to the "into" node's edges_in.  We don't
            // maintain edges_in for undirected graphs since it's redundant with
            // edges_out.
            self.node_mut(&into).edges_in.push(eid.clone());
        } else if from != into {
            // For undirected graphs (non-self-loop), add to the other node's edges_out
            self.node_mut(&into).edges_out.push(edge);
        }

        AddEdgeResult::Added(eid)
    }

    fn remove_node(&mut self, nid: &Self::NodeId) -> N {
        let index = self
            .nodes
            .iter()
            .position(|node| self.node_id(node) == *nid)
            .expect("Node does not exist");
        let node = self.nodes.remove(index);

        // Remove outgoing edges from other nodes
        for edge in &node.edges_out {
            // For undirected graphs, the "other" node could be either edge.from or edge.into
            match edge.ends.other_value(nid) {
                OtherValue::First(other_nid) | OtherValue::Second(other_nid) => {
                    let other_node = self.node_mut(other_nid);
                    if self.is_directed() {
                        // For directed graphs, remove from edges_in
                        other_node.edges_in.retain(|eid| *eid != self.edge_id(edge));
                    } else {
                        // For undirected graphs, remove from edges_out
                        other_node
                            .edges_out
                            .retain(|e| self.edge_id(e) != self.edge_id(edge));
                    }
                }
                OtherValue::Both(_) => {}
            };
        }

        if self.is_directed() {
            // For directed graphs, also remove incoming edges from source nodes' edges_out
            for eid in &node.edges_in {
                let edge = self.edge(eid);
                let from_nid = edge.ends.source();
                if *from_nid != *nid {
                    let from_node = self.node_mut(&from_nid.clone());
                    from_node
                        .edges_out
                        .retain(|edge| self.edge_id(edge) != *eid);
                }
            }
        }

        Arc::into_inner(node)
            .expect("Node has multiple references")
            .data
    }

    fn remove_edge(&mut self, eid: &Self::EdgeId) -> Self::EdgeData {
        self.assert_valid_edge_id(eid);
        let edge = eid.ptr.upgrade().expect("EdgeId is dangling");
        let (from_nid, into_nid) = edge.ends.values();

        // Remove from source node's edges_out
        let from_node = self.node_mut(from_nid);
        from_node
            .edges_out
            .retain(|edge| *eid != self.edge_id(edge));

        if self.is_directed() {
            // For directed graphs, remove from target node's edges_in
            let to_node = self.node_mut(into_nid);
            to_node.edges_in.retain(|eid2| *eid != *eid2);
        } else if from_nid != into_nid {
            // For undirected graphs (non-self-loop), remove from target node's edges_out
            let to_node = self.node_mut(into_nid);
            to_node.edges_out.retain(|edge| *eid != self.edge_id(edge));
        }

        Arc::into_inner(edge)
            .expect("Edge has multiple references")
            .data
            .into_inner()
    }
}

impl<N, E, D, M> Clone for LinkedGraph<N, E, D, M>
where
    N: Clone,
    E: Clone,
    D: DirectednessTrait + Default,
    M: EdgeMultiplicityTrait + Default,
{
    fn clone(&self) -> Self {
        let mut new_graph = LinkedGraph::default();
        new_graph.copy_from(self);
        new_graph
    }
}

impl<N, E, D, M> Debug for LinkedGraph<N, E, D, M>
where
    N: Debug,
    E: Debug,
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "LinkedGraph")
    }
}
