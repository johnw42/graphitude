use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use crate::{Graph, util::sort_pair};

struct VertexTag<'a>(&'a str);

impl<'a> Debug for VertexTag<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct VertexDebug<'a, G: Graph> {
    graph: &'a G,
    vertex_order: &'a [G::VertexId],
    vertex_tags: &'a HashMap<G::VertexId, String>,
    show_data: bool,
}

impl<'a, G> Debug for VertexDebug<'a, G>
where
    G: Graph,
    G::VertexData: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.show_data {
            f.debug_map()
                .entries(self.vertex_order.iter().map(|vid| {
                    (
                        VertexTag(&self.vertex_tags[vid]),
                        self.graph.vertex_data(vid),
                    )
                }))
                .finish()
        } else {
            f.debug_list()
                .entries(
                    self.vertex_order
                        .iter()
                        .map(|vid| self.graph.vertex_data(vid)),
                )
                .finish()
        }
    }
}

struct EdgeTag<'a>(&'a str, &'a str, bool);

impl<'a> Debug for EdgeTag<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.2 {
            write!(f, "{} -> {}", &self.0, &self.1)
        } else {
            let (t1, t2) = sort_pair(&self.0, &self.1);
            write!(f, "{} -- {}", t1, t2)
        }
    }
}

struct EdgeDebug<'a, G: Graph> {
    graph: &'a G,
    edge_order: &'a [G::EdgeId],
    vertex_tags: &'a HashMap<G::VertexId, String>,
    show_data: bool,
}

impl<'a, G> Debug for EdgeDebug<'a, G>
where
    G: Graph,
    G::EdgeData: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let make_edge_tag = |eid: &G::EdgeId| {
            let (from, to) = self.graph.edge_ends(eid.clone());
            EdgeTag(
                &self.vertex_tags[&from],
                &self.vertex_tags[&to],
                self.graph.is_directed(),
            )
        };

        if self.show_data {
            f.debug_map()
                .entries(
                    self.edge_order
                        .iter()
                        .map(|eid| (make_edge_tag(eid), self.graph.edge_data(eid))),
                )
                .finish()
        } else {
            f.debug_list()
                .entries(self.edge_order.iter().map(make_edge_tag))
                .finish()
        }
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
        .map(|(i, vid)| (vid.clone(), i.to_string()))
        .collect();
    format_debug_with(
        graph,
        fmt,
        name,
        &mut |vid| vertex_tags[vid].clone(),
        true,
        true,
    )
}

pub fn format_debug_with<'g, G>(
    graph: &'g G,
    fmt: &mut Formatter<'_>,
    name: &str,
    vertex_tag: &mut impl FnMut(&G::VertexId) -> String,
    show_edge_data: bool,
    show_vertex_data: bool,
) -> std::fmt::Result
where
    G: Graph,
    G::VertexData: Debug,
    G::EdgeData: Debug,
{
    let vertex_tags: HashMap<_, _> = graph
        .vertex_ids()
        .map(|vid: <G as Graph>::VertexId| (vid.clone(), vertex_tag(&vid)))
        .collect();
    let mut vertex_order = vertex_tags.keys().cloned().collect::<Vec<_>>();
    vertex_order.sort_by_key(|vid| &vertex_tags[vid]);

    let edge_tags: HashMap<_, _> = graph
        .edge_ids()
        .map(|eid: <G as Graph>::EdgeId| {
            let (from, to) = graph.edge_ends(eid.clone());
            (eid.clone(), (&vertex_tags[&from], &vertex_tags[&to]))
        })
        .collect();
    let mut edge_order = edge_tags.keys().cloned().collect::<Vec<_>>();
    edge_order.sort_by_key(|eid| &edge_tags[eid]);

    fmt.debug_struct(name)
        .field(
            "vertices",
            &VertexDebug {
                graph,
                vertex_order: &vertex_order,
                vertex_tags: &vertex_tags,
                show_data: show_vertex_data,
            },
        )
        .field(
            "edges",
            &EdgeDebug {
                graph,
                edge_order: &edge_order,
                vertex_tags: &vertex_tags,
                show_data: show_edge_data,
            },
        )
        .finish()
}

#[cfg(test)]
mod tests {
    use crate::{linked_graph::LinkedGraph, *};

    #[test]
    fn test_format_debug() {
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
