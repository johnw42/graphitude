use std::{fmt::Debug, marker::PhantomData};

use crate::{LinkedGraph, prelude::*};

/// A view of a graph with transformed node and edge data, suitable for debugging.
///
/// This type creates a snapshot of a graph with transformed data that can be
/// used for debug formatting. The transformation is applied once during construction,
/// and the result is stored in an internal `LinkedGraph`.
///
/// Note: This view always represents a directed graph, regardless of the source graph's
/// directedness. Edges are copied as-is from the source, so each edge in the source
/// graph becomes one directed edge in the view.
pub struct DebugGraphView<N, E, D: DirectednessTrait, M: EdgeMultiplicityTrait> {
    inner: LinkedGraph<N, E, D>,
    multiplicity: PhantomData<M>,
}

impl<N, E, D, M> DebugGraphView<N, E, D, M>
where
    N: Debug,
    E: Debug,
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    /// Creates a new `DebugGraphView` by transforming the data from the source graph.
    pub fn new<G, NF, EF>(graph: &G, node_fn: NF, edge_fn: EF) -> Self
    where
        G: Graph<Directedness = D, EdgeMultiplicity = M> + ?Sized,
        NF: Fn(&G::NodeData) -> N,
        EF: Fn(&G::EdgeData) -> E,
    {
        let mut inner = LinkedGraph::with_directedness(graph.directedness());
        inner.copy_from_with(graph, node_fn, edge_fn);
        Self {
            inner,
            multiplicity: PhantomData,
        }
    }
}

impl<N, E, D, M> Graph for DebugGraphView<N, E, D, M>
where
    N: Debug,
    E: Debug,
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    type Directedness = D;
    type EdgeMultiplicity = M;
    type NodeData = N;
    type NodeId = <LinkedGraph<N, E, D> as Graph>::NodeId;
    type EdgeData = E;
    type EdgeId = <LinkedGraph<N, E, D> as Graph>::EdgeId;

    fn directedness(&self) -> Self::Directedness {
        self.inner.directedness()
    }

    fn node_ids(&self) -> impl Iterator<Item = Self::NodeId> {
        self.inner.node_ids()
    }

    fn node_data(&self, id: &Self::NodeId) -> &Self::NodeData {
        self.inner.node_data(id)
    }

    fn edge_ids(&self) -> impl Iterator<Item = Self::EdgeId> + '_ {
        self.inner.edge_ids()
    }

    fn edge_data(&self, id: &Self::EdgeId) -> &Self::EdgeData {
        self.inner.edge_data(id)
    }

    fn edges_from_into<'a, 'b: 'a>(
        &'a self,
        from: &'b Self::NodeId,
        to: &'b Self::NodeId,
    ) -> impl Iterator<Item = Self::EdgeId> + 'a {
        self.inner.edges_from_into(from, to)
    }
}

impl<N, E, D, M> Debug for DebugGraphView<N, E, D, M>
where
    N: Debug,
    E: Debug,
    D: DirectednessTrait,
    M: EdgeMultiplicityTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.inner, f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LinkedGraph, Undirected};

    #[test]
    fn test_new_empty_graph() {
        let graph: LinkedGraph<i32, ()> = LinkedGraph::default();
        let view = DebugGraphView::new(&graph, |&n| n, |_| ());

        assert_eq!(view.node_ids().count(), 0);
        assert_eq!(view.edge_ids().count(), 0);
    }

    #[test]
    fn test_new_with_nodes() {
        let mut graph: LinkedGraph<i32, ()> = LinkedGraph::default();
        let _n1 = graph.add_node(10);
        let _n2 = graph.add_node(20);
        let _n3 = graph.add_node(30);

        let view = DebugGraphView::new(&graph, |&n| n, |_| ());

        assert_eq!(view.node_ids().count(), 3);
        let node_data: Vec<i32> = view.node_ids().map(|id| *view.node_data(&id)).collect();
        assert!(node_data.contains(&10));
        assert!(node_data.contains(&20));
        assert!(node_data.contains(&30));
    }

    #[test]
    fn test_new_with_edges() {
        let mut graph: LinkedGraph<&str, i32> = LinkedGraph::default();
        let n1 = graph.add_node("A");
        let n2 = graph.add_node("B");
        let n3 = graph.add_node("C");

        graph.add_new_edge(&n1, &n2, 100);
        graph.add_new_edge(&n2, &n3, 200);

        let view = DebugGraphView::new(&graph, |&s| s, |&e| e);

        assert_eq!(view.edge_ids().count(), 2);
        let edge_data: Vec<i32> = view.edge_ids().map(|id| *view.edge_data(&id)).collect();
        assert!(edge_data.contains(&100));
        assert!(edge_data.contains(&200));
    }

    #[test]
    fn test_node_transformation() {
        let mut graph: LinkedGraph<i32, ()> = LinkedGraph::default();
        graph.add_node(1);
        graph.add_node(2);
        graph.add_node(3);

        // Transform by doubling the values
        let view = DebugGraphView::new(&graph, |&n| n * 2, |_| ());

        let node_data: Vec<i32> = view.node_ids().map(|id| *view.node_data(&id)).collect();
        assert!(node_data.contains(&2));
        assert!(node_data.contains(&4));
        assert!(node_data.contains(&6));
    }

    #[test]
    fn test_edge_transformation() {
        let mut graph: LinkedGraph<(), i32> = LinkedGraph::default();
        let n1 = graph.add_node(());
        let n2 = graph.add_node(());

        graph.add_new_edge(&n1, &n2, 5);

        // Transform by multiplying by 10
        let view = DebugGraphView::new(&graph, |_| (), |&e| e * 10);

        let edge_data: Vec<i32> = view.edge_ids().map(|id| *view.edge_data(&id)).collect();
        assert_eq!(edge_data, vec![50]);
    }

    #[test]
    fn test_type_transformation() {
        let mut graph: LinkedGraph<i32, f64> = LinkedGraph::default();
        let n1 = graph.add_node(42);
        let n2 = graph.add_node(100);

        #[allow(clippy::approx_constant)]
        graph.add_new_edge(&n1, &n2, 3.14);

        // Transform types: i32 -> String, f64 -> bool
        let view = DebugGraphView::new(&graph, |&n| format!("Node_{}", n), |&e| e > 2.0);

        let node_data: Vec<String> = view
            .node_ids()
            .map(|id| view.node_data(&id).clone())
            .collect();
        assert!(node_data.contains(&"Node_42".to_string()));
        assert!(node_data.contains(&"Node_100".to_string()));

        let edge_data: Vec<bool> = view.edge_ids().map(|id| *view.edge_data(&id)).collect();
        assert_eq!(edge_data, vec![true]);
    }

    #[test]
    fn test_edges_between() {
        let mut graph: LinkedGraph<&str, i32> = LinkedGraph::default();
        let n1 = graph.add_node("A");
        let n2 = graph.add_node("B");

        graph.add_new_edge(&n1, &n2, 1);
        graph.add_new_edge(&n1, &n2, 2);

        let view = DebugGraphView::new(&graph, |&s| s, |&e| e);

        let node_ids: Vec<_> = view.node_ids().collect();
        let edges_between: Vec<_> = view.edges_from_into(&node_ids[0], &node_ids[1]).collect();
        assert_eq!(edges_between.len(), 2);
    }

    #[test]
    fn test_debug_format_directed() {
        let mut graph: LinkedGraph<i32, &str> = LinkedGraph::default();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);

        graph.add_new_edge(&n1, &n2, "edge");

        let view = DebugGraphView::new(&graph, |&n| n, |&e| e);

        let debug_output = format!("{:?}", view);
        assert!(debug_output.contains("LinkedGraph"));
        assert!(debug_output.contains("->"));
        assert!(!debug_output.contains("--"));
    }

    #[test]
    fn test_debug_format_undirected() {
        let mut graph: LinkedGraph<i32, &str, Undirected> = LinkedGraph::default();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);

        graph.add_new_edge(&n1, &n2, "edge");

        let view = DebugGraphView::new(&graph, |&n| n, |&e| e);

        let debug_output = format!("{:?}", view);
        assert!(debug_output.contains("LinkedGraph"));
        assert!(!debug_output.contains("->"));
        assert!(debug_output.contains("--"));
    }

    #[test]
    fn test_debug_format_alternate() {
        let mut graph: LinkedGraph<i32, &str> = LinkedGraph::default();
        let n1 = graph.add_node(1);
        let n2 = graph.add_node(2);

        graph.add_new_edge(&n1, &n2, "edge");

        let view = DebugGraphView::new(&graph, |&n| n, |&e| e);

        let debug_output = format!("{:#?}", view);
        assert!(debug_output.contains("LinkedGraph"));
    }
}
