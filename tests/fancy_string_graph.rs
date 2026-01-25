use std::collections::HashMap;

use jrw_graph::{
    Directed, Graph, GraphMut, graph_test_copy_from_with, graph_tests, tests::TestDataBuilder,
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

    fn vertex(&self, id: &VertexId) -> &Vertex {
        self.vertices.get(id).expect("Invalid vertex ID")
    }

    fn vertex_mut(&mut self, id: &VertexId) -> &mut Vertex {
        self.vertices.get_mut(id).expect("Invalid vertex ID")
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
    type Directedness = Directed;

    fn vertex_data(&self, id: Self::VertexId) -> &Self::VertexData {
        &self.vertex(&id).data
    }

    fn edge_data(&self, id: Self::EdgeId) -> &Self::EdgeData {
        &self.edge(&id).data
    }

    fn edge_source(&self, id: Self::EdgeId) -> Self::VertexId {
        id.0
    }

    fn edge_target(&self, id: Self::EdgeId) -> Self::VertexId {
        id.1
    }

    fn vertex_ids(&self) -> impl Iterator<Item = Self::VertexId> {
        self.vertices.keys().cloned()
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.vertices.iter().flat_map(|(from_id, vertex)| {
            vertex
                .edges_out
                .iter()
                .map(move |edge| (from_id.clone(), edge.target.clone()))
        })
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::VertexId, Self::VertexId) {
        eid
    }
}

impl GraphMut for StringGraph {
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

    fn add_or_replace_edge(
        &mut self,
        from: Self::VertexId,
        to: Self::VertexId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        assert!(self.vertices.contains_key(&to), "Invalid 'to' vertex ID");
        self.vertices
            .get_mut(&from)
            .expect("Invalid 'from' vertex ID")
            .edges_out
            .push(Edge {
                target: to.clone(),
                data: data,
            });
        ((from, to), None)
    }

    fn remove_vertex(&mut self, id: Self::VertexId) -> Self::VertexData {
        self.vertices
            .remove(&id)
            .map(|v| v.data)
            .expect("Invalid vertex ID")
    }

    fn remove_edge(&mut self, (from, to): Self::EdgeId) -> Option<Self::EdgeData> {
        let vertex = self.vertices.get_mut(&from)?;
        if let Some(pos) = vertex.edges_out.iter().position(|e| e.target == to) {
            let edge = vertex.edges_out.remove(pos);
            Some(edge.data)
        } else {
            None
        }
    }
}

impl TestDataBuilder for StringGraph {
    type Graph = Self;

    fn new_graph() -> Self::Graph {
        Self::new()
    }

    fn new_edge_data(i: usize) -> String {
        format!("e{}", i)
    }

    fn new_vertex_data(i: usize) -> String {
        format!("v{}", i)
    }
}

graph_tests!(StringGraph);
graph_test_copy_from_with!(
    StringGraph,
    |data| format!("{}-copied", data),
    |data| format!("{}-copied", data)
);
