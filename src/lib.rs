// use std::{collections::HashMap, hash::Hash, marker::PhantomData};

// mod id;
// mod kind;

// use id::*;
// use kind::*;

// pub struct Graph<K: Directedness, S: EdgeStorage> {
//     vertices: HashMap<VertexId<K>, VertexImpl<K::VertexData, K::EdgeData>>,
//     next_vertex_id: usize,
//     p: PhantomData<E>,
// }

// struct VertexImpl<V, E> {
//     data: V,
//     edges: HashMap<VertexId, EdgeImpl<V, E>>,
// }

// struct EdgeImpl<V, E> {
//     data: E,
//     vertex_to: VertexId,
//     p: PhantomData<V>,
// }

// impl<V, E> Graph<V, E> {
//     pub fn new() -> Self {
//         Graph {
//             vertices: HashMap::new(),
//             next_vertex_id: 0,
//             p: PhantomData,
//         }
//     }

//     pub fn add_vertex(&mut self, label: V) -> modname::VertexId {
//         let vertex_id = modname::VertexId(self.next_vertex_id);
//         self.next_vertex_id += 1;
//         self.vertices.insert(
//             vertex_id,
//             VertexImpl {
//                 label,
//                 edges: HashMap::new(),
//             },
//         );
//         vertex_id
//     }

//     pub fn add_edge(&mut self, from: modname::VertexId, to: modname::VertexId, label: E) {
//         assert!(self.vertices.contains_key(&to), "To vertex does not exist");
//         if let Some(vertex_data) = self.vertices.get_mut(&from) {
//             vertex_data.edges.insert(
//                 to,
//                 EdgeImpl {
//                     label,
//                     vertex_to: to,
//                     p: PhantomData,
//                 },
//             );
//         } else {
//             panic!("From vertex does not exist");
//         }
//     }
// }

pub mod graph;
pub mod object_graph;

pub use graph::{Graph, GraphMut};
