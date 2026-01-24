use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use crate::Graph;

struct VertexDebug<'a, G: Graph> {
    verticies: Vec<G::VertexId>,
    graph: &'a G,
    vertex_tags: &'a HashMap<G::VertexId, usize>,
}

impl<'a, G> Debug for VertexDebug<'a, G>
where
    G: Graph,
    G::VertexData: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.verticies
                    .iter()
                    .map(|vid| (self.vertex_tags[vid], self.graph.vertex_data(vid))),
            )
            .finish()
    }
}

struct EdgeTag(usize, usize);

impl Debug for EdgeTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {}", self.0, self.1)
    }
}

struct EdgeDebug<'a, G: Graph> {
    graph: &'a G,
    vertex_tags: &'a HashMap<G::VertexId, usize>,
}

impl<'a, G> Debug for EdgeDebug<'a, G>
where
    G: Graph,
    G::EdgeData: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.graph.edge_ids().map(|eid| {
                let (from, to) = self.graph.edge_ends(eid.clone());
                (
                    EdgeTag(self.vertex_tags[&from], self.vertex_tags[&to]),
                    self.graph.edge_data(&eid),
                )
            }))
            .finish()
    }
}

pub fn format_debug<'g, G>(graph: &'g G, fmt: &mut Formatter<'_>, name: &str) -> std::fmt::Result
where
    G: Graph,
    G::VertexData: Debug,
    G::EdgeData: Debug,
{
    let vertex_tags: HashMap<_, _> = graph
        .vertex_ids()
        .enumerate()
        .map(|(i, vid)| (vid, i))
        .collect();

    fmt.debug_struct(name)
        .field(
            "vertices",
            &VertexDebug {
                verticies: graph.vertex_ids().collect(),
                graph,
                vertex_tags: &vertex_tags,
            },
        )
        .field(
            "edges",
            &EdgeDebug {
                graph,
                vertex_tags: &vertex_tags,
            },
        )
        .finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{linked_graph::LinkedGraph, *};

    #[test]
    fn test_vertex_debug_empty_graph() {
        let graph = LinkedGraph::<(), ()>::new();
        let vertex_tags = HashMap::new();
        let debug = VertexDebug {
            verticies: vec![],
            graph: &graph,
            vertex_tags: &vertex_tags,
        };
        let output = format!("{:?}", debug);
        assert_eq!(output, "{}");
    }

    #[test]
    fn test_vertex_debug_single_vertex() {
        let mut graph = LinkedGraph::<i32, ()>::new();
        let v1 = graph.add_vertex(42);
        let mut vertex_tags = HashMap::new();
        vertex_tags.insert(v1, 0);
        let debug = VertexDebug {
            verticies: vec![v1],
            graph: &graph,
            vertex_tags: &vertex_tags,
        };
        let output = format!("{:?}", debug);
        assert_eq!(output, "{0: 42}");
    }

    #[test]
    fn test_format_debug_basic() {
        let mut graph = LinkedGraph::<&str, i32>::new();
        let v1 = graph.add_vertex("A");
        let v2 = graph.add_vertex("B");
        graph.add_edge(&v1, &v2, 10);

        // Single-line output.
        let output = format!("{:?}", &graph);
        let expected = r#"LinkedGraph { vertices: {0: "A", 1: "B"}, edges: {0 -> 1: 10} }"#;
        assert_eq!(output, expected);

        // Multi-line output.
        let output = format!("{:#?}", &graph);
        let expected = r#"LinkedGraph {
    vertices: {
        0: "A",
        1: "B",
    },
    edges: {
        0 -> 1: 10,
    },
}"#;
        assert_eq!(output, expected);
    }
}
