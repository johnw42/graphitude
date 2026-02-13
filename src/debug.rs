use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use crate::{
    prelude::*,
    util::{FormatDebugAs, FormatDebugWith},
};

/// Formats a graph for debug output with automatic node numbering.
///
/// Nodes are labeled with sequential numbers (0, 1, 2, ...) and both node and edge data are displayed.
pub fn format_debug<'g, 'f, G>(
    graph: &'g G,
    fmt: &mut Formatter<'f>,
    name: &str,
) -> std::fmt::Result
where
    G: Graph,
    G::NodeData: Debug,
    G::EdgeData: Debug,
{
    let node_tags: HashMap<_, _> = graph
        .node_ids()
        .enumerate()
        .map(|(i, nid)| (nid.clone(), i.to_string()))
        .collect();
    format_debug_with(
        graph,
        fmt,
        name,
        &mut |nid| node_tags[nid].clone(),
        &|nid| graph.node_data(&nid),
        &|eid| graph.edge_data(&eid),
    )
}

/// Formats a graph for debug output with customizable node labels and data formatters.
///
/// # Arguments
/// * `graph` - The graph to format
/// * `fmt` - The formatter to write to
/// * `name` - The name to display for the graph type
/// * `node_tag` - A function to generate labels for node IDs
/// * `node_data_fn` - Function to format node data; data is omitted if NodeData is zero-sized
/// * `edge_data_fn` - Function to format edge data; data is omitted if EdgeData is zero-sized
pub fn format_debug_with<'g, 'f, G, T, N, E, NF, EF>(
    graph: &'g G,
    fmt: &mut Formatter<'f>,
    name: &str,
    node_tag: &mut T,
    node_data_fn: &NF,
    edge_data_fn: &EF,
) -> std::fmt::Result
where
    G: Graph,
    T: FnMut(&G::NodeId) -> String,
    NF: Fn(G::NodeId) -> N,
    EF: Fn(G::EdgeId) -> E,
    N: Debug,
    E: Debug,
{
    let node_tags: HashMap<_, _> = graph
        .node_ids()
        .map(|nid: <G as Graph>::NodeId| (nid.clone(), node_tag(&nid)))
        .collect();
    let mut node_order = node_tags.keys().cloned().collect::<Vec<_>>();
    node_order.sort_by_key(|nid| &node_tags[nid]);

    let edge_tags: HashMap<_, _> = graph
        .edge_ids()
        .map(|eid: <G as Graph>::EdgeId| {
            let (from, to) = (eid.source(), eid.target());
            (eid.clone(), (&node_tags[&from], &node_tags[&to]))
        })
        .collect();
    let mut edge_order = edge_tags.keys().cloned().collect::<Vec<_>>();
    edge_order.sort_by_key(|eid| &edge_tags[eid]);

    fmt.debug_struct(name)
        .field(
            "nodes",
            &FormatDebugWith(|f: &mut Formatter<'_>| {
                if std::mem::size_of::<N>() == 0 {
                    f.debug_list()
                        .entries(
                            node_order
                                .iter()
                                .map(|nid| FormatDebugAs(node_tags[nid].clone())),
                        )
                        .finish()
                } else {
                    f.debug_map()
                        .entries(node_order.iter().map(|nid| {
                            (
                                FormatDebugAs(node_tags[nid].clone()),
                                node_data_fn(nid.clone()),
                            )
                        }))
                        .finish()
                }
            }),
        )
        .field(
            "edges",
            &FormatDebugWith(|f: &mut Formatter<'_>| {
                let make_edge_tag = |eid: &G::EdgeId| {
                    let (from, to) = eid.ends().into_values();
                    let tag = if graph.is_directed() {
                        format!("{} -> {}", &node_tags[&from], &node_tags[&to])
                    } else {
                        format!("{} -- {}", &node_tags[&from], &node_tags[&to])
                    };
                    FormatDebugAs(tag)
                };

                if std::mem::size_of::<E>() == 0 {
                    f.debug_list()
                        .entries(edge_order.iter().map(make_edge_tag))
                        .finish()
                } else {
                    f.debug_map()
                        .entries(
                            edge_order
                                .iter()
                                .map(|eid| (make_edge_tag(eid), edge_data_fn(eid.clone()))),
                        )
                        .finish()
                }
            }),
        )
        .finish()
}

#[cfg(test)]
mod tests {
    use crate::{linked_graph::LinkedGraph, prelude::*};

    #[cfg(feature = "bitvec")]
    use crate::adjacency_graph::AdjacencyGraph;

    #[test]
    fn test_format_debug() {
        let mut graph = LinkedGraph::<&str, i32, Directed, MultipleEdges>::default();
        let n1 = graph.add_node("A");
        let n2 = graph.add_node("B");
        graph.add_new_edge(&n1, &n2, 10);

        // Single-line output.
        let output = format!("{:?}", &graph);
        let expected = r#"LinkedGraph { nodes: {0: "A", 1: "B"}, edges: {0 -> 1: 10} }"#;
        assert_eq!(output, expected);

        // Multi-line output.
        let output = format!("{:#?}", &graph);
        let expected = r#"LinkedGraph {
    nodes: {
        0: "A",
        1: "B",
    },
    edges: {
        0 -> 1: 10,
    },
}"#;
        assert_eq!(output, expected);
    }

    #[cfg(feature = "bitvec")]
    #[test]
    fn test_format_debug_with_undirected() {
        type UndirectedGraph = AdjacencyGraph<&'static str, i32, Undirected>;
        let mut graph = UndirectedGraph::default();
        let n1 = graph.add_node("B");
        let n2 = graph.add_node("A");
        let n3 = graph.add_node("C");
        graph.add_edge(&n1, &n2, 10);
        graph.add_edge(&n2, &n3, 20);

        let output = format!("{:?}", &graph);

        // Check structure
        assert!(output.starts_with("AdjacencyGraph { nodes: {"));
        assert!(output.contains("edges: {"));

        // Check all nodes are present
        assert!(output.contains(r#""A""#));
        assert!(output.contains(r#""B""#));
        assert!(output.contains(r#""C""#));

        // Check edges use undirected notation (--)
        assert!(output.contains("--"));
        assert!(!output.contains("->"));

        // Check edge data is present
        assert!(output.contains("10"));
        assert!(output.contains("20"));

        // Multi-line output should have proper structure
        let output = format!("{:#?}", &graph);
        assert!(output.starts_with("AdjacencyGraph {\n    nodes: {"));
        assert!(output.contains("edges: {"));
        assert!(output.contains("--"));
        assert!(!output.contains("->"));
    }
}
