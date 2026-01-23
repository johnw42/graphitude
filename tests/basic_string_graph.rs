use std::collections::HashSet;

use jrw_graph::{
    Graph, GraphMut, adjacency_matrix::{AdjacencyMatrix, SymmetricAdjacencyMatrix}, graph::Undirected, graph_tests, tests::TestDataBuilder
};

/// An undirected graph where vertices are identified by strings.  A vertex's ID
/// is the same as its data.  Edges have no data and are identified by the pair
/// of vertex IDs they connect.
struct StringGraph {
    vertices: HashSet<VertexId>,
    edges: SymmetricAdjacencyMatrix<VertexId, ()>,
}

type VertexId = String;

// Invariant: `EdgeId` always has the smaller `VertexId` first.
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
struct EdgeId(VertexId, VertexId);

impl EdgeId {
    fn new(from: VertexId, into: VertexId) -> Self {
        if from <= into {
            EdgeId(from, into)
        } else {
            EdgeId(into, from)
        }
    }
}

impl StringGraph {
    fn new() -> Self {
        Self {
            vertices: HashSet::new(),
            edges: SymmetricAdjacencyMatrix::new(),
        }
    }

    fn edge_id(
        &self,
        from: <StringGraph as Graph>::VertexId,
        into: <StringGraph as Graph>::VertexId,
    ) -> <StringGraph as Graph>::EdgeId {
        assert!(self.vertices.contains(&from));
        debug_assert!(self.vertices.contains(&into));
        EdgeId::new(from, into)
    }
}

impl Graph for StringGraph {
    type VertexData = String;
    type VertexId = VertexId;
    type EdgeData = ();
    type EdgeId = EdgeId;
    type Directedness = Undirected;

    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData {
        self.vertices.get(id).expect("Vertex does not exist")
    }

    fn num_edges_between(&self, from: Self::VertexId, into: Self::VertexId) -> usize {
        self.edges.get(&from, &into).into_iter().count()
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        self.edges.get(&id.0, &id.1).expect("Edge does not exist")
    }

    fn edge_ends(&self, eid: Self::EdgeId) -> (Self::VertexId, Self::VertexId) {
        (eid.0.clone(), eid.1.clone())
    }

    fn vertex_ids(&self) -> impl Iterator<Item = Self::VertexId> {
        self.vertices.iter().cloned()
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> {
        self.edges
            .edges()
            .map(|(from, into, _)| EdgeId::new(from, into))
    }
}

impl GraphMut for StringGraph {
    fn add_vertex(&mut self, data: Self::VertexData) -> Self::VertexId {
        self.vertices.insert(data.clone());
        data
    }

    fn add_or_replace_edge(
        &mut self,
        from: &Self::VertexId,
        into: &Self::VertexId,
        data: Self::EdgeData,
    ) -> (Self::EdgeId, Option<Self::EdgeData>) {
        let old_data = self.edges.insert(from.clone(), into.clone(), data);
        (self.edge_id(from.clone(), into.clone()), old_data)
    }

    fn remove_vertex(&mut self, id: &Self::VertexId) -> String {
        let edges_from = self
            .edges
            .edges_from(id)
            .map(|(into, _)| into)
            .collect::<Vec<_>>();
        for into in edges_from {
            self.edges.remove(id, &into);
        }
        self.vertices.remove(id);
        id.clone()
    }

    fn remove_edge(&mut self, id: &Self::EdgeId) -> Option<Self::EdgeData> {
        self.edges.remove(&id.0, &id.1).map(|_| ())
    }
}

impl TestDataBuilder for StringGraph {
    type Graph = Self;

    fn new_graph() -> Self::Graph {
        Self::new()
    }

    fn new_edge_data(_i: usize) -> () {
        ()
    }

    fn new_vertex_data(i: usize) -> String {
        format!("v{}", i)
    }
}

graph_tests!(StringGraph);

#[test]
fn test_edge_id_ordering() {
    let edge1 = EdgeId::new("Z".to_string(), "A".to_string());
    let edge2 = EdgeId::new("A".to_string(), "Z".to_string());
    assert_eq!(edge1, edge2);
    assert_eq!(edge1.0, "A".to_string());
}
