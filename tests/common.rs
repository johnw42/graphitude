#![cfg(feature = "nope")]
use jrw_graph::{Graph, graph::GraphMutStructure};

pub fn test_graph<E, G: Graph + GraphMutStructure<VertexData = String, EdgeData = E>>(g: G) {
    assert_eq!(g.num_vertices(), 0);
    let v1 = g.add_vertex("A".to_string());
    let v2 = g.add_vertex("B".to_string());
    assert_eq!(g.num_vertices(), 2);

    let (e1, old_data) = g.add_edge(&v1, &v2, "edge_AB".to_string());
    assert!(old_data.is_none());

    assert_eq!(g.edge_data(&e1), Some(&"edge_AB"));
}
