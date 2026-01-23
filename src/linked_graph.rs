use std::hash::Hash;

use crate::{Graph, GraphMut};

struct VertexNode<V, E> {
    data: V,
    edges_out: Vec<Box<EdgeNode<V, E>>>,
    edges_in: Vec<EdgeId<V, E>>,
}

#[derive(PartialOrd, Ord, Debug)]
pub struct VertexId<V, E>(*mut VertexNode<V, E>);

impl<V, E> Clone for VertexId<V, E> {
    fn clone(&self) -> Self {
        VertexId(self.0)
    }
}

impl<V, E> Copy for VertexId<V, E> {}

impl<V, E> PartialEq for VertexId<V, E> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V, E> Eq for VertexId<V, E> {}

impl<V, E> Hash for VertexId<V, E> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.0 as usize).hash(state);
    }
}

impl<V, E> From<&VertexNode<V, E>> for VertexId<V, E> {
    fn from(ptr: &VertexNode<V, E>) -> Self {
        VertexId(ptr as *const _ as *mut _)
    }
}

struct EdgeNode<V, E> {
    data: E,
    from: VertexId<V, E>,
    to: VertexId<V, E>,
}

#[derive(PartialOrd, Ord, Debug)]
pub struct EdgeId<V, E>(*mut EdgeNode<V, E>);

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

pub struct LinkedGraph<V, E> {
    vertices: Vec<Box<VertexNode<V, E>>>,
}

impl<V, E> LinkedGraph<V, E> {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
        }
    }
}

impl<V, E> Graph for LinkedGraph<V, E> {
    type VertexId = VertexId<V, E>;
    type VertexData = V;
    type EdgeId = EdgeId<V, E>;
    type EdgeData = E;

    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData {
        unsafe { &(*id.0).data }
    }

    fn vertex_ids(&self) -> impl Iterator<Item = Self::VertexId> {
        self.vertices.iter().map(|node| VertexId::from(&**node))
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        unsafe { &(*id.0).data }
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.vertices
            .iter()
            .flat_map(|vnode| vnode.edges_out.iter().map(|enode| EdgeId::from(&**enode)))
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::VertexId, Self::VertexId) {
        let edge_node = unsafe { &*eid.0 };
        (edge_node.from, edge_node.to)
    }

    fn edges_from(&self, from: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> {
        unsafe { &*from.0 }
            .edges_out
            .iter()
            .map(|enode| EdgeId::from(&**enode))
    }

    fn edges_into(&self, into: Self::VertexId) -> impl Iterator<Item = Self::EdgeId> {
        unsafe { &*into.0 }.edges_in.iter().cloned()
    }

    fn num_edges_into(&self, into: Self::VertexId) -> usize {
        unsafe { &*into.0 }.edges_in.len()
    }

    fn num_edges_from(&self, from: Self::VertexId) -> usize {
        unsafe { &*from.0 }.edges_out.len()
    }

    fn edges_between(
        &self,
        from: Self::VertexId,
        into: Self::VertexId,
    ) -> impl Iterator<Item = Self::EdgeId> {
        self.edges_from(from).filter(move |eid| {
            let (source, target) = self.edge_ends(*eid);
            source == from && target == into
        })
    }
}

impl<V, E> GraphMut for LinkedGraph<V, E> {
    fn clear(&mut self) {
        self.vertices.clear();
    }

    fn add_vertex(&mut self, data: Self::VertexData) -> Self::VertexId {
        let vnode = Box::new(VertexNode {
            data,
            edges_out: Vec::new(),
            edges_in: Vec::new(),
        });
        let vid = VertexId::from(&*vnode);
        self.vertices.push(vnode);
        vid
    }

    fn add_or_replace_edge(
        &mut self,
        from: &Self::VertexId,
        into: &Self::VertexId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let enode = Box::new(EdgeNode {
            data,
            from: *from,
            to: *into,
        });
        let eid = EdgeId::from(&*enode);

        unsafe {
            (&mut *from.0).edges_out.push(enode);
            (&mut *into.0).edges_in.push(eid);
        }

        (eid, None)
    }

    fn remove_vertex(&mut self, id: &Self::VertexId) -> V {
        let index = self
            .vertices
            .iter()
            .position(|vnode| VertexId::from(&**vnode) == *id)
            .expect("Vertex does not exist");
        let vnode = self.vertices.remove(index);
        for enode in &vnode.edges_out {
            let to_vid = enode.to;
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

    fn remove_edge(&mut self, eid: &Self::EdgeId) -> Option<Self::EdgeData> {
        let enode = unsafe { &*eid.0 };
        let from_vid = enode.from;
        let to_vid = enode.to;

        let from_vnode = unsafe { &mut *from_vid.0 };
        from_vnode
            .edges_out
            .retain(|enode| *eid != EdgeId::from(enode));

        let to_vnode = unsafe { &mut *to_vid.0 };
        to_vnode.edges_in.retain(|&eid2| *eid != eid2);

        Some(unsafe { Box::from_raw(eid.0).data })
    }
}


#[cfg(test)]
mod tests {
    use crate::{tests::TestDataBuilder, *};
    use super::*;

    impl TestDataBuilder for LinkedGraph<i32, String> {
        type Graph = Self;

        fn new_graph() -> Self::Graph {
            Self::new()
        }

        fn new_edge_data(i: usize) -> String {
            format!("e{}", i)
        }

        fn new_vertex_data(i: usize) -> i32 {
            i as i32
        }
    }

    graph_tests!(LinkedGraph<i32, String>);
    graph_test_copy_from_with!(
        LinkedGraph<i32, String>,
        |data| data * 2,
        |data: &String| format!("{}-copied", data));
}
