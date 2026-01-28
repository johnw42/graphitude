#![allow(unused)]
#![cfg(feature = "bitvec")]

use std::{
    collections::HashMap,
    mem::MaybeUninit,
    ops::{Index, IndexMut, Range},
};

use bitvec::vec::BitVec;

/// An index into an `IdVec`. Stable across insertions and removals, but not
/// across shrink_to_fit operations.
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

/// A mapping from old `IdVecKey`s to new `IdVecKey`s after a compaction
/// operation.  Can represent either a full mapping with possible removals,
/// or an identity mapping where keys map to themselves.
pub enum IdVecKeyMap {
    Mapping {
        data: Vec<Option<IdVecKey>>,
        index_offset: usize,
    },
    Identity {
        range: Range<usize>,
    },
}

impl IdVecKeyMap {
    pub fn get(&self, old_key: IdVecKey) -> Option<IdVecKey> {
        match self {
            IdVecKeyMap::Mapping { data, index_offset } => {
                data.get(old_key.0 - *index_offset).copied().flatten()
            }
            IdVecKeyMap::Identity { .. } => Some(old_key),
        }
    }

    fn insert(&mut self, old_key: IdVecKey, new_key: Option<IdVecKey>) {
        match self {
            IdVecKeyMap::Mapping { data, index_offset } => {
                debug_assert_eq!(old_key.0 - *index_offset, data.len());
                data.push(new_key);
            }
            IdVecKeyMap::Identity { .. } => {
                panic!("Cannot push to Identity key map")
            }
        }
    }
}

impl Into<HashMap<IdVecKey, IdVecKey>> for IdVecKeyMap {
    fn into(self) -> HashMap<IdVecKey, IdVecKey> {
        match self {
            IdVecKeyMap::Mapping { data, index_offset } => data
                .into_iter()
                .enumerate()
                .flat_map(|(old_index, new_key)| {
                    new_key.map(|new_key| (IdVecKey(old_index + index_offset), new_key))
                })
                .collect(),
            IdVecKeyMap::Identity { range } => range.map(|i| (IdVecKey(i), IdVecKey(i))).collect(),
        }
    }
}

/// A map-like structure that assigns stable keys to inserted values.  Keys
/// remain valid across insertions and removals, but not across `compact` or
/// `shrink_to_fit` operations.  Internally uses a `Vec<MaybeUninit<T>>` to
/// store values and a `BitVec` to track live entries.
pub struct IdVec<T> {
    vec: Vec<MaybeUninit<T>>,
    liveness: BitVec,
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
        let index = self.decode_key(key);
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
        let index = self.decode_key(key);
        if index < self.liveness.len() && self.liveness[index] {
            Some(unsafe { &*self.vec[index].as_ptr() })
        } else {
            None
        }
    }

    /// Gets a reference to the item at the given key, if it exists.
    pub fn get_mut(&mut self, key: IdVecKey) -> Option<&mut T> {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let index = self.decode_key(key);
        if index < self.liveness.len() && self.liveness[index] {
            Some(unsafe { &mut *self.vec[index].as_mut_ptr() })
        } else {
            None
        }
    }

    /// Compacts the `IdVec` by removing all dead entries and shifting live
    /// entries down to fill the gaps. This invalidates all existing keys.
    /// No memory is reallocated.
    ///
    /// Returns a mapping from old keys to new keys. Use `IdVecKeyMap::get(old_key)`
    /// to retrieve the new key for a live entry, or `None` if the entry was removed.
    /// Old keys cannot be used directly after compaction.
    pub fn compact(&mut self) -> IdVecKeyMap {
        let new_key_offset = self.vec.len();
        let mut key_map = IdVecKeyMap::Mapping {
            data: Vec::with_capacity(self.vec.len()),
            index_offset: self.key_offset,
        };
        let mut di = 0;
        for si in self.key_offset..self.liveness.len() {
            let old_key = IdVecKey(si + self.key_offset);
            if self.liveness[si] {
                key_map.insert(old_key, Some(IdVecKey(di + new_key_offset)));
                self.vec[di] = MaybeUninit::new(unsafe { self.vec[si].assume_init_read() });
                self.liveness.set(di, true);
                di += 1;
            } else {
                key_map.insert(old_key, None);
            }
        }
        self.vec.truncate(di);
        self.liveness.truncate(di);
        self.key_offset = new_key_offset;
        key_map
    }

    /// Compacts the `IdVec` by removing all dead entries without shifting live entries.
    /// This invalidates all existing keys. Memory is reallocated to fit exactly.
    ///
    /// Returns a mapping from old keys to new keys. Use `IdVecKeyMap::get(old_key)`
    /// to retrieve the new key for a live entry, or `None` if the entry was removed.
    /// If no entries were removed, returns an `Identity` mapping where keys map to themselves.
    /// Old keys cannot be used directly after compaction.
    pub fn shrink_to_fit(&mut self) -> IdVecKeyMap {
        if self.len() == self.vec.len() + self.key_offset {
            return IdVecKeyMap::Identity {
                range: self.key_offset..(self.vec.len() + self.key_offset),
            };
        }

        let mut key_map = IdVecKeyMap::Mapping {
            data: Vec::with_capacity(self.vec.len()),
            index_offset: self.key_offset,
        };

        let new_key_offset = self.vec.len();
        let mut new_vec: Vec<MaybeUninit<T>> = Vec::with_capacity(self.vec.len());
        let mut new_liveness = BitVec::with_capacity(self.liveness.len());

        for (si, live) in self.liveness.iter().enumerate() {
            let old_key = IdVecKey(si + self.key_offset);
            if *live {
                let di = new_vec.len();
                key_map.insert(old_key, Some(IdVecKey(di + new_key_offset)));
                new_vec.push(unsafe { MaybeUninit::new(self.vec[si].assume_init_read()) });
                new_liveness.push(true);
            } else {
                key_map.insert(old_key, None);
            }
        }

        self.vec = new_vec;
        self.liveness = new_liveness;
        self.key_offset = new_key_offset;

        key_map
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

    /// Decodes an `IdVecKey` into its internal index.
    fn decode_key(&self, index: IdVecKey) -> usize {
        index.0 - self.key_offset
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
        let key_map = vec.compact();

        assert_eq!(vec.len(), 2);
        assert_eq!(vec.iter().sum::<i32>(), 4);

        // Test the key mapping
        let new_id1 = key_map.get(id1).expect("id1 should have a mapping");
        let new_id3 = key_map.get(id3).expect("id3 should have a mapping");
        assert!(
            key_map.get(id2).is_none(),
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
        let key_map = vec.shrink_to_fit();

        assert_eq!(vec.len(), 1);

        // Test the key mapping
        assert!(
            key_map.get(id1).is_none(),
            "id1 was removed, should map to None"
        );
        let new_id2 = key_map.get(id2).expect("id2 should have a mapping");
        assert!(
            key_map.get(id3).is_none(),
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
        vec.compact();

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
    fn test_idveckeymap_get_mapping() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);

        vec.remove(id2);
        let key_map = vec.compact();

        // Old id1 should map to a new key
        assert!(key_map.get(id1).is_some());
        // Old id2 (removed) should map to None
        assert!(key_map.get(id2).is_none());
        // Old id3 should map to a new key
        assert!(key_map.get(id3).is_some());
    }

    #[test]
    fn test_idveckeymap_get_identity() {
        let mut vec = IdVec::new();
        vec.insert(1);
        vec.insert(2);
        vec.insert(3);

        // shrink_to_fit with no removals returns Identity map
        let key_map = vec.shrink_to_fit();

        let key = IdVecKey(1);
        assert_eq!(key_map.get(key), Some(key));
    }

    #[test]
    fn test_idveckeymap_into_hashmap_mapping() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);
        let id3 = vec.insert(3);

        vec.remove(id2);
        let key_map = vec.compact();

        let hash_map: HashMap<IdVecKey, IdVecKey> = key_map.into();

        // Should only have entries for keys that still exist (not removed)
        assert!(hash_map.contains_key(&id1));
        assert!(!hash_map.contains_key(&id2)); // id2 was removed, so it won't be in the HashMap
        assert!(hash_map.contains_key(&id3));

        // Verify the mappings exist
        assert!(hash_map.get(&id1).is_some());
        assert!(hash_map.get(&id3).is_some());
    }

    #[test]
    fn test_idveckeymap_into_hashmap_identity() {
        let mut vec = IdVec::new();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);

        let key_map = vec.shrink_to_fit();
        let hash_map: HashMap<IdVecKey, IdVecKey> = key_map.into();

        // Identity map should have entries for each key mapping to itself
        assert_eq!(hash_map.get(&id1), Some(&id1));
        assert_eq!(hash_map.get(&id2), Some(&id2));
    }
}
