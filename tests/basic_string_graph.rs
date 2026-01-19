use std::collections::HashSet;

use jrw_graph::{
    Graph,
    adjacency_matrix::{AdjacencyMatrix, SymmetricAdjacencyMatrix},
    graph::GraphMutStructure,
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

    fn num_edges_between(&self, from: &Self::VertexId, into: &Self::VertexId) -> usize {
        self.edges.get(from, into).into_iter().count()
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        self.edges.get(&id.0, &id.1).expect("Edge does not exist")
    }

    fn edge_source(&self, id: &Self::EdgeId) -> Self::VertexId {
        id.0.clone()
    }

    fn edge_target(&self, id: &Self::EdgeId) -> Self::VertexId {
        id.1.clone()
    }

    fn vertex_ids(&self) -> Vec<String> {
        self.vertices.iter().cloned().collect()
    }

    fn edge_ids(&self) -> Vec<Self::EdgeId> {
        self.edges
            .edges()
            .map(|(from, into, _)| EdgeId::new(from, into))
            .collect()
    }
}

impl GraphMutStructure for StringGraph {
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
            .edges_out(&a)
            .into_iter()
            .map(|edge_id| graph.edge_target(&edge_id))
            .collect::<Vec<_>>(),
        vec![b.clone()]
    );
    assert_eq!(graph.vertex_data(&a), &"A".to_string());
    assert_eq!(graph.edge_data(&ab), (&()));
    assert_eq!(graph.edge_data(&bc), (&()));
}
