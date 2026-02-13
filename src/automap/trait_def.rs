//! Trait for stable-key vector implementations.

use std::{fmt::Debug, hash::Hash};

/// A trait for keys of `Automap` implementations.
pub trait AutomapKeyTrait: Clone + Eq + Hash + Ord + Debug {}

/// A trait for exposing the ability to convert between `AutomapKey` and
/// zero-based indices for use in adjacency matrices and similar structures.
pub trait AutomapIndexing {
    type Key: AutomapKeyTrait;

    /// Decodes an automap key into an index smaller than the size of the map.
    /// Used for correlating items in the map with other data structures that
    /// use zero-based indexing, such as adjacency matrices.
    ///
    /// This is used internally to map from the stable key to the internal
    /// vector index.
    fn key_to_index(&self, index: Self::Key) -> usize;

    /// Encodes an index returned by `key_to_index` back into an automap key.
    /// Use with caution, because there is no guarantee that the index is valid
    /// unless it came directly from `key_to_index`, and even then, it may be a
    /// the key of a removed entry.
    fn index_to_key(&self, index: usize) -> Self::Key;
}

/// A trait for map-like containers that assign stable keys to inserted values.
///
/// Keys remain valid across insertions and removals, though behavior may vary
/// across different implementations. Consult specific implementations for details
/// about key stability across operations like compaction or shrinking.
///
/// # Implementations
///
/// - `OffsetAutomap`: Uses a bitvec to track liveness with key offset management.
/// - `IndexedAutomap`: Uses dense index mapping for O(1) swap-remove operations.
pub trait Automap<T> {
    type Key: Copy + Eq + Hash + Ord + Debug;
    type Indexing: AutomapIndexing<Key = Self::Key>;

    /// Inserts a value and returns a stable key for accessing it.
    fn insert(&mut self, value: T) -> Self::Key;

    /// Returns a reference to the value associated with the given key, or `None` if removed.
    fn get(&self, key: Self::Key) -> Option<&T>;

    /// Returns a mutable reference to the value associated with the given key, or `None` if removed.
    fn get_mut(&mut self, key: Self::Key) -> Option<&mut T>;

    /// Removes the value associated with the given key and returns it, or `None` if already removed.
    fn remove(&mut self, key: Self::Key) -> Option<T>;

    /// Returns the number of live entries in the container.
    fn len(&self) -> usize;

    /// Checks if the container is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all entries from the container.
    fn clear(&mut self);

    /// Returns the capacity of the container.
    fn capacity(&self) -> usize;

    /// Reserves capacity for at least `additional` more elements.
    fn reserve(&mut self, additional: usize);

    /// Reserves the exact capacity for `additional` more elements.
    fn reserve_exact(&mut self, additional: usize);

    /// Returns an iterator over keys of live entries.
    fn iter_keys(&self) -> impl Iterator<Item = Self::Key>;

    /// Returns an iterator over references to live values.
    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a;

    /// Returns an iterator over mutable references to live values.
    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a;

    /// Returns an iterator over (key, value) pairs.
    fn iter_pairs<'a>(&'a self) -> impl Iterator<Item = (Self::Key, &'a T)>
    where
        T: 'a;

    /// Returns an iterator over (key, mutable value) pairs.
    fn iter_pairs_mut<'a>(&'a mut self) -> impl Iterator<Item = (Self::Key, &'a mut T)>
    where
        T: 'a;

    /// Returns an indexing helper for zero-based index conversions.
    fn indexing(&self) -> Self::Indexing;

    /// Compacts the `Automap` by removing all dead entries and shifting live
    /// entries down to fill the gaps. This invalidates all existing keys.  No
    /// memory is reallocated.
    fn compact(&mut self);

    /// Compacts the `Automap` by removing all dead entries and shifting live
    /// entries down to fill the gaps. This invalidates all existing keys.  No
    /// memory is reallocated.
    ///
    /// Calls the provided callback for each key, passing in the old ID as the
    /// first parameter.  If the old ID was still valid, the new ID is passed as
    /// the second parameter; otherwise, None is passed.  If no entries were
    /// removed, all keys map to themselves, so the callback is not called.
    fn compact_with(&mut self, callback: impl FnMut(Self::Key, Option<Self::Key>));

    /// Shrinks the internal storage to fit the current size.  This may
    /// invalidate existing keys, depending on the implementation.  Consult
    /// specific implementations for details.
    fn shrink_to_fit(&mut self);

    /// Compacts the `Automap` by removing all dead entries without shifting live
    /// entries.  This may invalidate all existing keys. Memory is reallocated to
    /// fit exactly.
    ///
    /// Calls the provided callback for each key, passing in the old ID as the
    /// first parameter.  If the old ID was still valid, the new ID is passed as
    /// the second parameter; otherwise, None is passed.  If no entries were
    /// removed, all keys map to themselves, so the callback is not called.
    fn shrink_to_fit_with(&mut self, callback: impl FnMut(Self::Key, Option<Self::Key>));
}

/// Macro to generate common unit tests for `Automap` implementations.
///
/// # Usage
/// ```
/// #[cfg(test)]
/// mod tests {
///     use super::*;
///     use super::super::trait_def::Automap;
///     use crate::automap_tests;
///
///     automap_tests!(MyAutomapImpl, i32, |i: i32| i);
/// }
/// ```
///
/// The third parameter is a type constructor/default value generator for test values.
#[macro_export]
macro_rules! automap_tests {
    ($impl_type:ty, $value_type:ty, $default_fn:expr) => {
        #[test]
        fn test_new() {
            let vec: $impl_type = Default::default();
            assert!(vec.is_empty());
            assert_eq!(vec.len(), 0);
        }

        #[test]
        fn test_insert_and_get() {
            let mut vec: $impl_type = Default::default();
            let k1 = vec.insert($default_fn(1));
            let k2 = vec.insert($default_fn(2));

            assert_eq!(vec.len(), 2);
            assert_eq!(vec.get(k1), Some(&$default_fn(1)));
            assert_eq!(vec.get(k2), Some(&$default_fn(2)));
        }

        #[test]
        fn test_remove() {
            let mut vec: $impl_type = Default::default();
            let k1 = vec.insert($default_fn(42));
            let k2 = vec.insert($default_fn(100));

            assert_eq!(vec.remove(k1), Some($default_fn(42)));
            assert_eq!(vec.len(), 1);
            assert_eq!(vec.get(k2), Some(&$default_fn(100)));
        }

        #[test]
        fn test_insert_and_remove() {
            let mut vec: $impl_type = Default::default();
            let k1 = vec.insert($default_fn(42));
            let k2 = vec.insert($default_fn(43));
            let k3 = vec.insert($default_fn(44));

            assert_eq!(vec.remove(k2), Some($default_fn(43)));
            assert_eq!(vec.len(), 2);
            assert_eq!(vec.get(k1), Some(&$default_fn(42)));
            assert_eq!(vec.get(k3), Some(&$default_fn(44)));
        }

        #[test]
        fn test_remove_dead_returns_none() {
            let mut vec: $impl_type = Default::default();
            let k = vec.insert($default_fn(42));
            vec.remove(k);
            assert!(vec.remove(k).is_none());
        }

        #[test]
        fn test_clear() {
            let mut vec: $impl_type = Default::default();
            let k1 = vec.insert($default_fn(1));
            let k2 = vec.insert($default_fn(2));
            let k3 = vec.insert($default_fn(3));

            vec.remove(k2);
            assert_eq!(vec.len(), 2);
            assert!(!vec.is_empty());

            vec.clear();

            assert_eq!(vec.len(), 0);
            assert!(vec.is_empty());
            assert!(vec.get(k1).is_none());
            assert!(vec.get(k3).is_none());

            let k4 = vec.insert($default_fn(4));
            assert_eq!(vec.len(), 1);
            assert_eq!(vec.get(k4), Some(&$default_fn(4)));
        }

        #[test]
        fn test_get_mut() {
            let mut vec: $impl_type = Default::default();
            let k = vec.insert($default_fn(42));

            if let Some(val) = vec.get_mut(k) {
                *val = $default_fn(100);
            }
            assert_eq!(vec.get(k), Some(&$default_fn(100)));
        }

        #[test]
        fn test_len() {
            let mut vec: $impl_type = Default::default();
            assert_eq!(vec.len(), 0);

            vec.insert($default_fn(1));
            assert_eq!(vec.len(), 1);

            let k2 = vec.insert($default_fn(2));
            assert_eq!(vec.len(), 2);

            vec.remove(k2);
            assert_eq!(vec.len(), 1);
        }

        #[test]
        fn test_is_empty() {
            let mut vec: $impl_type = Default::default();
            assert!(vec.is_empty());

            let k = vec.insert($default_fn(1));
            assert!(!vec.is_empty());

            vec.remove(k);
            assert!(vec.is_empty());
        }

        #[test]
        fn test_capacity() {
            let mut vec: $impl_type = Default::default();
            let initial_capacity = vec.capacity();

            vec.reserve(100);
            assert!(vec.capacity() > initial_capacity);
            assert!(vec.capacity() >= 100);
        }

        #[test]
        fn test_iter_keys() {
            let mut vec: $impl_type = Default::default();
            let k1 = vec.insert($default_fn(10));
            let k2 = vec.insert($default_fn(20));
            let k3 = vec.insert($default_fn(30));

            vec.remove(k2);

            let indices: Vec<_> = vec.iter_keys().collect();
            assert_eq!(indices.len(), 2);
        }

        #[test]
        fn test_iter() {
            let mut vec: $impl_type = Default::default();
            vec.insert($default_fn(1));
            vec.insert($default_fn(2));
            vec.insert($default_fn(3));

            let count = vec.iter().count();
            assert_eq!(count, 3);
        }

        #[test]
        fn test_get_returns_none_on_dead_id() {
            let mut vec: $impl_type = Default::default();
            let k = vec.insert($default_fn(42));
            vec.remove(k);
            assert!(vec.get(k).is_none());
        }

        #[test]
        fn test_get_mut_returns_none_on_dead_id() {
            let mut vec: $impl_type = Default::default();
            let k = vec.insert($default_fn(42));
            vec.remove(k);
            assert!(vec.get_mut(k).is_none());
        }

        #[test]
        fn test_multiple_inserts_after_removals() {
            let mut vec: $impl_type = Default::default();
            let k1 = vec.insert($default_fn(1));
            let k2 = vec.insert($default_fn(2));
            let k3 = vec.insert($default_fn(3));

            vec.remove(k1);
            vec.remove(k3);

            let k4 = vec.insert($default_fn(4));
            let k5 = vec.insert($default_fn(5));

            assert_eq!(vec.len(), 3);
            assert_eq!(vec.get(k2), Some(&$default_fn(2)));
            assert_eq!(vec.get(k4), Some(&$default_fn(4)));
            assert_eq!(vec.get(k5), Some(&$default_fn(5)));
        }

        #[test]
        fn test_reserve() {
            let mut vec: $impl_type = Default::default();
            let old_capacity = vec.capacity();
            vec.reserve(100);
            assert!(vec.capacity() >= old_capacity + 100);
        }

        #[test]
        fn test_reserve_exact() {
            let mut vec: $impl_type = Default::default();
            vec.reserve_exact(50);
            assert!(vec.capacity() >= 50);

            for i in 0..10 {
                vec.insert($default_fn(i));
            }
            assert_eq!(vec.len(), 10);
        }

        #[test]
        fn test_iter_pairs() {
            let mut vec: $impl_type = Default::default();
            let k1 = vec.insert($default_fn(10));
            let k2 = vec.insert($default_fn(20));
            let k3 = vec.insert($default_fn(30));

            vec.remove(k2);

            let pairs: Vec<_> = vec.iter_pairs().collect();
            assert_eq!(pairs.len(), 2);
        }

        #[test]
        fn test_iter_pairs_mut() {
            let mut vec: $impl_type = Default::default();
            let k1 = vec.insert($default_fn(10));
            let k2 = vec.insert($default_fn(20));
            let k3 = vec.insert($default_fn(30));

            vec.remove(k2);

            for (_k, _val) in vec.iter_pairs_mut() {
                // Just verify iteration works
            }

            let pairs: Vec<_> = vec.iter_pairs().collect();
            assert_eq!(pairs.len(), 2);
        }

        #[test]
        fn test_compact_with_callback_after_removals() {
            let mut vec: $impl_type = Default::default();
            let id1 = vec.insert($default_fn(1));
            let id2 = vec.insert($default_fn(2));
            let id3 = vec.insert($default_fn(3));

            vec.remove(id2);

            let mut key_map = std::collections::HashMap::new();
            vec.compact_with(|old_key, new_key| {
                if let Some(new_key) = new_key {
                    key_map.insert(old_key, new_key);
                }
            });

            // Use the remapped keys if they exist, otherwise use original keys
            let final_id1 = key_map.get(&id1).copied().unwrap_or(id1);
            let final_id3 = key_map.get(&id3).copied().unwrap_or(id3);

            assert_eq!(vec.get(final_id1), Some(&$default_fn(1)));
            assert_eq!(vec.get(final_id3), Some(&$default_fn(3)));
        }

        #[test]
        fn test_compact_with_callback_no_removals() {
            let mut vec: $impl_type = Default::default();
            let id1 = vec.insert($default_fn(1));
            vec.insert($default_fn(2));
            vec.insert($default_fn(3));

            // compact with no removals does nothing, callback not called
            let mut callback_called = false;

            vec.compact_with(|_, _| {
                callback_called = true;
            });

            // Callback should not be called for identity case
            assert!(!callback_called);
            // Old keys should still work
            assert_eq!(vec.get(id1), Some(&$default_fn(1)));
        }

        #[test]
        fn test_shrink_to_fit_with_callback_no_removals() {
            let mut vec: $impl_type = Default::default();
            let id1 = vec.insert($default_fn(1));
            vec.insert($default_fn(2));
            vec.insert($default_fn(3));

            // shrink_to_fit with no removals does nothing, callback not called
            let mut callback_called = false;

            vec.shrink_to_fit_with(|_, _| {
                callback_called = true;
            });

            // Callback should not be called for identity case
            assert!(!callback_called);
            // Old keys should still work
            assert_eq!(vec.get(id1), Some(&$default_fn(1)));
        }

        #[test]
        fn test_shrink_to_fit_with_callback_identity_case() {
            let mut vec: $impl_type = Default::default();
            let id1 = vec.insert($default_fn(1));
            let id2 = vec.insert($default_fn(2));

            // shrink_to_fit with no removals does nothing, callback not called
            let mut callback_called = false;

            vec.shrink_to_fit_with(|_, _| {
                callback_called = true;
            });

            // Callback should not be called for identity case
            assert!(!callback_called);
            // Old keys should still work
            assert_eq!(vec.get(id1), Some(&$default_fn(1)));
            assert_eq!(vec.get(id2), Some(&$default_fn(2)));
        }

        #[test]
        fn test_multiple_compactions() {
            let mut vec: $impl_type = Default::default();
            let id1 = vec.insert($default_fn(1));
            let id2 = vec.insert($default_fn(2));
            let id3 = vec.insert($default_fn(3));
            let id4 = vec.insert($default_fn(4));
            let id5 = vec.insert($default_fn(5));

            // First round of removals
            vec.remove(id2);
            vec.remove(id4);

            let mut key_map = std::collections::HashMap::new();
            vec.compact_with(|old_key, new_key| {
                if let Some(new_key) = new_key {
                    key_map.insert(old_key, new_key);
                }
            });

            // Update keys after first compaction (or keep original if not remapped)
            let new_id1 = key_map.get(&id1).copied().unwrap_or(id1);
            let new_id3 = key_map.get(&id3).copied().unwrap_or(id3);
            let new_id5 = key_map.get(&id5).copied().unwrap_or(id5);

            assert_eq!(vec.len(), 3);
            assert_eq!(vec.get(new_id1), Some(&$default_fn(1)));
            assert_eq!(vec.get(new_id3), Some(&$default_fn(3)));
            assert_eq!(vec.get(new_id5), Some(&$default_fn(5)));

            // Insert more values
            let id6 = vec.insert($default_fn(6));
            let id7 = vec.insert($default_fn(7));

            // Second round of removals
            vec.remove(new_id3);
            vec.remove(id6);

            key_map.clear();
            vec.compact_with(|old_key, new_key| {
                if let Some(new_key) = new_key {
                    key_map.insert(old_key, new_key);
                }
            });

            // Update keys after second compaction (or keep current if not remapped)
            let final_id1 = key_map.get(&new_id1).copied().unwrap_or(new_id1);
            let final_id5 = key_map.get(&new_id5).copied().unwrap_or(new_id5);
            let final_id7 = key_map.get(&id7).copied().unwrap_or(id7);

            assert_eq!(vec.len(), 3);
            assert_eq!(vec.get(final_id1), Some(&$default_fn(1)));
            assert_eq!(vec.get(final_id5), Some(&$default_fn(5)));
            assert_eq!(vec.get(final_id7), Some(&$default_fn(7)));
        }

        #[test]
        fn test_multiple_shrink_to_fits() {
            let mut vec: $impl_type = Default::default();
            let id1 = vec.insert($default_fn(1));
            let id2 = vec.insert($default_fn(2));
            let id3 = vec.insert($default_fn(3));
            let id4 = vec.insert($default_fn(4));
            let id5 = vec.insert($default_fn(5));

            // First round of removals
            vec.remove(id2);
            vec.remove(id4);

            let mut key_map = std::collections::HashMap::new();
            vec.shrink_to_fit_with(|old_key, new_key| {
                if let Some(new_key) = new_key {
                    key_map.insert(old_key, new_key);
                }
            });

            // Update keys after first shrink_to_fit (or keep original if not remapped)
            let new_id1 = key_map.get(&id1).copied().unwrap_or(id1);
            let new_id3 = key_map.get(&id3).copied().unwrap_or(id3);
            let new_id5 = key_map.get(&id5).copied().unwrap_or(id5);

            assert_eq!(vec.len(), 3);
            assert_eq!(vec.get(new_id1), Some(&$default_fn(1)));
            assert_eq!(vec.get(new_id3), Some(&$default_fn(3)));
            assert_eq!(vec.get(new_id5), Some(&$default_fn(5)));

            // Insert more values
            let id6 = vec.insert($default_fn(6));
            let id7 = vec.insert($default_fn(7));

            // Second round of removals
            vec.remove(new_id3);
            vec.remove(id6);

            key_map.clear();
            vec.shrink_to_fit_with(|old_key, new_key| {
                if let Some(new_key) = new_key {
                    key_map.insert(old_key, new_key);
                }
            });

            // Update keys after second shrink_to_fit (or keep current if not remapped)
            let final_id1 = key_map.get(&new_id1).copied().unwrap_or(new_id1);
            let final_id5 = key_map.get(&new_id5).copied().unwrap_or(new_id5);
            let final_id7 = key_map.get(&id7).copied().unwrap_or(id7);

            assert_eq!(vec.len(), 3);
            assert_eq!(vec.get(final_id1), Some(&$default_fn(1)));
            assert_eq!(vec.get(final_id5), Some(&$default_fn(5)));
            assert_eq!(vec.get(final_id7), Some(&$default_fn(7)));
        }

        #[test]
        fn test_automap_large() {
            // Insert a large number of entries, remove many of them in
            // pseudo-random order, and occasionally compact while tracking
            // remappings. Inspired by the large-graph deconstruction test.
            let mut vec: $impl_type = Default::default();
            let mut map = std::collections::HashMap::new();
            let mut keys = Vec::new();
            let total: usize = 1000;

            for i in 0..total {
                let k = vec.insert($default_fn(i as $value_type));
                map.insert(k, $default_fn(i as $value_type));
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
                    vec.compact_with(|old_key, new_key| {
                        match new_key {
                            Some(new_key) => {
                                let was_present = key_set.remove(&old_key);
                                // old may already have been removed earlier
                                assert!(was_present || removed.contains(&old_key));
                                // Update key_set and also move any stored mapping in `map`.
                                let inserted = key_set.insert(new_key);
                                assert!(inserted);
                                if let Some(v) = map.remove(&old_key) {
                                    map.insert(new_key, v);
                                }
                            }
                            None => {
                                // Deleted entries should previously have been removed
                                assert!(removed.contains(&old_key));
                            }
                        }
                    });

                    assert_eq!(vec.len(), len_before);
                    for k in key_set.iter() {
                        assert!(vec.get(*k).is_some());
                    }
                }
            }

            assert_eq!(vec.len(), 0);
        }
    };
}
