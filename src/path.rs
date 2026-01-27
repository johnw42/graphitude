use std::{cmp::Ordering, fmt::Debug, hash::Hash, iter::once};

use crate::{Graph, GraphMut as _, LinkedGraph, debug::format_debug};

/// A path in a graph, represented as a sequence of vertices and the edges that
/// connect them.
pub struct Path<'g, G: Graph> {
    graph: &'g G,
    edges: Vec<G::EdgeId>,
    vertices: Vec<G::VertexId>,
}

impl<'g, G: Graph> Path<'g, G> {
    /// Creates a new path starting at the given vertex.
    pub fn new(graph: &'g G, start: G::VertexId) -> Self {
        Self {
            graph,
            edges: Vec::new(),
            vertices: vec![start],
        }
    }

    /// Returns the first vertex in the path.
    pub fn first_vertex(&self) -> G::VertexId {
        self.vertices.first().expect("Path has no vertices").clone()
    }

    /// Returns the last vertex in the path.
    pub fn last_vertex(&self) -> G::VertexId {
        self.vertices.last().expect("Path has no vertices").clone()
    }

    /// Returns an iterator over the edges in the path.
    pub fn edges(&self) -> impl Iterator<Item = G::EdgeId> + '_ {
        self.edges.iter().cloned()
    }

    /// Returns an iterator over the vertices in the path.
    pub fn vertices(&self) -> impl Iterator<Item = G::VertexId> + '_ {
        self.vertices.iter().cloned()
    }

    /// Returns an iterator over the vertices in the path along with the edges
    /// connecting them. Each item is a tuple of the for `(incoming_edge,
    /// vertex, outgoing_edge)`; the edges are optional but will always be
    /// `Some` except for the first vertex (no incoming edge) and the last
    /// vertex (no outgoing edge).
    pub fn vertices_with_edges(
        &self,
    ) -> impl Iterator<Item = (Option<G::EdgeId>, G::VertexId, Option<G::EdgeId>)> + '_ {
        let incoming = once(None).chain(self.edges.iter().cloned().map(Some));
        let outgoing = self.edges.iter().cloned().map(Some).chain(once(None));
        let vertices = self.vertices.iter().cloned();
        incoming
            .zip(outgoing)
            .zip(vertices)
            .map(|((in_edge, out_edge), vertex)| {
                if let Some(ref e) = in_edge {
                    debug_assert!(self.graph.edge_target(e.clone()) == vertex);
                }
                if let Some(ref e) = out_edge {
                    debug_assert!(self.graph.edge_source(e.clone()) == vertex);
                }
                (in_edge, vertex, out_edge)
            })
    }

    /// Adds an edge to the end of the path, extending it to the edge's target
    /// vertex. Panics if the edge's source vertex does not match the current
    /// last vertex of the path.
    pub fn add_edge(&mut self, edge_id: G::EdgeId) {
        assert_eq!(self.graph.edge_source(edge_id.clone()), self.last_vertex());
        self.edges.push(edge_id.clone());
        self.vertices.push(
            self.graph
                .other_end(edge_id, self.last_vertex())
                .unwrap_or_else(|| self.last_vertex()),
        );
    }

    /// Extends the path by appending all edges from another path. Panics if
    /// the first vertex of the other path does not match the current last
    /// vertex of this path.
    pub fn extend_with(&mut self, other: &Path<'g, G>) {
        for edge_id in other.edges() {
            self.add_edge(edge_id);
        }
    }
}

impl<'g, G> Clone for Path<'g, G>
where
    G: Graph,
{
    fn clone(&self) -> Self {
        Self {
            graph: self.graph,
            edges: self.edges.clone(),
            vertices: self.vertices.clone(),
        }
    }
}

impl<'g, G> PartialEq for Path<'g, G>
where
    G: Graph,
{
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.graph, other.graph)
            && self.edges == other.edges
            && self.vertices == other.vertices
    }
}

impl<'g, G> Eq for Path<'g, G> where G: Graph {}

impl<'g, G> Hash for Path<'g, G>
where
    G: Graph,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (self.graph as *const G).hash(state);
        self.edges.hash(state);
        self.vertices.hash(state);
    }
}

impl<'g, G> PartialOrd for Path<'g, G>
where
    G: Graph,
{
    /// A path is "less than" another path if its last vertex matches the
    /// other's first vertex, and "greater than" if its first vertex matches the
    /// other's last vertex. If neither condition is met and the paths are not
    /// equal, they are considered unordered.
    fn partial_cmp(&self, other: &Path<'g, G>) -> Option<std::cmp::Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if self.last_vertex() == other.first_vertex() {
            Some(Ordering::Less)
        } else if self.first_vertex() == other.last_vertex() {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}

impl<G> Extend<G::EdgeId> for Path<'_, G>
where
    G: Graph,
{
    fn extend<T: IntoIterator<Item = G::EdgeId>>(&mut self, iter: T) {
        for edge_id in iter {
            self.add_edge(edge_id);
        }
    }
}

impl<'g, G> Debug for Path<'g, G>
where
    G: Graph,
    G::VertexId: Debug + Clone,
    G::EdgeId: Debug + Clone,
    G::VertexData: Debug,
    G::EdgeData: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Create a temporary LinkedGraph to leverage `format_debug`.
        let mut temp_graph = LinkedGraph::<&'g G::VertexData, &'g G::EdgeData>::new();
        let mut prev_new_vid = None;
        for (eid, vid, _) in self.vertices_with_edges() {
            let v_data = self.graph.vertex_data(vid.clone());
            let new_vid = temp_graph.add_vertex(v_data);
            if let Some(eid) = eid {
                let e_data = self.graph.edge_data(eid.clone());
                temp_graph.add_edge(prev_new_vid.take().unwrap(), new_vid.clone(), e_data);
            }
            prev_new_vid = Some(new_vid);
        }
        format_debug(&temp_graph, f, "Path")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_path() {
        let mut graph = LinkedGraph::<i32, ()>::new();
        let v1 = graph.add_vertex(1);
        let path = Path::new(&graph, v1);
        assert_eq!(path.first_vertex(), v1);
        assert_eq!(path.last_vertex(), v1);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_vertex(1);
        let v2 = graph.add_vertex(2);
        let e1 = graph.add_edge(v1, v2, ());

        let mut path = Path::new(&graph, v1);
        path.add_edge(e1);

        assert_eq!(path.last_vertex(), v2);
        assert_eq!(path.edges().count(), 1);
        assert_eq!(path.vertices().count(), 2);
    }

    #[test]
    fn test_vertices_with_edges() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_vertex(1);
        let v2 = graph.add_vertex(2);
        let v3 = graph.add_vertex(3);
        let e1 = graph.add_edge(v1.clone(), v2.clone(), ());
        let e2 = graph.add_edge(v2.clone(), v3.clone(), ());
        let mut path = Path::new(&graph, v1.clone());
        path.add_edge(e1);
        path.add_edge(e2);
        let mut iter = path.vertices_with_edges();
        assert_eq!(iter.next(), Some((None, v1.clone(), Some(e1.clone()))));
        assert_eq!(
            iter.next(),
            Some((Some(e1.clone()), v2.clone(), Some(e2.clone())))
        );
        assert_eq!(iter.next(), Some((Some(e2.clone()), v3.clone(), None)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_extend_with() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_vertex("v1");
        let v2 = graph.add_vertex("v2");
        let v3 = graph.add_vertex("v3");
        let e1 = graph.add_edge(v1, v2, "e12");
        let e2 = graph.add_edge(v2, v3, "e23");

        let mut path1 = Path::new(&graph, v1);
        path1.add_edge(e1);

        let mut path2 = Path::new(&graph, v2);
        path2.add_edge(e2);

        path1.extend_with(&path2);

        assert_eq!(path1.vertices().count(), 3);
        assert_eq!(path1.edges().count(), 2);
    }

    #[test]
    fn test_extend() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_vertex("v1");
        let v2 = graph.add_vertex("v2");
        let v3 = graph.add_vertex("v3");
        let e1 = graph.add_edge(v1, v2, "e12");
        let e2 = graph.add_edge(v2, v3, "e23");

        let mut path = Path::new(&graph, v1);
        path.extend(vec![e1, e2]);

        assert_eq!(path.vertices().count(), 3);
        assert_eq!(path.edges().count(), 2);
    }

    #[test]
    fn test_debug() {
        let mut graph = LinkedGraph::new();
        let v1 = graph.add_vertex("v1");
        let v2 = graph.add_vertex("v2");
        let v3 = graph.add_vertex("v3");
        let e1 = graph.add_edge(v1, v2, "e12");
        let e2 = graph.add_edge(v2, v3, "e23");
        let e3 = graph.add_edge(v3, v1, "e31");

        let mut path = Path::new(&graph, v1);
        path.extend(vec![e1, e2, e3]);
        assert_eq!(
            format!("{:?}", path),
            r#"Path { vertices: {0: "v1", 1: "v2", 2: "v3", 3: "v1"}, edges: {0 -> 1: "e12", 1 -> 2: "e23", 2 -> 3: "e31"} }"#
        );
    }
}
