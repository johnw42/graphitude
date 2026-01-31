use std::{
    fmt::Debug,
    sync::Arc,
};

use crate::{Graph, GraphMut, debug::format_debug, directedness::Directed, graph_id::GraphId};

mod node_id;
mod edge_id;

pub use node_id::NodeId;
pub use edge_id::EdgeId;

struct Node<N, E> {
    data: N,
    edges_out: Vec<Arc<Edge<N, E>>>,
    edges_in: Vec<EdgeId<N, E>>,
}

struct Edge<N, E> {
    data: E,
    from: NodeId<N, E>,
    into: NodeId<N, E>,
}

/// A graph representation using linked node and edge nodes.  Nodes and edges
/// are stored in insertion order.  Nodes and edge IDs remain valid until the
/// node or edge is removed.
pub struct LinkedGraph<N, E> {
    nodes: Vec<Arc<Node<N, E>>>,
    id: GraphId,
}

impl<N: Debug, E> LinkedGraph<N, E> {
    fn node_id(&self, ptr: &Arc<Node<N, E>>) -> NodeId<N, E> {
        NodeId {
            ptr: Arc::downgrade(ptr),
            graph_id: self.id,
        }
    }

    fn edge_id(&self, ptr: &Arc<Edge<N, E>>) -> EdgeId<N, E> {
        EdgeId {
            ptr: Arc::downgrade(ptr),
            graph_id: self.id,
        }
    }

    fn node(&self, id: NodeId<N, E>) -> &Node<N, E> {
        self.assert_valid_node_id(&id);
        let id = id.ptr.upgrade().expect("NodeId is dangling");
        // SAFETY: We have checked that the NodeId is valid, and the graph
        // contains all strong references to its nodes.
        unsafe { &*Arc::as_ptr(&id) }
    }

    /// Gets a mutable reference to the node with the given identifier.
    ///
    /// SAFETY: Caller must ensure that no other references to the node exist,
    /// and the graph outlives the returned reference.
    unsafe fn node_mut<'a>(&mut self, id: NodeId<N, E>) -> &'a mut Node<N, E> {
        self.assert_valid_node_id(&id);
        let id = id.ptr.upgrade().expect("NodeId is dangling");
        unsafe { &mut *(Arc::as_ptr(&id) as *mut _) }
    }

    fn edge(&self, id: EdgeId<N, E>) -> &Edge<N, E> {
        self.assert_valid_edge_id(&id);
        let id = id.ptr.upgrade().expect("EdgeId is dangling");
        // SAFETY: We have checked that the EdgeId is valid, and the graph
        // contains all strong references to its edges.
        unsafe { &*Arc::as_ptr(&id) }
    }
}

impl<N: Debug, E> Graph for LinkedGraph<N, E> {
    type NodeId = NodeId<N, E>;
    type NodeData = N;
    type EdgeId = EdgeId<N, E>;
    type EdgeData = E;
    type Directedness = Directed;

    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData {
        &self.node(id).data
    }

    /// Gets an iterator over all node identifiers in the graph in insertion order.
    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.iter().map(|node| self.node_id(node))
    }

    fn edge_data(&self, id: Self::EdgeId) -> &Self::EdgeData {
        &self.edge(id).data
    }

    /// Gets an iterator over all edge identifiers in the graph in insertion order of the
    /// source nodes and the insertion order of the edges from each source node.
    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.nodes
            .iter()
            .flat_map(|node| node.edges_out.iter().map(|edge| self.edge_id(edge)))
    }

    /// Gets an iterator over the edges outgoing from the given node in
    /// insertion order of the edges outgoing from the given node.
    fn edges_from(&self, from: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> {
        self.node(from)
            .edges_out
            .iter()
            .map(|edge| self.edge_id(edge))
    }

    /// Gets an iterator over the edges incoming to the given node in
    /// insertion order of the edges incoming to the given node.
    fn edges_into(&self, into: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> {
        self.node(into).edges_in.iter().cloned()
    }

    fn num_edges_into(&self, into: Self::NodeId) -> usize {
        self.node(into).edges_in.len()
    }

    fn num_edges_from(&self, from: Self::NodeId) -> usize {
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

impl<N: Debug, E> GraphMut for LinkedGraph<N, E> {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            id: GraphId::new(),
        }
    }

    fn clear(&mut self) {
        self.nodes.clear();
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let node = Arc::new(Node {
            data,
            edges_out: Vec::new(),
            edges_in: Vec::new(),
        });
        let nid = self.node_id(&node);
        self.nodes.push(node);
        nid
    }

    fn add_or_replace_edge(
        &mut self,
        from: Self::NodeId,
        into: Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let edge = Arc::new(Edge {
            data,
            from: from.clone(),
            into: into.clone(),
        });
        let eid = self.edge_id(&edge);

        unsafe {
            self.node_mut(from).edges_out.push(edge);
            self.node_mut(into).edges_in.push(eid.clone());
        }

        (eid, None)
    }

    fn remove_node(&mut self, nid: Self::NodeId) -> N {
        let index = self
            .nodes
            .iter()
            .position(|node| self.node_id(node) == nid)
            .expect("Node does not exist");
        let node = self.nodes.remove(index);
        for edge in &node.edges_out {
            let to_nid = edge.into.clone();
            let to_node = unsafe { self.node_mut(to_nid) };
            to_node.edges_in.retain(|eid| *eid != self.edge_id(edge));
        }
        for eid in &node.edges_in {
            let edge = self.edge(eid.clone());
            let from_nid = edge.from.clone();
            let from_node = unsafe { self.node_mut(from_nid) };
            from_node
                .edges_out
                .retain(|edge| self.edge_id(edge) != *eid);
        }
        Arc::into_inner(node)
            .expect("Node has multiple references")
            .data
    }

    fn remove_edge(&mut self, eid: Self::EdgeId) -> Self::EdgeData {
        self.assert_valid_edge_id(&eid);
        let edge = eid.ptr.upgrade().expect("EdgeId is dangling");
        let from_nid = edge.from.clone();
        let to_nid = edge.into.clone();

        let from_node = unsafe { self.node_mut(from_nid) };
        from_node.edges_out.retain(|edge| eid != self.edge_id(edge));

        let to_node = unsafe { self.node_mut(to_nid) };
        to_node.edges_in.retain(|eid2| eid != *eid2);

        Arc::into_inner(edge)
            .expect("Edge has multiple references")
            .data
    }
}

impl<N, E> Clone for LinkedGraph<N, E>
where
    N: Clone + Debug,
    E: Clone,
{
    fn clone(&self) -> Self {
        let mut new_graph = LinkedGraph::new();
        new_graph.copy_from(self);
        new_graph
    }
}

impl<N, E> Debug for LinkedGraph<N, E>
where
    N: Debug,
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "LinkedGraph")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::TestDataBuilder, *};

    impl TestDataBuilder for LinkedGraph<i32, String> {
        type Graph = Self;

        fn new_edge_data(i: usize) -> String {
            format!("e{}", i)
        }

        fn new_node_data(i: usize) -> i32 {
            i as i32
        }
    }

    graph_tests!(LinkedGraph<i32, String>);
    graph_test_copy_from_with!(
        LinkedGraph<i32, String>,
        |data| data * 2,
        |data| format!("{}-copied", data));
}
