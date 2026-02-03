/// Marker type representing single edges (no multiple edges between same nodes).
pub struct SingleEdge;

/// Marker type representing multiple edges (multiple edges allowed between same nodes).
pub struct MultipleEdges;

/// Trait defining the edge multiplicity behavior of graphs.
///
/// This trait is implemented by [`SingleEdge`] and [`MultipleEdges`] marker types
/// to provide compile-time specialization of graph behavior.
pub trait EdgeMultiplicity: Sized {
    fn allows_parallel_edges() -> bool;
}

impl EdgeMultiplicity for SingleEdge {
    fn allows_parallel_edges() -> bool {
        false
    }
}

impl EdgeMultiplicity for MultipleEdges {
    fn allows_parallel_edges() -> bool {
        true
    }
}
