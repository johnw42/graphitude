use std::collections::HashSet;

use jrw_graph::{
    Graph, GraphMut,
    adjacency_matrix::{AdjacencyMatrix, SymmetricAdjacencyMatrix},
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
        StringGraph {
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

    fn is_directed(&self) -> bool {
        false
    }

    fn vertex_data(&self, id: &Self::VertexId) -> &Self::VertexData {
        self.vertices.get(id).expect("Vertex does not exist")
    }

    fn num_edges_between(&self, from: Self::VertexId, into: Self::VertexId) -> usize {
        self.edges.get(&from, &into).into_iter().count()
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        self.edges.get(&id.0, &id.1).expect("Edge does not exist")
    }

    fn edge_source_and_target(&self, eid: Self::EdgeId) -> (Self::VertexId, Self::VertexId) {
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

    fn add_edge(
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

#[test]
fn test_string_graph() {
    let mut graph = StringGraph::new();
    let a = graph.add_vertex("A".to_string());
    let b = graph.add_vertex("B".to_string());
    let c = graph.add_vertex("C".to_string());
    let ab = graph.add_edge(&a, &b, ()).0;
    let bc = graph.add_edge(&b, &c, ()).0;
    assert_eq!(
        graph
            .edges_out(a.clone())
            .into_iter()
            .map(|edge_id| graph.edge_target(edge_id))
            .collect::<Vec<_>>(),
        vec![b.clone()]
    );
    assert_eq!(graph.vertex_data(&a), &"A".to_string());
    assert_eq!(graph.edge_data(&ab), (&()));
    assert_eq!(graph.edge_data(&bc), (&()));
}
#[test]
fn test_add_multiple_vertices() {
    let mut graph = StringGraph::new();
    let vertices: Vec<_> = vec!["A", "B", "C", "D"]
        .into_iter()
        .map(|s| graph.add_vertex(s.to_string()))
        .collect();
    assert_eq!(graph.vertex_ids().count(), 4);
    for v in vertices {
        assert_eq!(graph.vertex_data(&v), &v);
    }
}

#[test]
fn test_edge_id_ordering() {
    let edge1 = EdgeId::new("Z".to_string(), "A".to_string());
    let edge2 = EdgeId::new("A".to_string(), "Z".to_string());
    assert_eq!(edge1, edge2);
    assert_eq!(edge1.0, "A".to_string());
}

#[test]
fn test_symmetric_edges() {
    let mut graph = StringGraph::new();
    let a = graph.add_vertex("A".to_string());
    let b = graph.add_vertex("B".to_string());
    graph.add_edge(&a, &b, ());
    assert_eq!(graph.num_edges_between(a.clone(), b.clone()), 1);
    assert_eq!(graph.num_edges_between(b, a), 1);
}

#[test]
fn test_remove_vertex_cleans_edges() {
    let mut graph = StringGraph::new();
    let a = graph.add_vertex("A".to_string());
    let b = graph.add_vertex("B".to_string());
    graph.add_edge(&a, &b, ());
    graph.remove_vertex(&a);
    assert_eq!(graph.num_vertices(), 1);
    assert_eq!(graph.num_edges(), 0);
}

#[test]
fn test_remove_edge() {
    let mut graph = StringGraph::new();
    let a = graph.add_vertex("A".to_string());
    let b = graph.add_vertex("B".to_string());
    let edge = graph.add_edge(&a, &b, ()).0;
    assert_eq!(graph.num_edges(), 1);
    graph.remove_edge(&edge);
    assert_eq!(graph.num_edges(), 0);
}

#[test]
fn test_is_undirected() {
    let graph = StringGraph::new();
    assert!(!graph.is_directed());
}

#[test]
fn test_edges_out_from_vertex() {
    let mut graph = StringGraph::new();
    let a = graph.add_vertex("A".to_string());
    let b = graph.add_vertex("B".to_string());
    let c = graph.add_vertex("C".to_string());
    graph.add_edge(&a, &b, ());
    graph.add_edge(&a, &c, ());
    let edges_out = graph.edges_out(a);
    assert_eq!(edges_out.into_iter().count(), 2);
}
