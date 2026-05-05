use std::collections::HashMap;

/// A trait for building a mapping from old keys to new keys during compaction
/// operations.  Default implementations are provided to store the pairs in a
/// `Vec` or `HashMap`, or to call a provided function for each pair.
///
/// This crate has various types with a `compact` method that can be used to
/// reduce memory usage by reusing logical IDs.  The `compact` method takes an
/// optional `MapCollector` that can be used to track the mapping from old keys
/// to new keys during the compaction process.  This is useful for updating
/// external data structures that reference the keys, or for debugging purposes.
pub trait MapCollector<T> {
    /// Records a mapping from an old key to a new key.  The `compact` method
    /// will call this for each key that is being compacted, allowing the caller
    /// to track the changes.
    fn insert(&mut self, old_key: T, new_key: T);
}

impl<T> MapCollector<T> for &mut Vec<(T, T)> {
    fn insert(&mut self, old_key: T, new_key: T) {
        (*self).push((old_key, new_key));
    }
}

impl<T> MapCollector<T> for &mut HashMap<T, T>
where
    T: std::hash::Hash + Eq,
{
    fn insert(&mut self, old_key: T, new_key: T) {
        (*self).insert(old_key, new_key);
    }
}

impl<T, F> MapCollector<T> for F
where
    F: FnMut(T, T),
{
    fn insert(&mut self, old_key: T, new_key: T) {
        self(old_key, new_key);
    }
}
