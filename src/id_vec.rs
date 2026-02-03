#![allow(unused)]
#![cfg(feature = "bitvec")]

#[cfg(test)]
use std::cell::RefCell;

use std::{
    collections::HashMap,
    fmt::Debug,
    mem::MaybeUninit,
    ops::{Index, IndexMut, Range},
};

use bitvec::vec::BitVec;

use crate::MappingResult;

/// An key for an `IdVec`. Stable across insertions and removals, but not
/// across `compact` or `shrink_to_fit` operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdVecKey(usize);

// Needed for AdjacencyGraph implementation.
impl Into<usize> for IdVecKey {
    fn into(self) -> usize {
        self.0
    }
}

// Needed for AdjacencyGraph implementation.
impl From<usize> for IdVecKey {
    fn from(value: usize) -> Self {
        IdVecKey(value)
    }
}

/// Helper struct for indexing into an `IdVec`.
pub struct IdVecIndexing {
    key_offset: usize,
}

impl IdVecIndexing {
    /// Decodes an `IdVecKey` into an index smaller than the size of the
    /// `IdVec`. Used for correlating items in the `IdVec` with other data
    /// structures that use zero-based indexing, such as adjacency matrices.
    ///
    /// This is used internally to map from the stable key to the internal
    /// vector index.
    pub fn zero_based_index(&self, index: IdVecKey) -> usize {
        index.0 - self.key_offset
    }

    /// Encodes an index returned by `zero_based_index` back into an `IdVecKey`.
    /// Use with caution, because there is no guarantee that the index is valid
    /// unless it came directly from `zero_based_index`, and even then, it may
    /// be a the key of a removed entry.
    pub fn key_from_index(&self, index: usize) -> IdVecKey {
        IdVecKey(index + self.key_offset)
    }
}

/// A map- or bag-like structure that assigns stable keys to inserted values.
/// Keys remain valid across insertions and removals, but not across `compact`
/// or `shrink_to_fit` operations.  Internally uses a `Vec<MaybeUninit<T>>` to
/// store values and a `BitVec` to track live entries.
pub struct IdVec<T> {
    /// Internal vector storing the values.
    vec: Vec<MaybeUninit<T>>,
    /// BitVec tracking which entries are live (not removed).  This is used to
    /// determine which entries contain an initialized value.
    liveness: BitVec,
    /// The difference between indices stored in keys and actual vector indices.
    key_offset: usize,
}

impl<T> IdVec<T> {
    /// Creates a new, empty IdVec.
    pub fn new() -> Self {
        IdVec {
            vec: Vec::new(),
            liveness: BitVec::new(),
            key_offset: 0,
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

    /// Clears the `IdVec`, removing all entries.
    pub fn clear(&mut self) {
        self.vec.clear();
        self.liveness.clear();
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

    /// Inserts a new value into the `IdVec`, returning its key.
    pub fn insert(&mut self, value: T) -> IdVecKey {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let next_key = IdVecKey(self.vec.len() + self.key_offset);
        self.vec.push(MaybeUninit::new(value));
        self.liveness.push(true);
        next_key
    }

    /// Removes the value at the given key from the `IdVec`, returning it if it exists.
    pub fn remove(&mut self, key: IdVecKey) -> Option<T> {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let index = self.zero_based_index(key);
        if index < self.vec.len() && self.liveness[index] {
            self.liveness.set(index, false);
            Some(unsafe { self.vec[index].assume_init_read() })
        } else {
            None
        }
    }

    /// Gets a reference to the item at the given key, if it exists.
    pub fn get(&self, key: IdVecKey) -> Option<&T> {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let index = self.zero_based_index(key);
        if index < self.liveness.len() && self.liveness[index] {
            Some(unsafe { &*self.vec[index].as_ptr() })
        } else {
            None
        }
    }

    /// Gets a reference to the item at the given key, if it exists.
    pub fn get_mut(&mut self, key: IdVecKey) -> Option<&mut T> {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let index = self.zero_based_index(key);
        if index < self.liveness.len() && self.liveness[index] {
            Some(unsafe { &mut *self.vec[index].as_mut_ptr() })
        } else {
            None
        }
    }

    /// Compacts the `IdVec` by removing all dead entries and shifting live
    /// entries down to fill the gaps. This invalidates all existing keys.  No
    /// memory is reallocated.
    pub fn compact(&mut self) {
        self.compact_with(None::<fn(MappingResult<IdVecKey>)>);
    }

    /// Compacts the `IdVec` by removing all dead entries and shifting live
    /// entries down to fill the gaps. This invalidates all existing keys.  No
    /// memory is reallocated.
    ///
    /// Calls the provided callback with MappingResult for each key:
    /// - Remapped(old_key, new_key) for entries that moved
    /// - Deleted(old_key) for removed entries
    /// If no entries were removed, all keys map to themselves, so the callback is not called.
    pub fn compact_with(&mut self, mut callback: Option<impl FnMut(MappingResult<IdVecKey>)>) {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        if self.liveness.all() {
            return;
        }
        let new_key_offset = self.key_offset + self.liveness.len();
        let mut di = 0;
        for si in 0..self.liveness.len() {
            let old_key = IdVecKey(si + self.key_offset);
            if self.liveness[si] {
                let new_key = IdVecKey(di + new_key_offset);
                self.vec[di] = MaybeUninit::new(unsafe { self.vec[si].assume_init_read() });
                self.liveness.set(di, true);
                if let Some(ref mut cb) = callback {
                    cb(MappingResult::Remapped(old_key, new_key));
                }
                di += 1;
            } else {
                if let Some(ref mut cb) = callback {
                    cb(MappingResult::Deleted(old_key));
                }
            }
        }
        self.vec.truncate(di);
        self.liveness.truncate(di);
        self.key_offset = new_key_offset;
    }

    /// Compacts the `IdVec` by removing all dead entries without shifting live
    /// entries.  This invalidates all existing keys. Memory is reallocated to
    /// fit exactly.
    pub fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_with(None::<fn(MappingResult<IdVecKey>)>);
    }

    /// Compacts the `IdVec` by removing all dead entries without shifting live
    /// entries.  This invalidates all existing keys. Memory is reallocated to
    /// fit exactly.
    ///
    /// Calls the provided callback with MappingResult for each key:
    /// - Remapped(old_key, new_key) for entries that moved
    /// - Deleted(old_key) for removed entries
    /// If no entries were removed, all keys map to themselves, so the callback is not called.
    pub fn shrink_to_fit_with(
        &mut self,
        mut callback: Option<impl FnMut(MappingResult<IdVecKey>)>,
    ) {
        if self.len() == self.vec.len() + self.key_offset {
            return;
        }

        let new_key_offset = self.key_offset + self.liveness.len();
        let mut new_vec: Vec<MaybeUninit<T>> = Vec::with_capacity(self.vec.len());
        let mut new_liveness = BitVec::with_capacity(self.liveness.len());

        for (si, live) in self.liveness.iter().enumerate() {
            let old_key = IdVecKey(si + self.key_offset);
            if *live {
                let di = new_vec.len();
                let new_key = IdVecKey(di + new_key_offset);
                new_vec.push(unsafe { MaybeUninit::new(self.vec[si].assume_init_read()) });
                new_liveness.push(true);
                if let Some(ref mut cb) = callback {
                    cb(MappingResult::Remapped(old_key, new_key));
                }
            } else {
                if let Some(ref mut cb) = callback {
                    cb(MappingResult::Deleted(old_key));
                }
            }
        }

        self.vec = new_vec;
        self.liveness = new_liveness;
        self.key_offset = new_key_offset;
    }

    /// Iterates over all live indices in the `IdVec`.
    pub fn iter_keys(&self) -> impl Iterator<Item = IdVecKey> + '_ {
        Self::iter_live_indices(&self.liveness).map(|index| IdVecKey(index + self.key_offset))
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
    pub fn iter_pairs(&self) -> impl Iterator<Item = (IdVecKey, &T)> + '_ {
        Self::iter_live_indices(&self.liveness).map(|index| {
            (IdVecKey(index + self.key_offset), unsafe {
                &*self.vec[index].as_ptr()
            })
        })
    }

    /// Iterates mutably over all live values in the `IdVec` with their indices.
    pub fn iter_pairs_mut(&mut self) -> impl Iterator<Item = (IdVecKey, &mut T)> + '_ {
        Self::iter_live_indices(&self.liveness).map(|index| {
            (IdVecKey(index + self.key_offset), unsafe {
                &mut *self.vec[index].as_mut_ptr()
            })
        })
    }

    pub fn indexing(&self) -> IdVecIndexing {
        IdVecIndexing {
            key_offset: self.key_offset,
        }
    }

    /// Proxy to `IdVecIndexing::zero_based_index`.
    pub fn zero_based_index(&self, index: IdVecKey) -> usize {
        self.indexing().zero_based_index(index)
    }

    /// Proxy to `IdVecIndexing::key_from_index`.
    pub fn key_from_index(&self, index: usize) -> IdVecKey {
        self.indexing().key_from_index(index)
    }

    /// Helper function to iterate over live indices in the BitVec.  Needs to be
    /// a separate function to satisfy the borrow checker for mutable iterators.
    fn iter_live_indices(liveness: &BitVec) -> impl Iterator<Item = usize> + '_ {
        liveness
            .iter()
            .enumerate()
            .filter_map(move |(i, live)| if *live { Some(i) } else { None })
    }
}

impl<T> Debug for IdVec<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut builder = f.debug_struct("IdVec");
        for (index, value) in self.vec.iter().enumerate() {
            builder.field(
                &(index + self.key_offset).to_string(),
                &Box::new(if self.liveness[index] {
                    unsafe { value.assume_init_ref() as &dyn Debug }
                } else {
                    &None::<()> as &dyn Debug
                }),
            );
        }
        builder.finish()
    }
}

#[cfg(test)]
thread_local! {
static DROPPED_ENTRIES: RefCell<Vec<usize>> =
    RefCell::new(Vec::new());
}

impl<T> Drop for IdVec<T> {
    fn drop(&mut self) {
        for i in self.liveness.iter_ones() {
            unsafe {
                #[cfg(test)]
                DROPPED_ENTRIES.with(|dropped_entries| {
                    dropped_entries.borrow_mut().push(i);
                });
                self.vec[i].assume_init_drop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_new() {
        let vec: IdVec<i32> = IdVec::new();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(42);
        let id2 = vec.insert(100);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(id1), Some(&42));
        assert_eq!(vec.get(id2), Some(&100));
    }

    #[test]
    fn test_remove() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(42);
        let id2 = vec.insert(100);

        assert_eq!(vec.remove(id1), Some(42));
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(id2), Some(&100));
    }

    #[test]
    fn test_insert_and_remove() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(42);
        let id2 = vec.insert(43);
        let id3 = vec.insert(44);

        assert_eq!(vec.remove(id2), Some(43));
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(id1), Some(&42));
        assert_eq!(vec.get(id3), Some(&44));
    }

    #[test]
    fn test_remove_dead_returns_none() {
        let mut vec = IdVec::new();
        let id = vec.insert(42);
        vec.remove(id);
        assert!(vec.remove(id).is_none());
    }

    #[test]
    fn test_clear() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);

        vec.remove(id2);
        assert_eq!(vec.len(), 2);
        assert!(!vec.is_empty());

        vec.clear();

        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        // Capacity is preserved after clear (like Vec::clear())
        assert!(vec.get(id1).is_none());
        assert!(vec.get(id3).is_none());

        // Should be able to insert after clearing
        let id4 = vec.insert(4);
        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(id4), Some(&4));
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
        let mut key_map = HashMap::new();

        vec.compact_with(Some(|result| {
            if let MappingResult::Remapped(old_key, new_key) = result {
                key_map.insert(old_key, new_key);
            }
        }));

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.iter().sum::<i32>(), 4);

        // Test the key mapping
        let new_id1 = key_map
            .get(&id1)
            .copied()
            .expect("id1 should have a mapping");
        let new_id3 = key_map
            .get(&id3)
            .copied()
            .expect("id3 should have a mapping");
        assert!(
            key_map.get(&id2).copied().is_none(),
            "id2 was removed, should map to None"
        );

        // Verify we can access values with new keys
        assert_eq!(vec.get(new_id1), Some(&1));
        assert_eq!(vec.get(new_id3), Some(&3));

        let id4 = vec.insert(4);
        assert_eq!(vec.len(), 3);
        assert_eq!(id4.0, 5);
        assert_eq!(vec.get(id4), Some(&4));
        assert_eq!(
            vec.iter_keys().map(|id| id.0).collect::<Vec<_>>(),
            vec![3, 4, 5]
        );
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);
        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
        assert_eq!(id3.0, 2);

        vec.remove(id1);
        vec.remove(id3);
        let mut key_map = HashMap::new();

        vec.shrink_to_fit_with(Some(|result| {
            if let MappingResult::Remapped(old_key, new_key) = result {
                key_map.insert(old_key, new_key);
            }
        }));

        assert_eq!(vec.len(), 1);

        // Test the key mapping
        assert!(
            key_map.get(&id1).copied().is_none(),
            "id1 was removed, should map to None"
        );
        let new_id2 = key_map
            .get(&id2)
            .copied()
            .expect("id2 should have a mapping");
        assert!(
            key_map.get(&id3).copied().is_none(),
            "id3 was removed, should map to None"
        );

        // Verify we can access values with new key
        assert_eq!(vec.get(new_id2), Some(&2));

        let id4 = vec.insert(4);
        assert_eq!(vec.len(), 2);
        assert_eq!(id4.0, 4);
        assert_eq!(vec.get(id4), Some(&4));
        assert_eq!(
            vec.iter_keys().map(|id| id.0).collect::<Vec<_>>(),
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
    fn test_reserve_exact() {
        let mut vec: IdVec<i32> = IdVec::new();
        vec.reserve_exact(50);

        // Capacity should be at least 50
        assert!(vec.capacity() >= 50);

        // Insert elements to verify it works
        for i in 0..10 {
            vec.insert(i);
        }
        assert_eq!(vec.len(), 10);
    }

    #[test]
    fn test_len() {
        let mut vec = IdVec::new();
        assert_eq!(vec.len(), 0);

        vec.insert(1);
        assert_eq!(vec.len(), 1);

        let id2 = vec.insert(2);
        assert_eq!(vec.len(), 2);

        vec.remove(id2);
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn test_is_empty() {
        let mut vec: IdVec<i32> = IdVec::new();
        assert!(vec.is_empty());

        let id = vec.insert(1);
        assert!(!vec.is_empty());

        vec.remove(id);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_capacity() {
        let mut vec: IdVec<i32> = IdVec::new();
        let initial_capacity = vec.capacity();

        vec.reserve(100);
        assert!(vec.capacity() > initial_capacity);
        assert!(vec.capacity() >= 100);
    }

    #[test]
    fn test_iter_keys() {
        let mut vec = IdVec::new();
        let _id1 = vec.insert(10);
        let id2 = vec.insert(20);
        let _id3 = vec.insert(30);

        vec.remove(id2);

        let indices: Vec<_> = vec.iter_keys().collect();
        assert_eq!(indices.len(), 2);
    }

    #[test]
    fn test_get_mut() {
        let mut vec = IdVec::new();
        let id = vec.insert(42);

        let val = vec.get_mut(id).unwrap();
        *val = 100;
        assert_eq!(vec.get(id), Some(&100));
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
        assert!(vec.iter_keys().any(|id| id == id1));
        assert!(vec.iter_keys().any(|id| id == id5));
    }

    #[test]
    fn test_get_returns_none_on_dead_id() {
        let mut vec = IdVec::new();
        let id = vec.insert(42);
        vec.remove(id);
        assert!(vec.get(id).is_none());
    }

    #[test]
    fn test_get_mut_returns_none_on_dead_id() {
        let mut vec = IdVec::new();
        let id = vec.insert(42);
        vec.remove(id);
        assert!(vec.get_mut(id).is_none());
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
        assert_eq!(vec.get(id2), Some(&2));
        assert_eq!(vec.get(id4), Some(&4));
        assert_eq!(vec.get(id5), Some(&5));
    }

    #[test]
    #[should_panic]
    fn test_compact_invalidates_all_indices() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);

        vec.remove(id1);
        vec.compact_with(Some(|_result| {}));

        let _ = vec.get(id2);
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

    #[test]
    fn test_idveckey_into_usize() {
        let key = IdVecKey(42);
        let value: usize = key.into();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_idveckey_from_usize() {
        let key = IdVecKey::from(123);
        assert_eq!(key, IdVecKey(123));
    }

    #[test]
    fn test_compact_with_callback_after_removals() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);

        vec.remove(id2);

        let mut key_map = HashMap::new();
        vec.compact_with(Some(|result| {
            if let MappingResult::Remapped(old_key, new_key) = result {
                key_map.insert(old_key, new_key);
            }
        }));

        assert_eq!(vec.get(key_map[&id1]), Some(&1));
        assert_eq!(vec.get(key_map[&id3]), Some(&3));
    }

    #[test]
    fn test_compact_with_callback_no_removals() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        vec.insert(2);
        vec.insert(3);

        // shrink_to_fit with no removals does nothing, callback not called
        let mut callback_called = false;

        vec.compact_with(Some(|_result| {
            callback_called = true;
        }));

        // Callback should not be called for identity case
        assert!(!callback_called);
        // Old keys should still work
        assert_eq!(vec.get(id1), Some(&1));
    }

    #[test]
    fn test_shrink_to_fit_with_callback_no_removals() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        vec.insert(2);
        vec.insert(3);

        // shrink_to_fit with no removals does nothing, callback not called
        let mut callback_called = false;

        vec.shrink_to_fit_with(Some(|_result| {
            callback_called = true;
        }));

        // Callback should not be called for identity case
        assert!(!callback_called);
        // Old keys should still work
        assert_eq!(vec.get(id1), Some(&1));
    }

    #[test]
    fn test_shrink_to_fit_with_callback_identity_case() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);

        // shrink_to_fit with no removals does nothing, callback not called
        let mut callback_called = false;

        vec.shrink_to_fit_with(Some(|_result| {
            callback_called = true;
        }));

        // Callback should not be called for identity case
        assert!(!callback_called);
        // Old keys should still work
        assert_eq!(vec.get(id1), Some(&1));
        assert_eq!(vec.get(id2), Some(&2));
    }

    // Test compaction with multiple removals and insertions and multiple rounds of compaction.
    #[test]
    fn test_multiple_compactions() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);
        let id4 = vec.insert(4);
        let id5 = vec.insert(5);

        // First round of removals
        vec.remove(id2);
        vec.remove(id4);

        let mut key_map = HashMap::new();
        vec.compact_with(Some(|result| {
            if let MappingResult::Remapped(old_key, new_key) = result {
                key_map.insert(old_key, new_key);
            }
        }));

        // Update keys after first compaction
        let new_id1 = key_map[&id1];
        let new_id3 = key_map[&id3];
        let new_id5 = key_map[&id5];

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(new_id1), Some(&1));
        assert_eq!(vec.get(new_id3), Some(&3));
        assert_eq!(vec.get(new_id5), Some(&5));

        // Insert more values
        let id6 = vec.insert(6);
        let id7 = vec.insert(7);

        // Second round of removals
        vec.remove(new_id3);
        vec.remove(id6);

        key_map.clear();
        vec.compact_with(Some(|result| {
            if let MappingResult::Remapped(old_key, new_key) = result {
                key_map.insert(old_key, new_key);
            }
        }));

        // Update keys after second compaction
        let final_id1 = key_map[&new_id1];
        let final_id5 = key_map[&new_id5];
        let final_id7 = key_map[&id7];

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(final_id1), Some(&1));
        assert_eq!(vec.get(final_id5), Some(&5));
        assert_eq!(vec.get(final_id7), Some(&7));
    }

    #[test]
    fn test_multiple_shrink_to_fits() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);
        let id4 = vec.insert(4);
        let id5 = vec.insert(5);

        // First round of removals
        vec.remove(id2);
        vec.remove(id4);

        let mut key_map = HashMap::new();
        vec.shrink_to_fit_with(Some(|result| {
            if let MappingResult::Remapped(old_key, new_key) = result {
                key_map.insert(old_key, new_key);
            }
        }));

        // Update keys after first shrink_to_fit
        let new_id1 = key_map[&id1];
        let new_id3 = key_map[&id3];
        let new_id5 = key_map[&id5];

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(new_id1), Some(&1));
        assert_eq!(vec.get(new_id3), Some(&3));
        assert_eq!(vec.get(new_id5), Some(&5));

        // Insert more values
        let id6 = vec.insert(6);
        let id7 = vec.insert(7);

        // Second round of removals
        vec.remove(new_id3);
        vec.remove(id6);

        key_map.clear();
        vec.shrink_to_fit_with(Some(|result| {
            if let MappingResult::Remapped(old_key, new_key) = result {
                key_map.insert(old_key, new_key);
            }
        }));

        // Update keys after second shrink_to_fit
        let final_id1 = key_map[&new_id1];
        let final_id5 = key_map[&new_id5];
        let final_id7 = key_map[&id7];

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.get(final_id1), Some(&1));
        assert_eq!(vec.get(final_id5), Some(&5));
        assert_eq!(vec.get(final_id7), Some(&7));
    }

    #[test]
    fn test_drop() {
        DROPPED_ENTRIES.with_borrow_mut(|dropped_entries| {
            dropped_entries.clear();
        });

        let mut vec = IdVec::new();
        vec.insert(1);
        vec.insert(2);
        let id3 = vec.insert(3);
        vec.insert(4);
        vec.remove(id3);
        drop(vec);

        DROPPED_ENTRIES.with_borrow_mut(|dropped_entries| {
            dropped_entries.sort();
            assert_eq!(*dropped_entries, vec![0, 1, 3]);
        });
    }

    #[test]
    fn test_idvec_large() {
        // Insert a large number of entries, remove many of them in
        // pseudo-random order, and occasionally compact while tracking
        // remappings. Inspired by the large-graph deconstruction test.
        let mut vec = IdVec::new();
        let mut map = std::collections::HashMap::new();
        let mut keys = Vec::new();
        let total: usize = 1000;

        for i in 0..total {
            let k = vec.insert(i);
            map.insert(k, i);
            keys.push(k);
        }

        assert_eq!(vec.len(), total);

        let mut key_set: std::collections::HashSet<_> = keys.into_iter().collect();
        let mut removed = std::collections::HashSet::new();

        for i in 0..total {
            assert!(!key_set.is_empty());
            let _num = key_set.len();
            // pick an arbitrary key (HashSet iteration order is effectively random)
            let key = *key_set.iter().next().unwrap();
            key_set.remove(&key);

            let val = vec.remove(key).expect("expected present value");
            assert_eq!(val, map.remove(&key).unwrap());
            removed.insert(key);

            // Compact periodically and update tracked keys according to remap
            if i % 100 == 0 {
                let len_before = vec.len();
                vec.compact_with(Some(|r| {
                    match r {
                        MappingResult::Remapped(old, new) => {
                            let was_present = key_set.remove(&old);
                            // old may already have been removed earlier
                            assert!(was_present || removed.contains(&old));
                            // Update key_set and also move any stored mapping in `map`.
                            let inserted = key_set.insert(new);
                            assert!(inserted);
                            if let Some(v) = map.remove(&old) {
                                map.insert(new, v);
                            }
                        }
                        MappingResult::Deleted(old) => {
                            // Deleted entries should previously have been removed
                            assert!(removed.contains(&old));
                        }
                        _ => {}
                    }
                }));

                assert_eq!(vec.len(), len_before);
                for k in key_set.iter() {
                    assert!(vec.get(*k).is_some());
                }
            }
        }

        assert_eq!(vec.len(), 0);
    }
}
