use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use crate::{EdgeId, Graph, util::sort_pair};

struct NodeTag<'a>(&'a str);

impl<'a> Debug for NodeTag<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct NodeDebug<'a, G: Graph> {
    graph: &'a G,
    node_order: &'a [G::NodeId],
    node_tags: &'a HashMap<G::NodeId, String>,
    show_data: bool,
}

impl<'a, G> Debug for NodeDebug<'a, G>
where
    G: Graph,
    G::NodeData: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.show_data {
            f.debug_map()
                .entries(self.node_order.iter().map(|nid| {
                    (
                        NodeTag(&self.node_tags[nid]),
                        self.graph.node_data(nid.clone()),
                    )
                }))
                .finish()
        } else {
            f.debug_list()
                .entries(
                    self.node_order
                        .iter()
                        .map(|nid| self.graph.node_data(nid.clone())),
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
    node_tags: &'a HashMap<G::NodeId, String>,
    show_data: bool,
}

impl<'a, G> Debug for EdgeDebug<'a, G>
where
    G: Graph,
    G::EdgeData: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let make_edge_tag = |eid: &G::EdgeId| {
            let (from, to) = (eid.source(), eid.target());
            EdgeTag(
                &self.node_tags[&from],
                &self.node_tags[&to],
                self.graph.is_directed(),
            )
        };

        if self.show_data {
            f.debug_map()
                .entries(
                    self.edge_order
                        .iter()
                        .map(|eid| (make_edge_tag(eid), self.graph.edge_data(eid.clone()))),
                )
                .finish()
        } else {
            f.debug_list()
                .entries(self.edge_order.iter().map(make_edge_tag))
                .finish()
        }
    }
}

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
            &NodeDebug {
                graph,
                node_order: &node_order,
                node_tags: &node_tags,
                show_data: show_node_data,
            },
        )
        .field(
            "edges",
            &EdgeDebug {
                graph,
                edge_order: &edge_order,
                node_tags: &node_tags,
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
