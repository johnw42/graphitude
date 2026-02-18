use crate::{DirectednessTrait, coordinate_pair::CoordinatePair};

/// Bitvec-based adjacency matrix implementations.
pub mod bitvec;

/// Hash-based adjacency matrix implementations.
pub mod hash;

/// Storage types for adjacency matrices.
mod storage;

mod tests;

pub use storage::{BitvecStorage, HashStorage, Storage};

pub(crate) use storage::CompactionCount;

type Pair<M> = CoordinatePair<usize, <M as AdjacencyMatrix>::Directedness>;

/// Trait for adjacency matrix data structures.
///
/// Provides methods for inserting, removing, and querying entries in an adjacency matrix.
/// Supports both symmetric (undirected) and asymmetric (directed) matrix implementations.
pub trait AdjacencyMatrix
where
    Self: Sized,
{
    type Value;
    type Directedness: DirectednessTrait + Default;
    type Storage: Storage;

    /// Creates a new, empty adjacency matrix.
    fn with_size(size: usize) -> Self;

    /// Returns the directedness of the adjacency matrix.
    fn directedness(&self) -> Self::Directedness {
        Self::Directedness::default()
    }

    /// Inserts an entry at `row` and `col` with associated data `data`.
    /// Returns the previous data associated with the entry, if any.
    fn insert(&mut self, row: usize, col: usize, data: Self::Value) -> Option<Self::Value>;

    /// Clears all entries from the adjacency matrix.
    fn clear(&mut self);

    /// Clears all entries in the given row and and the given column.
    fn clear_row_and_column(&mut self, row: usize, col: usize);

    /// Gets a reference to the data associated with the entry at `row` and `col`, if it exists.
    fn get(&self, row: usize, col: usize) -> Option<&Self::Value>;

    /// Gets a mutable reference to the data associated with the entry at `row` and `col`, if it exists.
    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Self::Value>;

    /// Removes the entry at `row` and `col`, returning the associated data if it existed.
    fn remove(&mut self, row: usize, col: usize) -> Option<Self::Value>;

    /// Iterates over all entries in the adjacency matrix. Returns an iterator yielding
    /// `(row, col, data)` tuples.
    fn iter<'a>(&'a self) -> impl Iterator<Item = (usize, usize, &'a Self::Value)>
    where
        Self::Value: 'a;

    /// Iterates over all entries in the adjacency matrix, consuming the matrix.
    /// Returns an iterator yielding `(row, col, data)` tuples.
    ///
    /// This trait does not extend `IntoIterator` directly to allow for more
    /// flexible implementations.
    fn into_iter(self) -> impl Iterator<Item = (usize, usize, Self::Value)>;

    /// Returns the number of entries in the adjacency matrix.
    fn len(&self) -> usize {
        self.iter().count()
    }

    /// Returns `true` if the adjacency matrix contains no entries.
    fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    /// Returns an upper bound on the total number of rows and columns in the
    /// adjacency matrix.
    fn size_bound(&self) -> usize;

    /// For internal use.  Gets the canonical indices for the given indices.  This will return a pair
    /// `(i1, i2)` such that for symmetric matrices, `i1 <= i2`.
    #[doc(hidden)]
    fn entry_indices(i1: usize, i2: usize) -> Pair<Self> {
        Self::Directedness::default().coordinate_pair((i1, i2))
    }

    /// Gets the entry at the given row and col.
    fn entry_at(&self, row: usize, col: usize) -> Option<(Pair<Self>, &'_ Self::Value)> {
        self.get(row, col)
            .map(|data| (Self::entry_indices(row, col), data))
    }

    /// Iterates over all entries in the given row.
    fn entries_in_row(&self, row: usize) -> impl Iterator<Item = (usize, &'_ Self::Value)> + '_;

    /// Iterates over all entries in the given col.
    fn entries_in_col(&self, col: usize) -> impl Iterator<Item = (usize, &'_ Self::Value)> + '_;

    /// Reserves capacity for at least `additional` more rows and columns to be added.
    fn reserve(&mut self, _additional: usize);

    /// Reserves capacity for exactly `additional` more rows and columns to be added.
    fn reserve_exact(&mut self, _additional: usize);

    /// Shrinks the adjacency matrix to fit its current size.
    fn shrink_to_fit(&mut self) {
        let new_size = self
            .iter()
            .fold(0, |size, (row, col, _)| size.max(row.max(col) + 1));
        if new_size == self.size_bound() {
            return;
        }
        let mut new_self = Self::with_size(new_size);
        for (row, col) in self
            .iter()
            .map(|(row, col, _)| (row, col))
            .collect::<Vec<_>>()
        {
            new_self.insert(row, col, self.remove(row, col).unwrap());
        }
        *self = new_self;
    }
}

pub(crate) fn format_debug<M>(
    matrix: &M,
    f: &mut std::fmt::Formatter<'_>,
    name: &str,
) -> std::fmt::Result
where
    M: AdjacencyMatrix,
    M::Value: std::fmt::Debug,
{
    f.debug_struct(name)
        .field("directedness", &matrix.directedness())
        .field("entries", &matrix.iter().collect::<Vec<_>>())
        .finish()
}

#[cfg(test)]
pub mod test {
    use super::*;
    use quickcheck::Arbitrary;

    #[doc(hidden)]
    #[derive(Clone, Debug)]
    pub struct ArbMatrix<M> {
        pub matrix: M,
    }

    impl<M> Arbitrary for ArbMatrix<M>
    where
        M: AdjacencyMatrix + Clone + 'static,
        M::Value: Clone + Arbitrary,
    {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut matrix = M::with_size(g.size());
            if g.size() > 0 {
                for _ in 0..g.size() {
                    let row = usize::arbitrary(g) % g.size();
                    let col = usize::arbitrary(g) % g.size();
                    let data = M::Value::arbitrary(g);
                    matrix.insert(row, col, data);
                }
            }
            ArbMatrix { matrix }
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            let entries = self
                .matrix
                .iter()
                .map(|(r, c, _)| (r, c))
                .collect::<Vec<_>>();
            let matrix = self.matrix.clone();
            Box::new(entries.into_iter().map(move |(r, c)| {
                let mut smaller_matrix = matrix.clone();
                smaller_matrix.remove(r, c);
                smaller_matrix.shrink_to_fit();
                Self {
                    matrix: smaller_matrix,
                }
            }))
        }
    }
}
