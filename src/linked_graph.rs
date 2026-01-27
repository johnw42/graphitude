use std::{fmt::Debug, hash::Hash};

use crate::{Graph, GraphMut, debug::format_debug, directedness::Directed};

struct NodeNode<V, E> {
    data: V,
    edges_out: Vec<Box<EdgeNode<V, E>>>,
    edges_in: Vec<EdgeId<V, E>>,
}

#[derive(PartialOrd, Ord)]
pub struct NodeId<V, E>(*mut NodeNode<V, E>);

impl<V, E> Debug for NodeId<V, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.0)
    }
}

impl<V, E> Clone for NodeId<V, E> {
    fn clone(&self) -> Self {
        NodeId(self.0)
    }
}

impl<V, E> Copy for NodeId<V, E> {}

impl<V, E> PartialEq for NodeId<V, E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V, E> Eq for NodeId<V, E> {}

impl<V, E> Hash for NodeId<V, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state);
    }
}

impl<V, E> From<&NodeNode<V, E>> for NodeId<V, E> {
    fn from(ptr: &NodeNode<V, E>) -> Self {
        NodeId(ptr as *const _ as *mut _)
    }
}

struct EdgeNode<V, E> {
    data: E,
    from: NodeId<V, E>,
    into: NodeId<V, E>,
}

#[derive(PartialOrd, Ord)]
pub struct EdgeId<V, E>(*mut EdgeNode<V, E>);

impl<V, E> Debug for EdgeId<V, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.0)
    }
}

impl<V, E> Clone for EdgeId<V, E> {
    fn clone(&self) -> Self {
        EdgeId(self.0)
    }
}

impl<V, E> Copy for EdgeId<V, E> {}

impl<V, E> PartialEq for EdgeId<V, E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V, E> Eq for EdgeId<V, E> {}

impl<V, E> Hash for EdgeId<V, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state);
    }
}

impl<V, E> From<&EdgeNode<V, E>> for EdgeId<V, E> {
    fn from(ptr: &EdgeNode<V, E>) -> Self {
        EdgeId(ptr as *const _ as *mut _)
    }
}

impl<V, E> From<&Box<EdgeNode<V, E>>> for EdgeId<V, E> {
    fn from(ebox: &Box<EdgeNode<V, E>>) -> Self {
        EdgeId::from(&**ebox)
    }
}

/// A graph representation using linked node and edge nodes.
/// Nodes and edges are stored in insertion order.
pub struct LinkedGraph<V, E> {
    nodes: Vec<Box<NodeNode<V, E>>>,
}

impl<V, E> LinkedGraph<V, E> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }
}

impl<V, E> Graph for LinkedGraph<V, E> {
    type NodeId = NodeId<V, E>;
    type NodeData = V;
    type EdgeId = EdgeId<V, E>;
    type EdgeData = E;
    type Directedness = Directed;

    fn node_data(&self, id: Self::NodeId) -> &Self::NodeData {
        unsafe { &(*id.0).data }
    }

    /// Gets an iterator over all node identifiers in the graph in insertion order.
    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.iter().map(|node| NodeId::from(&**node))
    }

    fn edge_data(&self, id: Self::EdgeId) -> &Self::EdgeData {
        unsafe { &(*id.0).data }
    }

    /// Gets an iterator over all edge identifiers in the graph in insertion order of the
    /// source nodes and the insertion order of the edges from each source node.
    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.nodes
            .iter()
            .flat_map(|vnode| vnode.edges_out.iter().map(|enode| EdgeId::from(&**enode)))
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::NodeId, Self::NodeId) {
        self.check_edge_id(&eid);
        let edge_node = unsafe { &*eid.0 };
        (edge_node.from, edge_node.into)
    }

    /// Gets an iterator over the edges outgoing from the given node in
    /// insertion order of the edges outgoing from the given node.
    fn edges_from(&self, from: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> {
        self.check_node_id(&from);
        unsafe { &*from.0 }
            .edges_out
            .iter()
            .map(|enode| EdgeId::from(&**enode))
    }

    /// Gets an iterator over the edges incoming to the given node in
    /// insertion order of the edges incoming to the given node.
    fn edges_into(&self, into: Self::NodeId) -> impl Iterator<Item = Self::EdgeId> {
        self.check_node_id(&into);
        unsafe { &*into.0 }.edges_in.iter().cloned()
    }

    fn num_edges_into(&self, into: Self::NodeId) -> usize {
        self.check_node_id(&into);
        unsafe { &*into.0 }.edges_in.len()
    }

    fn num_edges_from(&self, from: Self::NodeId) -> usize {
        self.check_node_id(&from);
        unsafe { &*from.0 }.edges_out.len()
    }
}

impl<V, E> GraphMut for LinkedGraph<V, E> {
    fn clear(&mut self) {
        self.nodes.clear();
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let vnode = Box::new(NodeNode {
            data,
            edges_out: Vec::new(),
            edges_in: Vec::new(),
        });
        let vid = NodeId::from(&*vnode);
        self.nodes.push(vnode);
        vid
    }

    fn add_or_replace_edge(
        &mut self,
        from: Self::NodeId,
        into: Self::NodeId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let enode = Box::new(EdgeNode { data, from, into });
        let eid = EdgeId::from(&*enode);

        unsafe {
            (&mut *from.0).edges_out.push(enode);
            (&mut *into.0).edges_in.push(eid);
        }

        (eid, None)
    }

    fn remove_node(&mut self, vid: Self::NodeId) -> V {
        let index = self
            .nodes
            .iter()
            .position(|vnode| NodeId::from(&**vnode) == vid)
            .expect("Node does not exist");
        let vnode = self.nodes.remove(index);
        for enode in &vnode.edges_out {
            let to_vid = enode.into;
            let to_vnode = unsafe { &mut *to_vid.0 };
            to_vnode.edges_in.retain(|&eid| eid != EdgeId::from(enode));
        }
        for eid in &vnode.edges_in {
            let enode = unsafe { &*eid.0 };
            let from_vid = enode.from;
            let from_vnode = unsafe { &mut *from_vid.0 };
            from_vnode
                .edges_out
                .retain(|enode| EdgeId::from(enode) != *eid);
        }
        vnode.data
    }

    fn remove_edge(&mut self, eid: Self::EdgeId) -> Option<Self::EdgeData> {
        let enode = unsafe { &*eid.0 };
        let from_vid = enode.from;
        let to_vid = enode.into;

        let from_vnode = unsafe { &mut *from_vid.0 };
        from_vnode
            .edges_out
            .retain(|enode| eid != EdgeId::from(enode));

        let to_vnode = unsafe { &mut *to_vid.0 };
        to_vnode.edges_in.retain(|&eid2| eid != eid2);

        Some(unsafe { Box::from_raw(eid.0).data })
    }
}

impl<V, E> Debug for LinkedGraph<V, E>
where
    V: Debug,
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

        fn new_graph() -> Self::Graph {
            Self::new()
        }

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
