//! Trait for stable-key vector implementations.

use std::{fmt::Debug, hash::Hash};

/// A trait for keys of `IdVec` implementations.
pub trait IdVecKeyTrait: Clone + Eq + Hash + Ord + Debug {}

/// A trait for exposing the ability to convert between `IdVecKey` and
/// zero-based indices for use in adjacency matrices and similar structures.
pub trait IdVecIndexing {
    type Key: IdVecKeyTrait;

    /// Decodes an `IdVecKey` into an index smaller than the size of the
    /// `IdVec`. Used for correlating items in the `IdVec` with other data
    /// structures that use zero-based indexing, such as adjacency matrices.
    ///
    /// This is used internally to map from the stable key to the internal
    /// vector index.
    fn zero_based_index(&self, index: Self::Key) -> usize;

    /// Encodes an index returned by `zero_based_index` back into an `IdVecKey`.
    /// Use with caution, because there is no guarantee that the index is valid
    /// unless it came directly from `zero_based_index`, and even then, it may
    /// be a the key of a removed entry.
    fn key_from_index(&self, index: usize) -> Self::Key;
}

/// A trait for map-like containers that assign stable keys to inserted values.
///
/// Keys remain valid across insertions and removals, though behavior may vary
/// across different implementations. Consult specific implementations for details
/// about key stability across operations like compaction or shrinking.
///
/// # Implementations
///
/// - `OffsetIdVec`: Uses a bitvec to track liveness with key offset management.
/// - `IndexedIdVec`: Uses dense index mapping for O(1) swap-remove operations.
pub trait IdVec<T> {
    type Key: Copy + Eq + Hash + Ord + Debug;
    type Indexing: IdVecIndexing<Key = Self::Key>;

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

    /// Compacts the `IdVec` by removing all dead entries and shifting live
    /// entries down to fill the gaps. This invalidates all existing keys.  No
    /// memory is reallocated.
    fn compact(&mut self);

    /// Compacts the `IdVec` by removing all dead entries and shifting live
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

    /// Compacts the `IdVec` by removing all dead entries without shifting live
    /// entries.  This may invalidate all existing keys. Memory is reallocated to
    /// fit exactly.
    ///
    /// Calls the provided callback for each key, passing in the old ID as the
    /// first parameter.  If the old ID was still valid, the new ID is passed as
    /// the second parameter; otherwise, None is passed.  If no entries were
    /// removed, all keys map to themselves, so the callback is not called.
    fn shrink_to_fit_with(&mut self, callback: impl FnMut(Self::Key, Option<Self::Key>));
}
