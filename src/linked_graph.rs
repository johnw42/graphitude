use std::{fmt::Debug, hash::Hash};

use crate::{Graph, GraphMut, debug::format_debug, directedness::Directed};

struct NodeNode<N, E> {
    data: N,
    edges_out: Vec<Box<EdgeNode<N, E>>>,
    edges_in: Vec<EdgeId<N, E>>,
}

#[derive(PartialOrd, Ord)]
pub struct NodeId<N, E>(*mut NodeNode<N, E>);

impl<N, E> Debug for NodeId<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NodeId({:?})", self.0)
    }
}

impl<N, E> Clone for NodeId<N, E> {
    fn clone(&self) -> Self {
        NodeId(self.0)
    }
}

impl<N, E> Copy for NodeId<N, E> {}

impl<N, E> PartialEq for NodeId<N, E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<N, E> Eq for NodeId<N, E> {}

impl<N, E> Hash for NodeId<N, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state);
    }
}

impl<N, E> From<&NodeNode<N, E>> for NodeId<N, E> {
    fn from(ptr: &NodeNode<N, E>) -> Self {
        NodeId(ptr as *const _ as *mut _)
    }
}

struct EdgeNode<N, E> {
    data: E,
    from: NodeId<N, E>,
    into: NodeId<N, E>,
}

#[derive(PartialOrd, Ord)]
pub struct EdgeId<N, E>(*mut EdgeNode<N, E>);

impl<N, E> Debug for EdgeId<N, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EdgeId({:?})", self.0)
    }
}

impl<N, E> Clone for EdgeId<N, E> {
    fn clone(&self) -> Self {
        EdgeId(self.0)
    }
}

impl<N, E> Copy for EdgeId<N, E> {}

impl<N, E> PartialEq for EdgeId<N, E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<N, E> Eq for EdgeId<N, E> {}

impl<N, E> Hash for EdgeId<N, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state);
    }
}

impl<N, E> From<&EdgeNode<N, E>> for EdgeId<N, E> {
    fn from(ptr: &EdgeNode<N, E>) -> Self {
        EdgeId(ptr as *const _ as *mut _)
    }
}

impl<N, E> From<&Box<EdgeNode<N, E>>> for EdgeId<N, E> {
    fn from(ebox: &Box<EdgeNode<N, E>>) -> Self {
        EdgeId::from(&**ebox)
    }
}

/// A graph representation using linked node and edge nodes.
/// Nodes and edges are stored in insertion order.
pub struct LinkedGraph<N, E> {
    nodes: Vec<Box<NodeNode<N, E>>>,
}

impl<N, E> LinkedGraph<N, E> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }
}

impl<N, E> Graph for LinkedGraph<N, E> {
    type NodeId<'g> = NodeId<N, E> where N: 'g, E: 'g;
    type NodeData = N;
    type EdgeId<'g> = EdgeId<N, E> where N: 'g, E: 'g;
    type EdgeData = E;
    type Directedness = Directed;

    fn node_data<'a>(&'a self, id: Self::NodeId<'a>) -> &Self::NodeData {
        unsafe { &(*id.0).data }
    }

    /// Gets an iterator over all node identifiers in the graph in insertion order.
    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId<'_>> {
        self.nodes.iter().map(|node| NodeId::from(&**node))
    }

    fn edge_data(&self, id: Self::EdgeId<'_>) -> &Self::EdgeData {
        unsafe { &(*id.0).data }
    }

    /// Gets an iterator over all edge identifiers in the graph in insertion order of the
    /// source nodes and the insertion order of the edges from each source node.
    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId<'_>> {
        self.nodes
            .iter()
            .flat_map(|vnode| vnode.edges_out.iter().map(|enode| EdgeId::from(&**enode)))
    }

    fn edge_ends<'a>(&'a self, eid: Self::EdgeId<'a>) -> (Self::NodeId<'_>, Self::NodeId<'_>) {
        self.check_edge_id(&eid);
        let edge_node = unsafe { &*eid.0 };
        (edge_node.from, edge_node.into)
    }

    /// Gets an iterator over the edges outgoing from the given node in
    /// insertion order of the edges outgoing from the given node.
    fn edges_from<'a>(&'a self, from: Self::NodeId<'a>) -> impl Iterator<Item = Self::EdgeId<'a>> + 'a {
        self.check_node_id(&from);
        unsafe { &*from.0 }
            .edges_out
            .iter()
            .map(|enode| EdgeId::from(&**enode))
    }

    /// Gets an iterator over the edges incoming to the given node in
    /// insertion order of the edges incoming to the given node.
    fn edges_into<'a>(&'a self, into: Self::NodeId<'a>) -> impl Iterator<Item = Self::EdgeId<'a>> + 'a {
        self.check_node_id(&into);
        unsafe { &*into.0 }.edges_in.iter().cloned()
    }

    fn num_edges_into<'a>(&'a self, into: Self::NodeId<'a>) -> usize {
        self.check_node_id(&into);
        unsafe { &*into.0 }.edges_in.len()
    }

    fn num_edges_from<'a>(&'a self, from: Self::NodeId<'a>) -> usize {
        self.check_node_id(&from);
        unsafe { &*from.0 }.edges_out.len()
    }
}

impl<N, E> GraphMut for LinkedGraph<N, E> {
    fn clear(&mut self) {
        self.nodes.clear();
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId<'_> {
        let vnode = Box::new(NodeNode {
            data,
            edges_out: Vec::new(),
            edges_in: Vec::new(),
        });
        let nid = NodeId::from(&*vnode);
        self.nodes.push(vnode);
        nid
    }

    fn add_or_replace_edge(
        &mut self,
        from: Self::NodeId<'_>,
        into: Self::NodeId<'_>,
        data: Self::EdgeData,
    ) -> (Self::EdgeId<'_>, Option<Self::EdgeData>) {
        let enode = Box::new(EdgeNode { data, from, into });
        let eid = EdgeId::from(&*enode);

        unsafe {
            (&mut *from.0).edges_out.push(enode);
            (&mut *into.0).edges_in.push(eid);
        }

        (eid, None)
    }

    fn remove_node(&mut self, nid: Self::NodeId<'_>) -> N {
        let index = self
            .nodes
            .iter()
            .position(|vnode| NodeId::from(&**vnode) == nid)
            .expect("Node does not exist");
        let vnode = self.nodes.remove(index);
        for enode in &vnode.edges_out {
            let to_nid = enode.into;
            let to_vnode = unsafe { &mut *to_nid.0 };
            to_vnode.edges_in.retain(|&eid| eid != EdgeId::from(enode));
        }
        for eid in &vnode.edges_in {
            let enode = unsafe { &*eid.0 };
            let from_nid = enode.from;
            let from_vnode = unsafe { &mut *from_nid.0 };
            from_vnode
                .edges_out
                .retain(|enode| EdgeId::from(enode) != *eid);
        }
        vnode.data
    }

    fn remove_edge(&mut self, eid: Self::EdgeId<'_>) -> Option<Self::EdgeData> {
        let enode = unsafe { &*eid.0 };
        let from_nid = enode.from;
        let to_nid = enode.into;

        let from_vnode = unsafe { &mut *from_nid.0 };
        from_vnode
            .edges_out
            .retain(|enode| eid != EdgeId::from(enode));

        let to_vnode = unsafe { &mut *to_nid.0 };
        to_vnode.edges_in.retain(|&eid2| eid != eid2);

        Some(unsafe { Box::from_raw(eid.0).data })
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
