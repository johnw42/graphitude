use std::{
    fmt::Debug,
    hash::Hash,
    sync::{Arc, Weak},
};

use crate::{Graph, GraphMut, debug::format_debug, directedness::Directed, graph_id::GraphId};

struct Node<N, E> {
    data: N,
    edges_out: Vec<Arc<Edge<N, E>>>,
    edges_in: Vec<EdgeId<N, E>>,
}

/// Node identifier for [`LinkedGraph`].
///
/// Contains a weak pointer to the node data and a graph ID for safety checks.
pub struct NodeId<N, E> {
    ptr: Weak<Node<N, E>>,
    graph_id: GraphId,
}

impl<N, E> Debug for NodeId<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.ptr)
    }
}

impl<N, E> Clone for NodeId<N, E> {
    fn clone(&self) -> Self {
        NodeId {
            ptr: Weak::clone(&self.ptr),
            graph_id: self.graph_id,
        }
    }
}

impl<N, E> PartialEq for NodeId<N, E> {
    fn eq(&self, other: &Self) -> bool {
        assert_eq!(self.graph_id, other.graph_id);
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<N, E> Eq for NodeId<N, E> {}

impl<N, E> Hash for NodeId<N, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.ptr.as_ptr() as usize).hash(state);
    }
}

impl<N, E> PartialOrd for NodeId<N, E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ptr.as_ptr().cmp(&other.ptr.as_ptr()))
    }
}

impl<N, E> Ord for NodeId<N, E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<N, E> crate::graph::NodeId for NodeId<N, E> {}

struct Edge<N, E> {
    data: E,
    from: NodeId<N, E>,
    into: NodeId<N, E>,
}

/// Edge identifier for [`LinkedGraph`].
///
/// Contains a weak pointer to the edge data and a graph ID for safety checks.
pub struct EdgeId<N, E> {
    ptr: Weak<Edge<N, E>>,
    graph_id: GraphId,
}

impl<N, E> Debug for EdgeId<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.ptr)
    }
}

impl<N, E> Clone for EdgeId<N, E> {
    fn clone(&self) -> Self {
        EdgeId {
            ptr: Weak::clone(&self.ptr),
            graph_id: self.graph_id,
        }
    }
}

impl<N, E> PartialEq for EdgeId<N, E> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.as_ptr() == other.ptr.as_ptr()
    }
}

impl<N, E> Eq for EdgeId<N, E> {}

impl<N, E> Hash for EdgeId<N, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.ptr.as_ptr() as usize).hash(state);
    }
}

impl<N, E> PartialOrd for EdgeId<N, E> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.ptr.as_ptr().cmp(&other.ptr.as_ptr()))
    }
}

impl<N, E> Ord for EdgeId<N, E> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.as_ptr().cmp(&other.ptr.as_ptr())
    }
}

impl<N: Debug, E> crate::graph::EdgeId for EdgeId<N, E> {
    type NodeId = NodeId<N, E>;

    fn source(&self) -> NodeId<N, E> {
        self.ptr
            .upgrade()
            .map(|edge| NodeId {
                ptr: Arc::downgrade(&edge.from.ptr.upgrade().expect("Source node dangling")),
                graph_id: self.graph_id,
            })
            .expect("EdgeId is dangling")
    }

    fn target(&self) -> NodeId<N, E> {
        self.ptr
            .upgrade()
            .map(|edge| NodeId {
                ptr: Arc::downgrade(&edge.into.ptr.upgrade().expect("Target node dangling")),
                graph_id: self.graph_id,
            })
            .expect("EdgeId is dangling")
    }
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
