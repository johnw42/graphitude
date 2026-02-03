use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use bitvec::vec::BitVec;

use super::symmetric_maxtrix_indexing::SymmetricMatrixIndexing;
use crate::SortedPair;
use crate::triangular::triangular;

use crate::adjacency_matrix::{AdjacencyMatrix, BitvecStorage, Symmetric};

/// Bitvec-based symmetric adjacency matrix for undirected graphs.
///
/// Uses a packed triangular matrix representation where only the upper triangle
/// is stored, providing memory-efficient storage for undirected graphs.
/// Requires indices that can be converted to/from usize.
pub struct SymmetricBitvecAdjacencyMatrix<I, V> {
    data: Vec<MaybeUninit<V>>,
    liveness: BitVec,
    indexing: SymmetricMatrixIndexing,
    index_type: PhantomData<I>,
}

impl<I, V> SymmetricBitvecAdjacencyMatrix<I, V>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq,
{
    pub fn with_size(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let indexing = SymmetricMatrixIndexing::new(capacity);
        let storage_size = indexing.storage_size();
        let mut liveness = BitVec::with_capacity(storage_size);
        liveness.resize(storage_size, false);
        let mut data = Vec::with_capacity(storage_size);
        data.resize_with(storage_size, MaybeUninit::uninit);
        Self {
            data,
            liveness,
            indexing,
            index_type: PhantomData,
        }
    }

    fn get_data_read(&self, index: usize) -> Option<V> {
        self.liveness[index].then(|| self.unchecked_get_data_read(index))
    }

    fn get_data_ref(&self, index: usize) -> Option<&V> {
        self.liveness[index].then(|| self.unchecked_get_data_ref(index))
    }

    fn unchecked_get_data_read(&self, index: usize) -> V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_read() }
    }

    fn unchecked_get_data_ref(&self, index: usize) -> &V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_ref() }
    }
}

impl<I, V> AdjacencyMatrix for SymmetricBitvecAdjacencyMatrix<I, V>
where
    I: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord,
{
    type Index = I;
    type Value = V;
    type Symmetry = Symmetric;
    type Storage = BitvecStorage;

    fn new() -> Self {
        Self {
            liveness: BitVec::new(),
            data: Vec::new(),
            indexing: SymmetricMatrixIndexing::new(0),
            index_type: PhantomData,
        }
    }

    fn insert(&mut self, row: I, col: I, data: V) -> Option<V> {
        let (i1, i2) = SortedPair::from((row.into(), col.into())).into();
        if self.indexing.index(i1, i2).is_none() {
            let required_size = (i2 + 1).next_power_of_two();
            if self.indexing.size() < required_size {
                self.indexing = SymmetricMatrixIndexing::new(required_size);
                let repr_size = self.indexing.storage_size();
                self.liveness.resize(repr_size, false);
                self.data.resize_with(repr_size, MaybeUninit::uninit);
            }
        }
        let index = self.indexing.unchecked_index(i1, i2);
        let old_data = self.get_data_read(index);
        self.liveness.set(index, true);
        self.data[index] = MaybeUninit::new(data);
        old_data
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        self.get_data_ref(self.indexing.index(row.into(), col.into())?)
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        let index = self.indexing.index(row.into(), col.into())?;
        let was_live = self.liveness[index];
        self.liveness.set(index, false);
        was_live.then(|| self.unchecked_get_data_read(index))
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (I, I, &'a V)>
    where
        V: 'a,
    {
        self.liveness.iter_ones().map(|index| {
            let (i1, i2) = self.indexing.coordinates(index).into();
            (i1.into(), i2.into(), self.unchecked_get_data_ref(index))
        })
    }

    fn into_iter(self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        let Self {
            data,
            liveness,
            indexing,
            ..
        } = self;
        liveness
            .into_iter()
            .enumerate()
            .filter_map(move |(index, bit)| {
                if bit {
                    let (i1, i2) = indexing.coordinates(index).into();
                    Some((i1.into(), i2.into(), unsafe {
                        data[index].assume_init_read()
                    }))
                } else {
                    None
                }
            })
    }

    fn entries_in_row<'a>(&'a self, row: I) -> impl Iterator<Item = (I, &'a V)>
    where
        V: 'a,
    {
        let row_idx = row.into();
        self.indexing.row(row_idx).filter_map(move |index| {
            if self.liveness[index] {
                let (i, j) = self.indexing.coordinates(index).into();
                debug_assert!(i <= row_idx);
                debug_assert!(i == row_idx || j == row_idx);
                Some((
                    if i == row_idx { j.into() } else { i.into() },
                    self.unchecked_get_data_ref(index),
                ))
            } else {
                None
            }
        })
    }

    fn entries_in_col<'a>(&'a self, col: I) -> impl Iterator<Item = (I, &'a V)>
    where
        V: 'a,
    {
        self.entries_in_row(col)
    }

    fn clear(&mut self) {
        self.liveness.fill(false);
    }

    fn reserve(&mut self, capacity: usize) {
        let current_capacity = self.indexing.size();
        if current_capacity < capacity {
            let new_indexing = SymmetricMatrixIndexing::new(capacity);
            let new_storage_size = new_indexing.storage_size();

            let mut new_liveness = BitVec::with_capacity(new_storage_size);
            new_liveness.resize(new_storage_size, false);

            let mut new_data = Vec::with_capacity(new_storage_size);
            new_data.resize_with(new_storage_size, MaybeUninit::uninit);

            // Copy existing data to the new storage
            for row in 0..current_capacity {
                for col in 0..=row {
                    if let Some(old_index) = self.indexing.index(row, col) {
                        if self.liveness[old_index] {
                            if let Some(new_index) = new_indexing.index(row, col) {
                                new_liveness.set(new_index, true);
                                // SAFETY: old_index is live, so data at that index is initialized
                                new_data[new_index] = MaybeUninit::new(unsafe {
                                    self.data[old_index].assume_init_read()
                                });
                            }
                        }
                    }
                }
            }

            self.indexing = new_indexing;
            self.liveness = new_liveness;
            self.data = new_data;
        }
    }
}

impl<I, V> Debug for SymmetricBitvecAdjacencyMatrix<I, V>
where
    I: Into<usize>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SymmetricBitvecAdjacencyMatrix {{")?;
        if f.alternate() {
            writeln!(f)?;
            for i in 0..self.indexing.size() {
                write!(f, "    ")?;
                if i >= 20 {
                    writeln!(f, "...")?;
                    break;
                }
                for j in 0..=i {
                    if j > 0 && j % 5 == 0 {
                        write!(f, " ")?;
                    }
                    let index = triangular(i) + j;
                    if self.liveness[index] {
                        write!(f, "1")?;
                    } else {
                        write!(f, "0")?;
                    }
                }
                writeln!(f)?;
            }
            writeln!(f, "}}")?;
        } else {
            for i in 0..self.indexing.size() {
                write!(f, " ")?;
                for j in 0..=i {
                    if i >= 10 {
                        write!(f, "...")?;
                    }
                    let index = triangular(i) + j;
                    if self.liveness[index] {
                        write!(f, "1")?;
                    } else {
                        write!(f, "0")?;
                    }
                }
            }
            write!(f, " }}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.get(0, 1), Some(&()));
        assert_eq!(matrix.get(1, 0), Some(&()));
    }

    #[test]
    fn test_entries_in_row() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(0, 2, ());
        let entries: Vec<_> = matrix.entries_in_row(0).collect();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_entries_in_col() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 2, ());
        matrix.insert(1, 2, ());
        matrix.insert(3, 3, ());
        assert_eq!(
            matrix.entries_in_col(2).collect::<Vec<_>>(),
            vec![(0, &()), (1, &())]
        );
        assert_eq!(matrix.entries_in_col(3).collect::<Vec<_>>(), vec![(3, &())]);
    }

    #[test]
    fn test_entries_in_col2() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.entries_in_col(1).collect::<Vec<_>>(), vec![(0, &())]);
        assert_eq!(matrix.entries_in_col(0).collect::<Vec<_>>(), vec![(1, &())]);
    }

    #[test]
    fn test_insert_duplicate() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        assert_eq!(matrix.insert(0, 1, ()), None);
        assert_eq!(matrix.insert(0, 1, ()), Some(()));
    }

    #[test]
    fn test_remove() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.remove(0, 1), Some(()));
        assert_eq!(matrix.get(0, 1), None);
    }

    #[test]
    fn test_remove_both_directions() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.remove(1, 0), Some(()));
        assert_eq!(matrix.get(0, 1), None);
        assert_eq!(matrix.get(1, 0), None);
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::<_, ()>::new();
        assert_eq!(matrix.remove(0, 1), None);
    }

    #[test]
    fn test_entries() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(3, 2, "b");
        let mut entries: Vec<_> = matrix.iter().collect();
        entries.sort();
        assert_eq!(entries, vec![(0, 1, &"a"), (2, 3, &"b")]);
    }

    #[test]
    fn test_large_indices() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(100, 200, ());
        assert_eq!(matrix.get(100, 200), Some(&()));
        assert_eq!(matrix.get(200, 100), Some(&()));
    }

    #[test]
    fn test_self_loop() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(5, 5, ());
        assert_eq!(matrix.get(5, 5), Some(&()));
    }

    #[test]
    fn test_debug_empty() {
        let matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
        assert_eq!(
            format!("{:?}", matrix),
            "SymmetricBitvecAdjacencyMatrix { }"
        );
    }

    #[test]
    fn test_debug_with_edges() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(1, 2, ());
        matrix.insert(0, 3, ());
        assert_eq!(
            format!("{:?}", matrix),
            "SymmetricBitvecAdjacencyMatrix { 0 10 010 1000 }"
        );
    }

    #[test]
    fn test_debug_alternate() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(2, 2, ());
        matrix.insert(0, 3, ());
        matrix.insert(0, 25, ());
        assert_eq!(
            format!("{:#?}", matrix),
            r#"SymmetricBitvecAdjacencyMatrix {
    0
    10
    001
    1000
    00000
    00000 0
    00000 00
    00000 000
    00000 0000
    00000 00000
    00000 00000 0
    00000 00000 00
    00000 00000 000
    00000 00000 0000
    00000 00000 00000
    00000 00000 00000 0
    00000 00000 00000 00
    00000 00000 00000 000
    00000 00000 00000 0000
    00000 00000 00000 00000
    ...
}
"#
        );
    }

    #[test]
    fn test_large_stress_symmetric() {
        // Insert many edges into the symmetric matrix (undirected), remove
        // them in pseudo-random order, and call reserve to exercise resizing.
        let mut matrix = SymmetricBitvecAdjacencyMatrix::<usize, usize>::new();
        let nodes: usize = 120;
        let mut entries = Vec::new();

        for i in 0..nodes {
            for j in 0..=i {
                // deterministic sparse pattern
                if (i * 29 + j * 13) % 19 == 0 {
                    matrix.insert(i, j, i * nodes + j);
                    if ((i + j) % 7) == 0 {
                        // insert both directions to test symmetry
                        matrix.insert(j, i, i * nodes + j);
                    }
                    entries.push((i, j));
                }
            }
        }

        let mut set: std::collections::HashSet<_> = entries.iter().cloned().collect();
        assert_eq!(matrix.iter().count(), set.len());

        // Remove entries and occasionally reserve a larger size
        let total = set.len();
        for k in 0..total {
            assert!(!set.is_empty());
            let &(a, b) = set.iter().next().unwrap();
            set.remove(&(a, b));

            let removed = matrix.remove(a, b).expect("expected present");
            assert_eq!(removed, a * nodes + b);

            if k % 60 == 0 {
                matrix.reserve(nodes + 32);
                for &(x, y) in set.iter() {
                    // undirected: both directions should be accessible
                    assert!(matrix.get(x, y).is_some());
                    assert!(matrix.get(y, x).is_some());
                }
            }
        }

        assert_eq!(matrix.iter().count(), 0);
    }
}
