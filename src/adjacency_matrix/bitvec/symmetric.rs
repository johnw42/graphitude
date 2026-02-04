use std::{fmt::Debug, hash::Hash, marker::PhantomData, mem::MaybeUninit};

use bitvec::vec::BitVec;

use super::symmetric_maxtrix_indexing::SymmetricMatrixIndexing;
use crate::{
    SortedPair,
    adjacency_matrix::{AdjacencyMatrix, BitvecStorage, Symmetric},
};

/// Bitvec-based symmetric adjacency matrix for undirected graphs.
///
/// Uses a packed triangular matrix representation for data storage, while the
/// liveness bitvec is stored as a full matrix. This preserves memory savings
/// for values while allowing O(1) liveness checks by row/col.
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
        let data_storage_size = indexing.storage_size();
        let full_storage_size = capacity
            .checked_mul(capacity)
            .expect("liveness matrix size overflow");
        let mut liveness = BitVec::with_capacity(full_storage_size);
        liveness.resize(full_storage_size, false);
        let mut data = Vec::with_capacity(data_storage_size);
        data.resize_with(data_storage_size, MaybeUninit::uninit);
        Self {
            data,
            liveness,
            indexing,
            index_type: PhantomData,
        }
    }

    fn liveness_index(&self, row: usize, col: usize) -> usize {
        Self::liveness_index_for(self.indexing.size(), row, col)
    }

    fn is_live(&self, row: usize, col: usize) -> bool {
        let index = self.liveness_index(row, col);
        self.liveness[index]
    }

    fn set_live(&mut self, row: usize, col: usize, live: bool) {
        let index = self.liveness_index(row, col);
        self.liveness.set(index, live);
    }

    fn get_data_read(&self, row: usize, col: usize, data_index: usize) -> Option<V> {
        self.is_live(row, col)
            .then(|| self.unchecked_get_data_read(data_index))
    }

    fn get_data_ref(&self, row: usize, col: usize, data_index: usize) -> Option<&V> {
        self.is_live(row, col)
            .then(|| self.unchecked_get_data_ref(data_index))
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

impl<I, V> SymmetricBitvecAdjacencyMatrix<I, V> {
    fn liveness_index_for(size: usize, row: usize, col: usize) -> usize {
        debug_assert!(row < size, "liveness row out of bounds");
        debug_assert!(col < size, "liveness col out of bounds");
        row * size + col
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
                let old_size = self.indexing.size();
                let old_liveness = std::mem::take(&mut self.liveness);

                self.indexing = SymmetricMatrixIndexing::new(required_size);
                let data_storage_size = self.indexing.storage_size();
                let full_storage_size = required_size
                    .checked_mul(required_size)
                    .expect("liveness matrix size overflow");

                let mut new_liveness = BitVec::with_capacity(full_storage_size);
                new_liveness.resize(full_storage_size, false);
                for row in 0..old_size {
                    for col in 0..old_size {
                        let old_index = Self::liveness_index_for(old_size, row, col);
                        if old_liveness[old_index] {
                            let new_index = Self::liveness_index_for(required_size, row, col);
                            new_liveness.set(new_index, true);
                        }
                    }
                }
                self.liveness = new_liveness;
                self.data
                    .resize_with(data_storage_size, MaybeUninit::uninit);
            }
        }
        let index = self.indexing.unchecked_index(i1, i2);
        let old_data = self.get_data_read(i1, i2, index);
        self.set_live(i1, i2, true);
        if i1 != i2 {
            self.set_live(i2, i1, true);
        }
        self.data[index] = MaybeUninit::new(data);
        old_data
    }

    fn get(&self, row: I, col: I) -> Option<&V> {
        let row = row.into();
        let col = col.into();
        let index = self.indexing.index(row, col)?;
        self.get_data_ref(row, col, index)
    }

    fn remove(&mut self, row: I, col: I) -> Option<V> {
        let row = row.into();
        let col = col.into();
        let index = self.indexing.index(row, col)?;
        let was_live = self.is_live(row, col);
        self.set_live(row, col, false);
        if row != col {
            self.set_live(col, row, false);
        }
        was_live.then(|| self.unchecked_get_data_read(index))
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (I, I, &'a V)>
    where
        V: 'a,
    {
        (0..self.indexing.storage_size()).filter_map(move |index| {
            let (i1, i2) = self.indexing.coordinates(index).into();
            if self.is_live(i1, i2) {
                Some((i1.into(), i2.into(), self.unchecked_get_data_ref(index)))
            } else {
                None
            }
        })
    }

    fn into_iter(self) -> impl Iterator<Item = (Self::Index, Self::Index, Self::Value)> {
        let Self {
            data,
            liveness,
            indexing,
            ..
        } = self;
        let size = indexing.size();
        (0..indexing.storage_size()).filter_map(move |index| {
            let (i1, i2) = indexing.coordinates(index).into();
            let live_index = Self::liveness_index_for(size, i1, i2);
            if liveness[live_index] {
                Some((i1.into(), i2.into(), unsafe {
                    data[index].assume_init_read()
                }))
            } else {
                None
            }
        })
    }

    fn entries_in_row(&self, row: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        let row_idx = row.into();
        let size = self.indexing.size();
        let row_start = Self::liveness_index_for(size, row_idx, 0);
        let row_end = row_start + size;
        self.liveness[row_start..row_end]
            .iter_ones()
            .filter_map(move |col| {
                let index = self.indexing.index(row_idx, col)?;
                self.get_data_ref(row_idx, col, index)
                    .map(|data| (col.into(), data))
            })
    }

    fn entries_in_col(&self, col: I) -> impl Iterator<Item = (I, &'_ V)> + '_ {
        self.entries_in_row(col)
    }

    fn clear(&mut self) {
        self.liveness.fill(false);
    }

    fn reserve(&mut self, capacity: usize) {
        let current_capacity = self.indexing.size();
        if current_capacity < capacity {
            let new_indexing = SymmetricMatrixIndexing::new(capacity);
            let new_data_storage_size = new_indexing.storage_size();
            let new_full_storage_size = capacity
                .checked_mul(capacity)
                .expect("liveness matrix size overflow");

            let mut new_liveness = BitVec::with_capacity(new_full_storage_size);
            new_liveness.resize(new_full_storage_size, false);

            let mut new_data = Vec::with_capacity(new_data_storage_size);
            new_data.resize_with(new_data_storage_size, MaybeUninit::uninit);

            // Copy existing data to the new storage
            for row in 0..current_capacity {
                for col in 0..=row {
                    if let Some(old_index) = self.indexing.index(row, col) {
                        if self.is_live(row, col) {
                            if let Some(new_index) = new_indexing.index(row, col) {
                                let idx1 = Self::liveness_index_for(capacity, row, col);
                                new_liveness.set(idx1, true);
                                if row != col {
                                    let idx2 = Self::liveness_index_for(capacity, col, row);
                                    new_liveness.set(idx2, true);
                                }
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
                    let live_index = Self::liveness_index_for(self.indexing.size(), i, j);
                    if self.liveness[live_index] {
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
                    let live_index = Self::liveness_index_for(self.indexing.size(), i, j);
                    if self.liveness[live_index] {
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
    use crate::triangular::triangular;

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

    #[test]
    fn test_reserve_storage_size() {
        let mut matrix = SymmetricBitvecAdjacencyMatrix::<usize, ()>::new();
        let capacity = 7;

        matrix.reserve(capacity);

        let expected_data_storage = triangular(capacity);
        let expected_liveness_storage = capacity * capacity;

        assert_eq!(matrix.indexing.size(), capacity);
        assert_eq!(matrix.data.len(), expected_data_storage);
        assert_eq!(matrix.liveness.len(), expected_liveness_storage);
    }
}
