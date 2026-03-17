use std::{
    cell::{RefCell, UnsafeCell},
    collections::HashSet,
    fmt::Debug,
    marker::PhantomData,
    rc::Rc,
};

use crate::{
    coordinate_pair::CoordinatePair, copier::GraphCopier, directedness::Directedness,
    edge_multiplicity::EdgeMultiplicity, format_debug::format_debug, graph_id::GraphId,
    graph_traits::AddEdgeResult, prelude::*, util::OtherValue,
};

mod edge_id;
mod node_id;

use derivative::Derivative;
pub use edge_id::EdgeId;
pub use node_id::NodeId;

struct Node<N, E, D: DirectednessTrait> {
    data: UnsafeCell<N>,
    // For undirected graphs, `edges` contains all edges incident to the node.
    // For directed graphs, `edges` contains only outgoing edges, and `back_edges`
    // contains incoming edges.
    edges: RefCell<Vec<Rc<Edge<N, E, D>>>>,
    // Only maintained for directed graphs, since for undirected graphs
    // `edges` is sufficient to find all edges.
    back_edges: RefCell<Vec<EdgeId<N, E, D>>>,
    directedness: PhantomData<D>,
}

struct Edge<N, E, D: DirectednessTrait> {
    data: UnsafeCell<E>,
    ends: CoordinatePair<NodeId<N, E, D>, D>,
    directedness: PhantomData<D>,
}

impl<N, E, D: DirectednessTrait> Edge<N, E, D> {
    fn new(data: E, from: NodeId<N, E, D>, into: NodeId<N, E, D>, directedness: D) -> Self {
        Self {
            data: UnsafeCell::new(data),
            ends: CoordinatePair::new(from, into, directedness),
            directedness: PhantomData,
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
///
/// # Saftey
///
/// This graph uses `Rc` and `RefCell` to allow multiple references to nodes and
/// edges while still allowing mutation.  To ensure safety, all access to node
/// and edge data is done through the `Graph` and `GraphMut` traits, which
/// require a valid `NodeId` or `EdgeId`.  The `NodeId` and `EdgeId` types
/// contain a `graph_id` field that must match the graph's ID, and a weak
/// pointer to the node or edge data.  This ensures that if a `NodeId` or
/// `EdgeId` is used after the graph has been dropped, it will fail gracefully
/// instead of causing undefined behavior.  Additionally, the graph's methods
/// ensure that there are no mutable references to the graph or its data while
/// any `NodeId` or `EdgeId` is in use, so there can be no aliasing mutable
/// references to the data.
#[derive(Derivative)]
#[derivative(Default(bound = "D: Default, M: Default"))]
pub struct LinkedGraph<N, E, D = Directedness, M = EdgeMultiplicity>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    nodes: Vec<Rc<Node<N, E, D>>>,
    id: GraphId,
    directedness: D,
    edge_multiplicity: M,
}

impl<N, E, D, M> LinkedGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn node_id(&self, ptr: &Rc<Node<N, E, D>>) -> NodeId<N, E, D> {
        NodeId {
            ptr: Rc::downgrade(ptr),
            graph_id: self.id.clone(),
            directedness: PhantomData,
        }
    }

    fn edge_id(&self, ptr: &Rc<Edge<N, E, D>>) -> EdgeId<N, E, D> {
        EdgeId {
            ptr: Rc::downgrade(ptr),
            graph_id: self.id.clone(),
            directedness: self.directedness,
        }
    }

    fn node(&self, id: &NodeId<N, E, D>) -> &Node<N, E, D> {
        self.assert_valid_node_id(id);
        let strong = id.ptr.upgrade().expect("NodeId is dangling");
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &*Rc::as_ptr(&strong) }
    }

    fn edge(&self, id: &EdgeId<N, E, D>) -> &Edge<N, E, D> {
        self.assert_valid_edge_id(id);
        let strong = id.ptr.upgrade().expect("EdgeId is dangling");
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &*Rc::as_ptr(&strong) }
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
        let node = self.node(id);
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &*node.data.get() }
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.iter().map(|node| self.node_id(node))
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        let edge = self.edge(id);
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &*edge.data.get() }
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        let mut seen = if self.is_directed() {
            None
        } else {
            Some(HashSet::<*const _>::new())
        };
        self.nodes.iter().flat_map(move |node| {
            node.edges
                .borrow()
                .iter()
                .filter(|edge| {
                    if let Some(seen) = &mut seen {
                        seen.insert(Rc::as_ptr(edge))
                    } else {
                        true
                    }
                })
                .map(|edge| self.edge_id(edge))
                .collect::<Vec<_>>()
        })
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.node(from)
            .edges
            .borrow()
            .iter()
            .map(|edge| self.edge_id(edge))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        if self.is_directed() {
            // For directed graphs, use edges_in.  We need to collect to avoid
            // borrowing issues with self.node() below since edges_in contains
            // EdgeIds which borrow self.
            self.node(into).back_edges.borrow().clone()
        } else {
            // For undirected graphs, edges_into is the same as edges_from
            // since edges appear in both nodes' edges_out lists
            self.node(into)
                .edges
                .borrow()
                .iter()
                .map(|edge| self.edge_id(edge))
                .collect::<Vec<_>>()
        }
        .into_iter()
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.node(from)
            .edges
            .borrow()
            .iter()
            .filter(move |edge| edge.ends.other_value(from).into_inner() == into)
            .map(|edge| self.edge_id(edge))
            .collect::<Vec<_>>()
            .into_iter()
    }

    fn has_edge_from_into(&self, from: &Self::NodeId, into: &Self::NodeId) -> bool {
        self.node(from)
            .edges
            .borrow()
            .iter()
            .any(|edge| edge.ends.other_value(from).into_inner() == into)
    }

    fn num_edges_into(&self, into: &Self::NodeId) -> usize {
        if self.is_directed() {
            self.node(into).back_edges.borrow().len()
        } else {
            self.node(into).edges.borrow().len()
        }
    }

    fn num_edges_from(&self, from: &Self::NodeId) -> usize {
        self.node(from).edges.borrow().len()
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

impl<N, E, D, M> GraphMut for LinkedGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn new(directedness: D, edge_multiplicity: M) -> Self {
        Self {
            nodes: Vec::new(),
            id: GraphId::default(),
            directedness,
            edge_multiplicity,
        }
    }

    fn node_data_mut(&mut self, id: &Self::NodeId) -> &mut Self::NodeData {
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &mut *self.node(id).data.get() }
    }

    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData {
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &mut *self.edge(id).data.get() }
    }

    fn clear(&mut self) {
        self.nodes.clear();
    }

    fn add_node(&mut self, data: Self::NodeData) -> Self::NodeId {
        let node = Rc::new(Node {
            data: UnsafeCell::new(data),
            edges: RefCell::new(Vec::new()),
            back_edges: RefCell::new(Vec::new()),
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
        let ends = self
            .directedness
            .coordinate_pair((from.clone(), into.clone()));

        if !self.allows_parallel_edges() {
            debug_assert!(self.num_edges_from_into(from, into) <= 1);
            if let Some(edge) = self
                .node(from)
                .edges
                .borrow_mut()
                .iter_mut()
                .find(|edge| edge.ends == ends)
            {
                let mut old_data = data;
                // SAFETY: See note on `LinkedGraph` type.
                std::mem::swap(unsafe { &mut *edge.data.get() }, &mut old_data);
                return AddEdgeResult::Updated(self.edge_id(edge), old_data);
            }
            debug_assert_eq!(self.num_edges_from_into(from, into), 0);
        }

        let (from, into) = ends.values();

        let edge = Rc::new(Edge::new(
            data,
            from.clone(),
            into.clone(),
            self.directedness(),
        ));

        let eid = self.edge_id(&edge);

        if self.is_directed() {
            // For directed graphs, add to the "into" node's `back_edges`.  We don't
            // maintain `back_edges` for undirected graphs since it's redundant with
            // `edges`.
            self.node(into).back_edges.borrow_mut().push(eid.clone());
        } else if from != into {
            // For undirected graphs (non-self-loop), add to the other node's edges_out
            self.node(into).edges.borrow_mut().push(Rc::clone(&edge));
        }

        self.node(from).edges.borrow_mut().push(edge);

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
        for edge in node.edges.borrow().iter() {
            // For undirected graphs, the "other" node could be either edge.from or edge.into
            match edge.ends.other_value(nid) {
                OtherValue::First(other_nid) | OtherValue::Second(other_nid) => {
                    let other_node = self.node(other_nid);
                    if self.is_directed() {
                        // For directed graphs, remove from edges_in
                        other_node
                            .back_edges
                            .borrow_mut()
                            .retain(|eid| *eid != self.edge_id(edge));
                    } else {
                        // For undirected graphs, remove from `edges`
                        other_node
                            .edges
                            .borrow_mut()
                            .retain(|e| self.edge_id(e) != self.edge_id(edge));
                    }
                }
                OtherValue::Both(_) => {}
            };
        }

        if self.is_directed() {
            // For directed graphs, also remove incoming edges from source nodes' edges_out
            for eid in node.back_edges.borrow().iter() {
                let edge = self.edge(eid);
                let from_nid = edge.ends.first();
                if from_nid != nid {
                    let from_node = self.node(&from_nid.clone());
                    from_node
                        .edges
                        .borrow_mut()
                        .retain(|edge| self.edge_id(edge) != *eid);
                }
            }
        }

        Rc::into_inner(node).unwrap().data.into_inner()
    }

    fn remove_edge(&mut self, eid: &Self::EdgeId) -> Self::EdgeData {
        self.assert_valid_edge_id(eid);
        let edge = eid.ptr.upgrade().expect("EdgeId is dangling");
        let (from_nid, into_nid) = edge.ends.values();

        // Remove from source node's edges_out
        let from_node = self.node(from_nid);
        from_node
            .edges
            .borrow_mut()
            .retain(|edge| *eid != self.edge_id(edge));

        if self.is_directed() {
            // For directed graphs, remove from target node's `back_edges`
            let to_node = self.node(into_nid);
            to_node.back_edges.borrow_mut().retain(|eid2| eid != eid2);
        } else if from_nid != into_nid {
            // For undirected graphs (non-self-loop), remove from target node's `edges`
            let to_node = self.node(into_nid);
            to_node
                .edges
                .borrow_mut()
                .retain(|edge| *eid != self.edge_id(edge));
        }

        Rc::into_inner(edge).unwrap().data.into_inner()
    }
}

impl<N, E, D, M> Clone for LinkedGraph<N, E, D, M>
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
