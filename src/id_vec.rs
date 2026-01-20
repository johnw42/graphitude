use std::{
    mem::MaybeUninit,
    ops::{Index, IndexMut},
};

use bitvec::vec::BitVec;

/// A vector-like structure that allows for stable IDs even after removals.
pub struct IdVec<T> {
    vec: Vec<MaybeUninit<T>>,
    liveness: BitVec,
    id_offset: usize,
}

/// An index into an `IdVec`. Stable across insertions and removals, but not
/// across compactions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdVecIndex(usize);

impl<T> IdVec<T> {
    /// Create a new, empty IdVec.
    pub fn new() -> Self {
        IdVec {
            vec: Vec::new(),
            liveness: BitVec::new(),
            id_offset: 0,
        }
    }

    /// Gets the number of live entries in the `IdVec`.
    pub fn len(&self) -> usize {
        self.liveness.count_ones()
    }

    /// Checks if the `IdVec` is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Gets the capacity of the `IdVec`.
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted.
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
        self.liveness.reserve(additional);
    }

    /// Reserves the exact capacity for `additional` more elements to be inserted.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.vec.reserve_exact(additional);
        self.liveness.reserve_exact(additional);
    }

    /// Insert a new value into the `IdVec`, returning its index.
    pub fn insert(&mut self, value: T) -> IdVecIndex {
        let next_id = self.next_id();
        self.vec.push(MaybeUninit::new(value));
        self.liveness.push(true);
        IdVecIndex(next_id)
    }

    /// Removes the value at the given index from the `IdVec`, returning it.
    ///
    /// Panics if the index is not live.
    pub fn remove(&mut self, index: IdVecIndex) -> T {
        let index = self.decode_index(index);
        self.liveness.set(index, false);
        unsafe { self.vec[index].as_ptr().read() }
    }

    /// Compact the `IdVec` by removing all dead entries and shifting live entries
    /// down to fill the gaps.  This invalidates all existing indices.
    /// No memory is reallocated.
    pub fn compact(&mut self) {
        let new_id_offset = self.next_id();
        let mut di = 0;
        for si in self.id_offset..self.liveness.len() {
            if self.liveness[si] {
                self.vec[di] = unsafe { MaybeUninit::new(self.vec[si].assume_init_read()) };
                self.liveness.set(di, true);
                di += 1;
            }
        }
        self.vec.truncate(di);
        self.liveness.truncate(di);
        self.id_offset = new_id_offset;
    }

    /// Compact the `IdVec` by removing all dead entries without shifting live entries.
    /// This invalidates all existing indices.  Memory is reallocated to fit exactly.
    pub fn compact_exact(&mut self) {
        if self.len() == self.vec.len() + self.id_offset {
            return;
        }

        let new_id_offset = self.next_id();
        let mut new_vec: Vec<MaybeUninit<T>> = Vec::with_capacity(self.vec.len());
        let mut new_liveness = BitVec::with_capacity(self.liveness.len());

        for (i, live) in self.liveness.iter().enumerate() {
            if *live {
                new_vec.push(unsafe { MaybeUninit::new(self.vec[i].assume_init_read()) });
                new_liveness.push(true);
            }
        }

        self.vec = new_vec;
        self.liveness = new_liveness;
        self.id_offset = new_id_offset;
    }

    /// Iterates over all live indices in the `IdVec`.
    pub fn iter_indices(&self) -> impl Iterator<Item = IdVecIndex> + '_ {
        Self::iter_raw_indices(&self.liveness).map(|index| IdVecIndex(index + self.id_offset))
    }

    /// Iterates over all live values in the `IdVec`.
    pub fn iter(&self) -> impl Iterator<Item = &T> + '_ {
        self.iter_pairs().map(|(_, value)| value)
    }

    /// Iterates mutably over all live values in the `IdVec`.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> + '_ {
        self.iter_pairs_mut().map(|(_, value)| value)
    }

    /// Iterates over all live values in the `IdVec` with their indices.
    pub fn iter_pairs(&self) -> impl Iterator<Item = (IdVecIndex, &T)> + '_ {
        Self::iter_raw_indices(&self.liveness).map(|index| {
            (IdVecIndex(index + self.id_offset), unsafe {
                &*self.vec[index].as_ptr()
            })
        })
    }

    /// Iterates mutably over all live values in the `IdVec` with their indices.
    pub fn iter_pairs_mut(&mut self) -> impl Iterator<Item = (IdVecIndex, &mut T)> + '_ {
        Self::iter_raw_indices(&self.liveness).map(|index| {
            (IdVecIndex(index + self.id_offset), unsafe {
                &mut *self.vec[index].as_mut_ptr()
            })
        })
    }

    fn next_id(&self) -> usize {
        self.vec.len() + self.id_offset
    }

    fn decode_index(&self, index: IdVecIndex) -> usize {
        let result = index.0 - self.id_offset;
        assert!(self.liveness[result], "Index {:?} is not live", index);
        result
    }

    fn iter_raw_indices(liveness: &BitVec) -> impl Iterator<Item = usize> + '_ {
        liveness
            .iter()
            .enumerate()
            .filter_map(move |(i, live)| if *live { Some(i) } else { None })
    }
}

impl<T> Index<IdVecIndex> for IdVec<T> {
    type Output = T;

    fn index(&self, index: IdVecIndex) -> &Self::Output {
        let index = self.decode_index(index);
        unsafe { &*self.vec[index].as_ptr() }
    }
}

impl<T> IndexMut<IdVecIndex> for IdVec<T> {
    fn index_mut(&mut self, index: IdVecIndex) -> &mut Self::Output {
        let index = self.decode_index(index);
        unsafe { &mut *self.vec[index].as_mut_ptr() }
    }
}

impl<T> Drop for IdVec<T> {
    fn drop(&mut self) {
        for (i, live) in self.liveness.iter().enumerate() {
            if *live {
                unsafe {
                    self.vec[i].assume_init_drop();
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let vec: IdVec<i32> = IdVec::new();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_insert_and_index() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(42);
        let id2 = vec.insert(100);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[id1], 42);
        assert_eq!(vec[id2], 100);
    }

    #[test]
    fn test_remove() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(42);
        let id2 = vec.insert(100);

        assert_eq!(vec.remove(id1), 42);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec[id2], 100);
    }

    #[test]
    fn test_insert_and_remove() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(42);
        let id2 = vec.insert(43);
        let id3 = vec.insert(44);

        assert_eq!(vec.remove(id2), 43);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[id1], 42);
        assert_eq!(vec[id3], 44);
    }

    #[test]
    #[should_panic]
    fn test_remove_dead_panics() {
        let mut vec = IdVec::new();
        let id = vec.insert(42);
        vec.remove(id);
        let _ = vec.remove(id);
    }

    #[test]
    fn test_iter() {
        let mut vec = IdVec::new();
        vec.insert(1);
        vec.insert(2);
        vec.insert(3);

        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_iter_mut() {
        let mut vec = IdVec::new();
        vec.insert(1);
        vec.insert(2);

        for val in vec.iter_mut() {
            *val *= 2;
        }

        let sum: i32 = vec.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_compact() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);
        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(id3.0, 2);

        vec.remove(id2);
        vec.compact();

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.iter().sum::<i32>(), 4);

        let id4 = vec.insert(4);
        assert_eq!(vec.len(), 3);
        assert_eq!(id4.0, 5);
        assert_eq!(vec[id4], 4);
        assert_eq!(
            vec.iter_indices().map(|id| id.0).collect::<Vec<_>>(),
            vec![3, 4, 5]
        );
    }

    #[test]
    fn test_compact_exact() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);
        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(id3.0, 2);

        vec.remove(id1);
        vec.remove(id3);
        vec.compact_exact();

        assert_eq!(vec.len(), 1);

        let id4 = vec.insert(4);
        assert_eq!(vec.len(), 2);
        assert_eq!(id4.0, 4);
        assert_eq!(vec[id4], 4);
        assert_eq!(
            vec.iter_indices().map(|id| id.0).collect::<Vec<_>>(),
            vec![3, 4]
        );
    }

    #[test]
    fn test_reserve() {
        let mut vec: IdVec<i32> = IdVec::new();
        let old_capacity = vec.capacity();
        vec.reserve(100);

        assert!(vec.capacity() >= old_capacity + 100);
    }

    #[test]
    fn test_iter_indices() {
        let mut vec = IdVec::new();
        let _id1 = vec.insert(10);
        let id2 = vec.insert(20);
        let _id3 = vec.insert(30);

        vec.remove(id2);

        let indices: Vec<_> = vec.iter_indices().collect();
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn test_index_mut() {
        let mut vec = IdVec::new();
        let id = vec.insert(42);

        vec[id] = 100;
        assert_eq!(vec[id], 100);
    }

    #[test]
    fn test_mixed_operations() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let _id3 = vec.insert(3);
        let id4 = vec.insert(4);

        vec.remove(id2);
        let id5 = vec.insert(5);
        vec.remove(id4);

        assert_eq!(vec.len(), 3);
        assert!(vec.iter_indices().any(|id| id == id1));
        assert!(vec.iter_indices().any(|id| id == id5));
    }

    #[test]
    #[should_panic]
    fn test_index_panics_on_dead_id() {
        let mut vec = IdVec::new();
        let id = vec.insert(42);
        vec.remove(id);
        let _ = vec[id];
    }

    #[test]
    #[should_panic]
    fn test_index_mut_panics_on_dead_id() {
        let mut vec = IdVec::new();
        let id = vec.insert(42);
        vec.remove(id);
        vec[id] = 100;
    }

    #[test]
    fn test_multiple_inserts_after_removals() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);

        vec.remove(id1);
        vec.remove(id3);

        let id4 = vec.insert(4);
        let id5 = vec.insert(5);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec[id2], 2);
        assert_eq!(vec[id4], 4);
        assert_eq!(vec[id5], 5);
    }

    #[test]
    #[should_panic]
    fn test_compact_invalidates_all_indices() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);

        vec.remove(id1);
        vec.compact();

        let _ = vec[id2];
    }

    #[test]
    fn test_iter_pairs() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(10);
        let id2 = vec.insert(20);
        let id3 = vec.insert(30);

        vec.remove(id2);

        let pairs: Vec<_> = vec.iter_pairs().collect();
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], (id1, &10));
        assert_eq!(pairs[1], (id3, &30));
    }

    #[test]
    fn test_iter_pairs_mut() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(10);
        let id2 = vec.insert(20);
        let id3 = vec.insert(30);

        vec.remove(id2);

        for (id, val) in vec.iter_pairs_mut() {
            *val *= 2;
            assert!(id == id1 || id == id3);
        }

        let pairs: Vec<_> = vec.iter_pairs().collect();
        assert_eq!(pairs.len(), 2);
        assert_eq!(*pairs[0].1, 20);
        assert_eq!(*pairs[1].1, 60);
    }

    #[test]
    fn test_iter_pairs_empty() {
        let vec: IdVec<i32> = IdVec::new();
        let pairs: Vec<_> = vec.iter_pairs().collect();
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_iter_pairs_mut_empty() {
        let mut vec: IdVec<i32> = IdVec::new();
        let pairs: Vec<_> = vec.iter_pairs_mut().collect();
        assert_eq!(pairs.len(), 0);
    }

    #[test]
    fn test_iter_pairs_all_removed() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);

        vec.remove(id1);
        vec.remove(id2);
        vec.remove(id3);

        let pairs: Vec<_> = vec.iter_pairs().collect();
        assert_eq!(pairs.len(), 0);
    }
}
