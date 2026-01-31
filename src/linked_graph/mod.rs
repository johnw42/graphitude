use std::{fmt::Debug, marker::PhantomData, sync::Arc};

use crate::{
    Directed, Graph, GraphMut, debug::format_debug, directedness::Directedness, graph_id::GraphId,
};

mod edge_id;
mod node_id;

pub use edge_id::EdgeId;
pub use node_id::NodeId;

struct Node<N, E, D> {
    data: N,
    edges_out: Vec<Arc<Edge<N, E, D>>>,
    edges_in: Vec<EdgeId<N, E, D>>,
    directedness: PhantomData<D>,
}

struct Edge<N, E, D> {
    data: E,
    from: NodeId<N, E, D>,
    into: NodeId<N, E, D>,
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
pub struct LinkedGraph<N, E, D = Directed>
where
    D: Directedness,
{
    nodes: Vec<Arc<Node<N, E, D>>>,
    id: GraphId,
    directedness: PhantomData<D>,
}

impl<N: Debug, E, D> LinkedGraph<N, E, D>
where
    D: Directedness,
{
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
            directedness: PhantomData,
        }
    }

    fn node(&self, id: NodeId<N, E, D>) -> &Node<N, E, D> {
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
    unsafe fn node_mut<'a>(&mut self, id: NodeId<N, E, D>) -> &'a mut Node<N, E, D> {
        self.assert_valid_node_id(&id);
        let id = id.ptr.upgrade().expect("NodeId is dangling");
        unsafe { &mut *(Arc::as_ptr(&id) as *mut _) }
    }

    fn edge(&self, id: EdgeId<N, E, D>) -> &Edge<N, E, D> {
        self.assert_valid_edge_id(&id);
        let id = id.ptr.upgrade().expect("EdgeId is dangling");
        // SAFETY: We have checked that the EdgeId is valid, and the graph
        // contains all strong references to its edges.
        unsafe { &*Arc::as_ptr(&id) }
    }
}

impl<N: Debug, E, D> Graph for LinkedGraph<N, E, D>
where
    D: Directedness,
{
    type NodeId = NodeId<N, E, D>;
    type NodeData = N;
    type EdgeId = EdgeId<N, E, D>;
    type EdgeData = E;
    type Directedness = D;

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
        if D::is_directed() {
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

impl<N: Debug, E, D> GraphMut for LinkedGraph<N, E, D>
where
    D: Directedness,
{
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            id: GraphId::new(),
            directedness: PhantomData,
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
            directedness: PhantomData,
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
        let (from_sorted, into_sorted) = D::maybe_sort(from.clone(), into.clone());

        let edge = Arc::new(Edge {
            data,
            from: from_sorted.clone(),
            into: into_sorted.clone(),
            directedness: PhantomData,
        });
        let eid = self.edge_id(&edge);

        // SAFETY: Calling node_mut is safe here because we have &mut self,
        // so no other references to the nodes can exist.
        // TODO: Make sure this is true!
        unsafe {
            // Always add to the sorted "from" node's edges_out
            self.node_mut(from_sorted.clone())
                .edges_out
                .push(edge.clone());

            // Always add to the sorted "into" node's edges_in
            self.node_mut(into_sorted.clone())
                .edges_in
                .push(eid.clone());

            // For undirected graphs, also add to the reverse direction
            // but only if it's not a self-loop
            if !D::is_directed() && from_sorted != into_sorted {
                // Add to the other node's edges_out as well (for edges_from to work)
                self.node_mut(into_sorted.clone()).edges_out.push(edge);
                // Add to the other node's edges_in as well
                self.node_mut(from_sorted).edges_in.push(eid.clone());
            }
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

        // Remove outgoing edges from target nodes' edges_in
        for edge in &node.edges_out {
            let to_nid = edge.into.clone();
            if to_nid != nid {
                let to_node = unsafe { self.node_mut(to_nid) };
                to_node.edges_in.retain(|eid| *eid != self.edge_id(edge));

                // For undirected graphs, also remove from edges_out
                if !D::is_directed() {
                    to_node
                        .edges_out
                        .retain(|e| self.edge_id(e) != self.edge_id(edge));
                }
            }
        }

        // Remove incoming edges from source nodes' edges_out
        for eid in &node.edges_in {
            let edge = self.edge(eid.clone());
            let from_nid = edge.from.clone();
            if from_nid != nid {
                let from_node = unsafe { self.node_mut(from_nid) };
                from_node
                    .edges_out
                    .retain(|edge| self.edge_id(edge) != *eid);

                // For undirected graphs, also remove from edges_in
                if !D::is_directed() {
                    from_node.edges_in.retain(|e| *e != *eid);
                }
            }
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

        // Remove from source node's edges_out
        let from_node = unsafe { self.node_mut(from_nid.clone()) };
        from_node.edges_out.retain(|edge| eid != self.edge_id(edge));

        // Remove from target node's edges_in
        let to_node = unsafe { self.node_mut(to_nid.clone()) };
        to_node.edges_in.retain(|eid2| eid != *eid2);

        // For undirected graphs, also remove reverse references
        if !D::is_directed() && from_nid != to_nid {
            let to_node = unsafe { self.node_mut(to_nid) };
            to_node.edges_out.retain(|edge| eid != self.edge_id(edge));

            let from_node = unsafe { self.node_mut(from_nid) };
            from_node.edges_in.retain(|eid2| eid != *eid2);
        }

        Arc::into_inner(edge)
            .expect("Edge has multiple references")
            .data
    }
}

impl<N, E, D> Clone for LinkedGraph<N, E, D>
where
    N: Clone + Debug,
    E: Clone,
    D: Directedness,
{
    fn clone(&self) -> Self {
        let mut new_graph = LinkedGraph::new();
        new_graph.copy_from(self);
        new_graph
    }
}

impl<N, E, D> Debug for LinkedGraph<N, E, D>
where
    N: Debug,
    E: Debug,
    D: Directedness,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_debug(self, f, "LinkedGraph")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::TestDataBuilder, *};

    mod directed {
        use super::*;

        impl TestDataBuilder for LinkedGraph<i32, String, Directed> {
            type Graph = Self;

            fn new_edge_data(i: usize) -> String {
                format!("e{}", i)
            }

            fn new_node_data(i: usize) -> i32 {
                i as i32
            }
        }

        graph_tests!(LinkedGraph<i32, String, Directed>);
        graph_test_copy_from_with!(
            LinkedGraph<i32, String, Directed>,
            |data| data * 2,
            |data| format!("{}-copied", data));
    }

    mod undirected {
        use super::*;

        impl TestDataBuilder for LinkedGraph<i32, String, Undirected> {
            type Graph = Self;

            fn new_edge_data(i: usize) -> String {
                format!("e{}", i)
            }

            fn new_node_data(i: usize) -> i32 {
                i as i32
            }
        }

        graph_tests!(LinkedGraph<i32, String, Undirected>);
        graph_test_copy_from_with!(
            LinkedGraph<i32, String, Undirected>,
            |data| data * 2,
            |data| format!("{}-copied", data));
    }
}
