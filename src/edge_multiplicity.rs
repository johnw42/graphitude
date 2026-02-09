/// Marker type representing single edges (no multiple edges between same nodes).
pub struct SingleEdge;

/// Marker type representing multiple edges (multiple edges allowed between same nodes).
pub struct MultipleEdges;

/// Trait defining the edge multiplicity behavior of graphs.
///
/// This trait is implemented by [`SingleEdge`] and [`MultipleEdges`] marker types
/// to provide compile-time specialization of graph behavior.
pub trait EdgeMultiplicityTrait {
    type Impl: EdgeMultiplicityTrait<Impl = Self>;
    fn allows_parallel_edges() -> bool;
}

impl EdgeMultiplicityTrait for SingleEdge {
    type Impl = SingleEdge;
    fn allows_parallel_edges() -> bool {
        false
    }
}

impl EdgeMultiplicityTrait for MultipleEdges {
    type Impl = MultipleEdges;
    fn allows_parallel_edges() -> bool {
        true
    }
}
