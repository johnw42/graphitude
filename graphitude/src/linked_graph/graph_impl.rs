use std::{
    cell::{Ref, RefCell, UnsafeCell},
    collections::HashSet,
    marker::PhantomData,
    rc::Rc,
};

use crate::{
    directedness::Directedness,
    edge_multiplicity::EdgeMultiplicity,
    end_pair::EndPair,
    graph_traits::{AddEdgeResult, EdgeIdTrait},
    linked_graph::{EdgeId, NodeId},
    prelude::*,
    util::OtherValue,
};

use derivative::Derivative;

pub(super) struct Node<N, E, D: DirectednessTrait> {
    data: UnsafeCell<N>,
    edges: RefCell<Vec<Rc<Edge<N, E, D>>>>,
    // Only maintained for directed graphs, since for undirected graphs
    // edges_out is sufficient to find all edges.
    back_edges: RefCell<Vec<EdgeId<N, E, D>>>,
    directedness: PhantomData<D>,
}

pub(super) struct Edge<N, E, D: DirectednessTrait> {
    pub(super) data: UnsafeCell<E>,
    pub(super) ends: EndPair<NodeId<N, E, D>, D>,
    pub(super) directedness: PhantomData<D>,
}

impl<N, E, D: DirectednessTrait> Edge<N, E, D> {
    fn new(data: E, from: NodeId<N, E, D>, into: NodeId<N, E, D>, directedness: D) -> Self {
        Self {
            data: UnsafeCell::new(data),
            ends: EndPair::new(from, into, directedness),
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
            directedness: PhantomData,
        }
    }

    fn edge_id(&self, ptr: &Rc<Edge<N, E, D>>) -> EdgeId<N, E, D> {
        EdgeId {
            ptr: Rc::downgrade(ptr),
            directedness: self.directedness,
        }
    }

    fn node(&self, id: &NodeId<N, E, D>) -> &Node<N, E, D> {
        let ptr = Rc::as_ptr(&id.ptr.upgrade().expect("NodeId is dangling"));
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &*ptr }
    }

    fn edge(&self, id: &EdgeId<N, E, D>) -> &Edge<N, E, D> {
        let ptr: *const Edge<N, E, D> = Rc::as_ptr(&id.ptr.upgrade().expect("EdgeId is dangling"));
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &*ptr }
    }
}

impl<N, E, D, M> GraphImpl for LinkedGraph<N, E, D, M>
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

    fn node_data<'a>(&'a self, id: &Self::NodeId) -> &'a Self::NodeData {
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &*self.node(id).data.get() }
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.nodes.iter().map(|node| self.node_id(node))
    }

    fn edge_data<'a>(&'a self, id: &Self::EdgeId) -> &'a Self::EdgeData {
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &*self.edge(id).data.get() }
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        // For undirected graphs, deduplicate by Rc pointer address
        let mut seen: HashSet<*const Edge<N, E, D>> = HashSet::new();

        self.nodes.iter().flat_map(move |node| {
            node.edges
                .borrow()
                .iter()
                .filter(|edge| self.directedness().is_directed() || seen.insert(Rc::as_ptr(edge)))
                .map(|edge| self.edge_id(edge))
                .collect::<Vec<_>>()
        })
    }

    fn edges_from<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        EdgesOutIter {
            borrow: self.node(from).edges.borrow(),
            graph: self,
            index: 0,
        }
    }

    fn edges_into<'a, 'b: 'a>(
        &'a self,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        let node = self.node(into);
        debug_assert!(self.directedness().is_directed() || node.back_edges.borrow().is_empty());
        std::iter::chain(
            EdgesInIter {
                borrow: node.back_edges.borrow(),
                index: 0,
            }
            .take_while(|_| self.directedness().is_directed()),
            EdgesOutIter {
                borrow: node.edges.borrow(),
                graph: self,
                index: 0,
            }
            .take_while(|_| !self.directedness().is_directed()),
        )
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        into: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.edges_from(from)
            .filter(move |edge| edge.other_end(&from) == *into)
    }

    fn has_edge_from_into(&self, from: &Self::NodeId, into: &Self::NodeId) -> bool {
        self.edges_from_into(from, into).next().is_some()
    }

    fn num_edges_into(&self, into: &Self::NodeId) -> usize {
        if self.directedness().is_directed() {
            self.node(into).back_edges.borrow().len()
        } else {
            self.node(into).edges.borrow().len()
        }
    }

    fn num_edges_from(&self, from: &Self::NodeId) -> usize {
        self.node(from).edges.borrow().len()
    }
}

impl<N, E, D, M> GraphImplMut for LinkedGraph<N, E, D, M>
where
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn new(directedness: D, edge_multiplicity: M) -> Self {
        Self {
            nodes: Vec::new(),
            directedness,
            edge_multiplicity,
        }
    }

    fn node_data_mut<'a>(&'a mut self, id: &'a Self::NodeId) -> &'a mut Self::NodeData {
        // SAFETY: See note on `LinkedGraph` type.
        unsafe { &mut *self.node(id).data.get() }
    }

    fn edge_data_mut<'a>(&'a mut self, id: &'a Self::EdgeId) -> &'a mut Self::EdgeData {
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

        if !self.edge_multiplicity().allows_parallel_edges() {
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

        if self.directedness().is_directed() {
            // For directed graphs, add to the "into" node's edges_in.  We don't
            // maintain edges_in for undirected graphs since it's redundant with
            // edges_out.
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
                OtherValue::Both(_) => {}
                OtherValue::First(other_nid) | OtherValue::Second(other_nid) => {
                    let other_node = self.node(other_nid);
                    if self.directedness().is_directed() {
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
            };
        }

        if self.directedness().is_directed() {
            // For directed graphs, also remove incoming edges from source nodes' edges_out
            for eid in node.back_edges.borrow().iter() {
                let edge = self.edge(eid);
                let from_nid = edge.ends.left();
                if from_nid != nid {
                    let from_node = self.node(from_nid);
                    from_node
                        .edges
                        .borrow_mut()
                        .retain(|edge| self.edge_id(edge) != *eid);
                }
            }
        } else {
            debug_assert!(node.back_edges.borrow().is_empty());
        }

        Rc::into_inner(node)
            // This step will always succeed because there are no other strong
            // reference to the node.
            .unwrap()
            .data
            .into_inner()
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

        if self.directedness().is_directed() {
            // For directed graphs, remove from target node's edges_in
            let to_node = self.node(into_nid);
            to_node.back_edges.borrow_mut().retain(|eid2| eid != eid2);
        } else if from_nid != into_nid {
            // For undirected graphs (non-self-loop), remove from target node's edges_out
            let to_node = self.node(into_nid);
            to_node
                .edges
                .borrow_mut()
                .retain(|edge| *eid != self.edge_id(edge));
        }

        Rc::into_inner(edge).unwrap().data.into_inner()
    }
}

struct EdgesOutIter<'a, N, E, D: DirectednessTrait, M: EdgeMultiplicityTrait> {
    borrow: Ref<'a, Vec<Rc<Edge<N, E, D>>>>,
    graph: &'a LinkedGraph<N, E, D, M>,
    index: usize,
}

impl<'a, N, E, D: DirectednessTrait, M: EdgeMultiplicityTrait> Iterator
    for EdgesOutIter<'a, N, E, D, M>
{
    type Item = EdgeId<N, E, D>;
    fn next(&mut self) -> Option<Self::Item> {
        let edge = self.borrow.get(self.index)?;
        self.index += 1;
        Some(self.graph.edge_id(edge))
    }
}

struct EdgesInIter<'a, N, E, D: DirectednessTrait> {
    borrow: Ref<'a, Vec<EdgeId<N, E, D>>>,
    index: usize,
}

impl<'a, N, E, D: DirectednessTrait> Iterator for EdgesInIter<'a, N, E, D> {
    type Item = EdgeId<N, E, D>;
    fn next(&mut self) -> Option<Self::Item> {
        let edge_id = self.borrow.get(self.index)?;
        self.index += 1;
        Some(edge_id.clone())
    }
}
