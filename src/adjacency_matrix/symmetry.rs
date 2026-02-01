use crate::{
    SortedPair,
    pairs::{OrderedPair, Pair},
};

/// Trait for matrix symmetry types.
///
/// Implemented by [`Symmetric`] and [`Asymmetric`] marker types.
///
pub trait Symmetry {
    /// Pair type used for indexing entries in the adjacency matrix.
    type Pair<T: Ord + Clone>: Pair<T> + Clone;
}

pub struct Symmetric;
pub struct Asymmetric;

impl Symmetry for Symmetric {
    type Pair<T: Ord + Clone> = SortedPair<T>;
}

impl Symmetry for Asymmetric {
    type Pair<T: Ord + Clone> = OrderedPair<T>;
}
