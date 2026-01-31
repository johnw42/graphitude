use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use crate::{
    EdgeId, Graph,
    util::{FormatDebugAs, FormatDebugWith, sort_pair},
};

/// Formats a graph for debug output with automatic node numbering.
///
/// Nodes are labeled with sequential numbers (0, 1, 2, ...) and both node and edge data are displayed.
pub fn format_debug<'g, G>(graph: &'g G, fmt: &mut Formatter<'_>, name: &str) -> std::fmt::Result
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
        true,
        true,
    )
}

/// Formats a graph for debug output with customizable node labels and data visibility.
///
/// # Arguments
/// * `graph` - The graph to format
/// * `fmt` - The formatter to write to
/// * `name` - The name to display for the graph type
/// * `node_tag` - A function to generate labels for node IDs
/// * `show_edge_data` - Whether to display edge data
/// * `show_node_data` - Whether to display node data
pub fn format_debug_with<'g, G>(
    graph: &'g G,
    fmt: &mut Formatter<'_>,
    name: &str,
    node_tag: &mut impl FnMut(&G::NodeId) -> String,
    show_edge_data: bool,
    show_node_data: bool,
) -> std::fmt::Result
where
    G: Graph,
    G::NodeData: Debug,
    G::EdgeData: Debug,
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
                if show_node_data {
                    f.debug_map()
                        .entries(node_order.iter().map(|nid| {
                            (
                                FormatDebugAs(node_tags[nid].clone()),
                                graph.node_data(nid.clone()),
                            )
                        }))
                        .finish()
                } else {
                    f.debug_list()
                        .entries(node_order.iter().map(|nid| graph.node_data(nid.clone())))
                        .finish()
                }
            }),
        )
        .field(
            "edges",
            &FormatDebugWith(|f: &mut Formatter<'_>| {
                let make_edge_tag = |eid: &G::EdgeId| {
                    let (from, to) = eid.ends();
                    let tag = if graph.is_directed() {
                        format!("{} -> {}", &node_tags[&from], &node_tags[&to])
                    } else {
                        let (t1, t2) = sort_pair(&node_tags[&from], &node_tags[&to]);
                        format!("{} -- {}", t1, t2)
                    };
                    FormatDebugAs(tag)
                };

                if show_edge_data {
                    f.debug_map()
                        .entries(
                            edge_order
                                .iter()
                                .map(|eid| (make_edge_tag(eid), graph.edge_data(eid.clone()))),
                        )
                        .finish()
                } else {
                    f.debug_list()
                        .entries(edge_order.iter().map(make_edge_tag))
                        .finish()
                }
            }),
        )
        .finish()
}

#[cfg(test)]
mod tests {
    use crate::{linked_graph::LinkedGraph, *};

    #[test]
    fn test_format_debug() {
        let mut graph = LinkedGraph::<&str, i32>::new();
        let n1 = graph.add_node("A");
        let n2 = graph.add_node("B");
        graph.add_edge(n1, n2, 10);

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
}
