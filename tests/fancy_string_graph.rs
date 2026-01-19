#![cfg(feature = "nope")]
use std::collections::HashMap;

use jrw_graph::{
    Graph,
    graph::{GraphMutData, GraphMutStructure},
};

struct StringGraph {
    vertices: HashMap<VertexId, Vertex>,
    next_vertex_id: usize,
}

type VertexId = usize;
type EdgeId = (VertexId, VertexId);

struct Vertex {
    data: String,
    edges_out: Vec<Edge>,
}

struct Edge {
    target: VertexId,
    data: String,
}

impl StringGraph {
    fn new() -> Self {
        StringGraph {
            vertices: HashMap::new(),
            next_vertex_id: 0,
        }
    }

    fn edge_id(&self, from: VertexId, to: VertexId) -> EdgeId {
        (from, to)
    }

    fn vertex(&self, id: VertexId) -> &Vertex {
        self.vertices.get(&id).expect("Invalid vertex ID")
    }

    fn vertex_mut(&mut self, id: VertexId) -> &mut Vertex {
        self.vertices.get_mut(&id).expect("Invalid vertex ID")
    }

    fn edge(&self, id: &EdgeId) -> &Edge {
        self.vertices
            .get(&id.0)
            .expect("Invalid edge ID")
            .edges_out
            .iter()
            .find(|e| e.target == id.1)
            .expect("Invalid edge ID")
    }

    fn edge_mut(&mut self, id: &EdgeId) -> &mut Edge {
        self.vertices
            .get_mut(&id.0)
            .expect("Invalid edge ID")
            .edges_out
            .iter_mut()
            .find(|e| e.target == id.1)
            .expect("Invalid edge ID")
    }
}

impl Graph for StringGraph {
    type VertexData = String;
    type VertexId = VertexId;
    type EdgeData = String;
    type EdgeId = EdgeId;

    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData {
        &self.vertex(*id).data
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        &self.edge(id).data
    }

    fn edge_source(&self, id: &Self::EdgeId) -> Self::VertexId {
        id.0
    }

    fn edge_target(&self, id: &Self::EdgeId) -> Self::VertexId {
        id.1
    }

    fn vertex_ids(&self) -> Vec<Self::VertexId> {
        self.vertices.keys().cloned().collect()
    }
}

impl GraphMutStructure for StringGraph {
    fn add_vertex(&mut self, data: Self::VertexData) -> Self::VertexId {
        let id = self.next_vertex_id;
        self.next_vertex_id += 1;
        self.vertices.insert(
            id,
            Vertex {
                data,
                edges_out: Vec::new(),
            },
        );
        id
    }

    fn add_edge(
        &mut self,
        from: &Self::VertexId,
        to: &Self::VertexId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        assert!(self.vertices.contains_key(to), "Invalid 'to' vertex ID");
        self.vertex_mut(*from).edges_out.push(Edge {
            target: *to,
            data: data,
        });
        ((*from, *to), None)
    }

    fn remove_vertex(&mut self, id: &Self::VertexId) -> Self::VertexData {
        self.vertices
            .remove(id)
            .map(|v| v.data)
            .expect("Invalid vertex ID")
    }

    fn remove_edge(&mut self, (from, to): &Self::EdgeId) -> Option<Self::EdgeData> {
        let vertex = self.vertices.get_mut(from)?;
        if let Some(pos) = vertex.edges_out.iter().position(|e| e.target == *to) {
            let edge = vertex.edges_out.remove(pos);
            Some(edge.data)
        } else {
            None
        }
    }
}

impl GraphMutData for StringGraph {
    fn edge_data_mut(&mut self, id: &Self::EdgeId) -> &mut Self::EdgeData {
        &mut self.edge_mut(id).data
    }

    fn vertex_data_mut(&mut self, id: &Self::VertexId) -> &mut Self::VertexData {
        &mut self.vertex_mut(*id).data
    }
}

#[test]
fn test_string_graph() {
    let mut graph = StringGraph::new();
    let a = graph.add_vertex("A".to_string());
    let b = graph.add_vertex("B".to_string());
    let c = graph.add_vertex("C".to_string());
    let ab = graph.add_edge(&a, &b, "edge_AB".to_string()).unwrap();
    let bc = graph.add_edge(&b, &c, "edge_BC".to_string()).unwrap();
    assert_eq!(
        graph.neighbors(&a).into_iter().collect::<Vec<_>>(),
        vec![b.clone()]
    );
    assert_eq!(graph.vertex_data(&a), &"A".to_string());
    assert_eq!(graph.edge_data(&ab), Some(&"edge_AB".to_string()));
    assert_eq!(graph.edge_data(&bc), Some(&"edge_BC".to_string()));
}
