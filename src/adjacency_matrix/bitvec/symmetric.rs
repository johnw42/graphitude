use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use bitvec::vec::BitVec;

use crate::symmetric_maxtrix_indexing::SymmetricMatrixIndexing;
use crate::triangular::triangular;
use crate::util::sort_pair;

use crate::adjacency_matrix::{AdjacencyMatrix, BitvecStorage, Symmetric};

pub struct SymmetricBitvecAdjacencyMatrix<K, V> {
    data: Vec<MaybeUninit<V>>,
    liveness: BitVec,
    indexing: SymmetricMatrixIndexing,
    key: PhantomData<K>,
}

impl<K, V> SymmetricBitvecAdjacencyMatrix<K, V>
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq,
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
            key: PhantomData,
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

impl<V, K> AdjacencyMatrix for SymmetricBitvecAdjacencyMatrix<K, V>
where
    K: Into<usize> + From<usize> + Clone + Copy + Eq + Hash + Ord,
{
    type Key = K;
    type Value = V;
    type Symmetry = Symmetric;
    type Storage = BitvecStorage;

    fn new() -> Self {
        Self {
            liveness: BitVec::new(),
            data: Vec::new(),
            indexing: SymmetricMatrixIndexing::new(0),
            key: PhantomData,
        }
    }

    fn insert(&mut self, row: K, col: K, data: V) -> Option<V> {
        let (k1, k2) = sort_pair(row.into(), col.into());
        if self.indexing.index(k1.into(), k2.into()).is_none() {
            let required_size = (k2 + 1).next_power_of_two();
            if self.indexing.storage_size() < required_size {
                self.indexing = SymmetricMatrixIndexing::new(required_size);
                let repr_size = self.indexing.unchecked_index(0, required_size);
                self.liveness.resize(repr_size, false);
                self.data.resize_with(repr_size, MaybeUninit::uninit);
            }
        }
        let index = self.indexing.unchecked_index(k1.into(), k2.into());
        let old_data = self.get_data_read(index);
        self.liveness.set(index, true);
        self.data[index] = MaybeUninit::new(data);
        old_data
    }

    fn get(&self, row: K, col: K) -> Option<&V> {
        self.get_data_ref(self.indexing.index(row.into(), col.into())?)
    }

    fn remove(&mut self, row: K, col: K) -> Option<V> {
        let index = self.indexing.index(row.into(), col.into())?;
        let was_live = self.liveness[index];
        self.liveness.set(index, false);
        was_live.then(|| self.unchecked_get_data_read(index))
    }

    fn entries<'a>(&'a self) -> impl Iterator<Item = (K, K, &'a V)>
    where
        V: 'a,
    {
        self.liveness.iter_ones().map(|index| {
            let (k1, k2) = self.indexing.coordinates(index);
            (k1.into(), k2.into(), self.unchecked_get_data_ref(index))
        })
    }

    fn entry_indices(k1: Self::Key, k2: Self::Key) -> (Self::Key, Self::Key) {
        sort_pair(k1, k2)
    }

    fn entries_in_row<'a>(&'a self, row: K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        let row_idx = row.into();
        self.indexing.row(row_idx).filter_map(move |index| {
            if self.liveness[index] {
                let (i, j) = self.indexing.coordinates(index);
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

    fn entries_in_col<'a>(&'a self, col: K) -> impl Iterator<Item = (K, &'a V)>
    where
        V: 'a,
    {
        self.entries_in_row(col)
    }

    fn clear(&mut self) {
        self.liveness.fill(false);
    }
}

impl<K, V> Debug for SymmetricBitvecAdjacencyMatrix<K, V>
where
    K: Into<usize>,
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
        let mut matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
        assert_eq!(matrix.remove(0, 1), None);
    }

    #[test]
    fn test_entries() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, "a");
        matrix.insert(3, 2, "b");
        let mut entries: Vec<_> = matrix.entries().collect();
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
}
