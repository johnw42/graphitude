use std::collections::HashMap;

use jrw_graph::{Graph, GraphMut};

struct StringGraph {
    vertices: Vec<String>,
    edges: HashMap<String, Edge>,
}

struct Edge {
    to: String,
    label: String,
}

impl StringGraph {
    fn new() -> Self {
        StringGraph {
            vertices: Vec::new(),
            edges: HashMap::new(),
        }
    }
}

impl Graph for StringGraph {
    type VertexId = String;
    type VertexData = String;
    type EdgeData = String;

    fn neighbors(&self, from: &Self::VertexId) -> impl IntoIterator<Item = Self::VertexId> {
        self.edges.get(from).into_iter().map(|edge| edge.to.clone())
    }

    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData {
        self.vertices
            .iter()
            .find(|v| *v == id)
            .expect("Vertex not found")
    }

    fn edge_data(&self, from: &Self::VertexId, to: &Self::VertexId) -> Option<&Self::EdgeData> {
        self.edges.get(from).and_then(|edge| {
            if &edge.to == to {
                Some(&edge.label)
            } else {
                None
            }
        })
    }

    fn vertex_ids(&self) -> Vec<Self::VertexId> {
        self.vertices.clone()
    }
}

impl GraphMut for StringGraph {
    fn add_vertex(&mut self, data: Self::VertexData) -> Self::VertexId {
        self.vertices.push(data.clone());
        data
    }

    fn add_edge(&mut self, from: &Self::VertexId, to: &Self::VertexId, data: Self::EdgeData) {
        self.edges.insert(
            from.clone(),
            Edge {
                to: to.clone(),
                label: data.clone(),
            },
        );
    }
}

#[test]
fn test_string_graph() {
    let mut graph = StringGraph::new();
    let a = graph.add_vertex("A".to_string());
    let b = graph.add_vertex("B".to_string());
    let c = graph.add_vertex("C".to_string());
    graph.add_edge(&a, &b, "edge_AB".to_string());
    graph.add_edge(&b, &c, "edge_BC".to_string());
    assert_eq!(
        graph.neighbors(&a).into_iter().collect::<Vec<_>>(),
        vec![b.clone()]
    );
    assert_eq!(graph.vertex_data(&a), &"A".to_string());
    assert_eq!(graph.edge_data(&a, &b), Some(&"edge_AB".to_string()));
    assert_eq!(graph.edge_data(&a, &c), None);
}
