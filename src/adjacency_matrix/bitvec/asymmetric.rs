use std::mem::MaybeUninit;

use bitvec::vec::BitVec;

use crate::adjacency_matrix::AdjacencyMatrix;

pub struct AsymmetricBitvecAdjacencyMatrix<V> {
    data: Vec<MaybeUninit<V>>,
    matrix: BitVec,
    size: usize,
    log2_size: u32,
}

impl<V> AsymmetricBitvecAdjacencyMatrix<V> {
    fn with_size(size: usize) -> Self {
        let capacity = size.next_power_of_two();
        let mut matrix = BitVec::with_capacity(capacity * capacity);
        matrix.resize(capacity * capacity, false);
        let mut data = Vec::with_capacity(capacity * capacity);
        data.resize_with(capacity * capacity, MaybeUninit::uninit);
        AsymmetricBitvecAdjacencyMatrix {
            data,
            matrix,
            size: capacity,
            log2_size: capacity.trailing_zeros(),
        }
    }

    /// Gets the linear index for the edge from `from` to `into`, if within bounds.
    fn index(&self, from: usize, into: usize) -> Option<usize> {
        (from < self.size && into < self.size).then(|| self.unchecked_index(from, into))
    }

    fn is_live(&self, index: usize) -> bool {
        self.matrix[index]
    }

    fn get_data_read(&self, index: usize) -> Option<V> {
        self.is_live(index)
            .then(|| self.unchecked_get_data_read(index))
    }

    fn get_data_ref(&self, index: usize) -> Option<&V> {
        self.is_live(index)
            .then(|| self.unchecked_get_data_ref(index))
    }

    fn unchecked_get_data_read(&self, index: usize) -> V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_read() }
    }

    fn unchecked_get_data_ref(&self, index: usize) -> &V {
        // SAFETY: Caller must ensure that the index is live.
        unsafe { self.data[index].assume_init_ref() }
    }

    /// Gets the linear index for the edge from `from` to `into` without bounds checking.
    fn unchecked_index(&self, from: usize, into: usize) -> usize {
        (from << self.log2_size) + into
    }

    fn coordinates(&self, index: usize) -> (usize, usize) {
        let from = index >> self.log2_size;
        let into = index & (self.size - 1);
        (from, into)
    }
}

impl<V> AdjacencyMatrix for AsymmetricBitvecAdjacencyMatrix<V> {
    type Key = usize;
    type Value = V;

    fn new() -> Self {
        Self {
            matrix: BitVec::new(),
            size: 0,
            log2_size: 0,
            data: Vec::new(),
        }
    }

    fn insert(&mut self, from: usize, into: usize, data: V) -> Option<V> {
        if self.index(from, into).is_none() {
            let required_size = usize::max(from, into) + 1;
            if self.size < required_size {
                let mut new_self = Self::with_size(required_size);
                for row in 0..self.size {
                    let old_start = self.unchecked_index(row, 0);
                    let new_start = new_self.unchecked_index(row, 0);
                    new_self.matrix[new_start..new_start + self.size]
                        .copy_from_bitslice(&self.matrix[old_start..old_start + self.size]);
                    for (col, old_datum) in self.data[old_start..old_start + self.size]
                        .iter_mut()
                        .enumerate()
                    {
                        std::mem::swap(old_datum, &mut new_self.data[new_start + col]);
                    }
                }
                *self = new_self;
            }
        }
        let index = self.unchecked_index(from, into);
        let old_data = self.get_data_read(index);
        self.matrix.set(index, true);
        self.data[index] = MaybeUninit::new(data);
        old_data
    }

    fn get(&self, from: &usize, into: &usize) -> Option<&V> {
        self.get_data_ref(self.index(*from, *into)?)
    }

    fn remove(&mut self, from: &usize, into: &usize) -> Option<V> {
        let index = self.index(*from, *into)?;
        let was_live = self.is_live(index);
        self.matrix.set(index, false);
        was_live.then(|| self.unchecked_get_data_read(index))
    }

    fn edges<'a>(&'a self) -> impl Iterator<Item = (usize, usize, &'a V)>
    where
        V: 'a,
    {
        self.matrix.iter_ones().map(|index| {
            let (from, into) = self.coordinates(index);
            (from, into, self.unchecked_get_data_ref(index))
        })
    }

    fn edges_from<'a>(&'a self, from: &usize) -> impl Iterator<Item = (usize, &'a V)>
    where
        V: 'a,
    {
        let row_start = self.index(*from, 0).expect("Invalid 'from' index");
        let row_end = row_start + self.size;
        self.matrix[row_start..row_end]
            .iter_ones()
            .map(|index| (index, self.unchecked_get_data_ref(index)))
    }

    fn edges_into<'a>(&'a self, into: &usize) -> impl Iterator<Item = (usize, &'a V)>
    where
        V: 'a,
    {
        let (_, into_col) = self.coordinates(*into);
        (0..self.size)
            .filter_map(move |from_row| self.get(&from_row, &into_col).map(|data| (from_row, data)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        assert_eq!(matrix.insert(0, 1, 42), None);
        assert_eq!(matrix.get(&0, &1), Some(&42));
        assert_eq!(matrix.get(&1, &0), None);
    }

    #[test]
    fn test_insert_duplicate() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        assert_eq!(matrix.insert(0, 1, ()), None);
        assert_eq!(matrix.insert(0, 1, ()), Some(()));
    }

    #[test]
    fn test_remove() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        assert_eq!(matrix.remove(&0, &1), Some(()));
        assert_eq!(matrix.get(&0, &1), None);
    }

    #[test]
    fn test_edges() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(2, 3, ());
        let edges: Vec<_> = matrix.edges().collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_edges_from() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, ());
        matrix.insert(0, 3, ());
        let edges: Vec<_> = matrix.edges_from(&0).collect();
        assert_eq!(edges.len(), 2);
        assert!(edges.iter().any(|(to, _)| *to == 1));
        assert!(edges.iter().any(|(to, _)| *to == 3));
    }

    #[test]
    fn test_edges_into() {
        let mut matrix = AsymmetricBitvecAdjacencyMatrix::new();
        matrix.insert(0, 1, "A");
        assert_eq!(matrix.get(&0, &1), Some(&"A"));
        matrix.insert(1, 1, "B");
        assert_eq!(matrix.get(&1, &1), Some(&"B"));
        matrix.insert(3, 1, "C");
        assert_eq!(matrix.get(&3, &1), Some(&"C"));
        let edges: Vec<_> = matrix.edges_into(&1).collect();
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().any(|(from, _)| *from == 0));
        assert!(edges.iter().any(|(from, _)| *from == 1));
        assert!(edges.iter().any(|(from, _)| *from == 3));
    }
}
