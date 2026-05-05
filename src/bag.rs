use std::{
    fmt::Debug,
    num::NonZero,
    ops::{Index, IndexMut},
};

use crate::map_collector::MapCollector;

/// A stable key type for entries in the `Bag`.  Internally, it is just an
/// integer index.
///
/// The implementation uses `NonZero` to ensure that an `Option<BagKey>` can be
/// represented as a single word.
#[derive(Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct BagKey(NonZero<usize>);

impl BagKey {
    pub fn to_index(&self) -> usize {
        self.0.get() - 1
    }

    pub fn from_index(index: usize) -> Self {
        BagKey(NonZero::new(index + 1).expect("index must be non-zero"))
    }
}

impl Debug for BagKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_index())
    }
}

/// A bag implementation that maintains a mapping from stable keys to values
/// using a dense index mapping. This allows for O(1) insertions, lookups, and
/// removals by swapping removed elements with the last element in the data
/// vector, while keeping track of the logical position of each element through
/// a separate index vector.
///
/// Keys are assigned sequentially starting from 0, so the maximum key for any
/// value in the bag should be N-1, where N is the largest number of values that
/// have been stored in the bag at once since the last call to `compact()`.
///
/// The `data` vector stores the values along with their logical IDs, while the
/// `index` vector maps logical IDs to their positions in the `data` vector.
/// When an element is removed, its logical ID is marked as `None` in the
/// `index` vector, and the last element in the `data` vector is moved to fill
/// the gap, with its logical ID updated accordingly. This approach allows for
/// efficient memory usage and fast operations while maintaining stable keys for
/// accessing values.
pub struct Bag<T> {
    /// The data with reverse indices, stored in arbitrary order.
    /// Each element is (value, reverse_index) where reverse_index maps back to the logical position.
    data: Vec<(T, BagKey)>,
    /// Maps from logical indices to positions in the data vector. None indicates the key was removed.
    index: Vec<Option<BagKey>>,
}

impl<T> Default for Bag<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            index: Vec::new(),
        }
    }
}

impl<T> Bag<T> {
    #[cfg(test)]
    fn verify_index(&self) {
        for (logical_id, opt_data_key) in self.index.iter().enumerate() {
            if let Some(data_key) = opt_data_key {
                let data_idx = data_key.to_index();
                assert!(
                    data_idx < self.data.len(),
                    "Data key index {} out of bounds for data length {}",
                    data_idx,
                    self.data.len()
                );
                let (_value, reverse_logical_id) = &self.data[data_idx];
                assert_eq!(
                    *reverse_logical_id,
                    BagKey::from_index(logical_id),
                    "Reverse index mismatch: logical_id {}, data_idx {}, expected logical_id {}",
                    logical_id,
                    data_idx,
                    reverse_logical_id.to_index()
                );
            }
        }
    }

    /// Inserts a value into the bag and returns a stable key that can be used
    /// to access it.  The key remains valid until the value is removed, even if
    /// other values are inserted or removed.  The key can be used to get a
    /// reference to the value, mutate it, or remove it from the bag.
    pub fn insert(&mut self, value: T) -> BagKey {
        let id = BagKey::from_index(self.index.len());
        let data_key = BagKey::from_index(self.data.len());
        self.data.push((value, id));
        self.index.push(Some(data_key));
        id
    }

    /// Returns a reference to the value associated with the given key, or
    /// `None` if the key is invalid or has been removed.
    pub fn get(&self, key: BagKey) -> Option<&T> {
        let data_key = (*self.index.get(key.to_index())?)?;
        Some(&self.data[data_key.to_index()].0)
    }

    /// Returns a mutable reference to the value associated with the given key, or
    /// `None` if the key is invalid or has been removed.
    pub fn get_mut(&mut self, key: BagKey) -> Option<&mut T> {
        if key.to_index() >= self.index.len() {
            return None;
        }
        match self.index[key.to_index()] {
            Some(data_key) => Some(&mut self.data[data_key.to_index()].0),
            None => None,
        }
    }

    /// Removes the value associated with the given key and returns it, or
    /// `None` if the key is invalid or has already been removed.
    ///
    /// This operation maintains the stability of other keys by swapping the
    /// removed element with the last element in the data vector and updating
    /// the index mapping accordingly.
    pub fn remove(&mut self, key: BagKey) -> Option<T> {
        if key.to_index() >= self.index.len() {
            return None;
        }

        // Mark as removed first
        let data_idx = self.index[key.to_index()].take()?.to_index();
        let (removed_value, _) = self.data.swap_remove(data_idx);

        // If an element was moved from the end, update its index mapping
        if data_idx < self.data.len() {
            let moved_logical_id = self.data[data_idx].1;
            self.index[moved_logical_id.to_index()] = Some(BagKey::from_index(data_idx));
        }

        Some(removed_value)
    }

    /// Returns the number of values currently in the bag.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the bag is empty, i.e., contains no values.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Removes all values from the bag, leaving it empty.  This does not change
    /// the capacity of the underlying storage, so it can be used to quickly
    /// clear the bag without deallocating memory.
    pub fn clear(&mut self) {
        self.data.clear();
        self.index.clear();
    }

    /// Returns the total capacity of the bag, which is the maximum number of
    /// values it can hold without reallocating.  This is typically greater than
    /// or equal to the current length of the bag, depending on how many
    /// insertions and removals have occurred.
    pub fn capacity(&self) -> usize {
        let extra_data_capacity = self.data.capacity() - self.data.len();
        let extra_index_capacity = self.index.capacity() - self.index.len();
        self.len() + extra_data_capacity.min(extra_index_capacity)
    }

    /// Reserves capacity for at least `additional` more values to be inserted
    /// into the bag without additional allocations.
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
        self.index.reserve(additional);
    }

    /// Reserves the minimum capacity for at at least `additional` more values
    /// to be inserted into the bag without additional allocations, without
    /// deliberately overallocating.
    pub fn reserve_exact(&mut self, additional: usize) {
        let extra_data_capacity = self.data.capacity() - self.data.len();
        let extra_index_capacity = self.index.capacity() - self.index.len();
        if extra_data_capacity >= additional && extra_index_capacity >= additional {
            return;
        }
        self.data.reserve_exact(additional - extra_data_capacity);
        self.index.reserve_exact(additional - extra_index_capacity);
    }

    /// Shrinks the capacity of the bag to fit its current length, which can
    /// help reduce memory usage if the bag has had many insertions and
    /// removals.  This does not change the logical keys of existing values, so
    /// they remain valid after shrinking.
    ///
    /// Consider calling `compact` instead if changing the keys of items in the
    /// bag is acceptable, as that can also reduce fragmentation in the index
    /// and improve cache locality.
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.index.shrink_to_fit();
    }

    /// Compacts the bag by reassigning keys to be contiguous from 0 to
    /// `len() - 1`, and optionally collecting the mapping from old keys to
    /// new keys using the provided `MapCollector`.  This will reduce the
    /// memory usage of the bag as much as possible.
    pub fn compact(&mut self, mut collector: Option<impl MapCollector<BagKey>>) {
        let mut new_index = Vec::with_capacity(self.data.len());

        for (logical_id, (data, old_key)) in self.data.iter_mut().enumerate() {
            let new_key = BagKey::from_index(logical_id);
            new_index.push(Some(new_key));
            if let Some(collector) = &mut collector {
                collector.insert(*old_key, new_key);
            }
            *old_key = new_key;
        }

        self.index = new_index;
    }

    /// Returns an iterator over the keys of the values currently in the bag.
    /// The keys are stable and can be used to access the values, but they are
    /// not guaranteed to be in any particular order.
    ///
    /// In particular, calling `bag.keys().zip(bag.iter())` is unlikey to
    /// produce pairs of matching keys and values, because the order of the
    /// iterators is not guaranteed to be the same.  If you need to iterate over
    /// key-value pairs, use `bag.pairs()` or `bag.pairs_mut()`
    /// instead.
    pub fn keys(&self) -> impl Iterator<Item = BagKey> {
        let keys_from_index = self
            .index
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|_| BagKey::from_index(i)));
        let keys_from_data = self.data.iter().map(|(_value, logical_id)| *logical_id);
        let use_keys_from_index = self.index.len() <= 2 * self.data.len();
        keys_from_index
            .take_while(move |_| use_keys_from_index)
            .chain(keys_from_data.take_while(move |_| !use_keys_from_index))
    }

    /// Returns an iterator over references to the values currently in the bag.
    /// The order of values is unspecified and may change as values are inserted
    /// and removed.
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.data.iter().map(|(value, _)| value)
    }

    /// Returns an iterator over mutable references to the values currently in
    /// the bag.  The order of values is unspecified and may change as values
    /// are inserted and removed.
    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        self.data.iter_mut().map(|(value, _)| value)
    }

    /// Gets an iterator over key-value pairs in the bag.  The keys are stable
    /// and can be used to access the values, but they are not guaranteed to be
    /// in any particular order.  The order of pairs may change as values are
    /// inserted and removed.
    pub fn pairs<'a>(&'a self) -> impl Iterator<Item = (BagKey, &'a T)>
    where
        T: 'a,
    {
        self.data
            .iter()
            .map(|(value, logical_id)| (*logical_id, value))
    }

    /// Gets an iterator over key-value pairs in the bag, where the values are
    /// mutable references.  The keys are stable and can be used to access the
    /// values, but they are not guaranteed to be in any particular order.  The
    /// order of pairs may change as values are inserted and removed.
    pub fn pairs_mut<'a>(&'a mut self) -> impl Iterator<Item = (BagKey, &'a mut T)>
    where
        T: 'a,
    {
        self.data
            .iter_mut()
            .map(|(value, logical_id)| (*logical_id, value))
    }
}

impl<T> Clone for Bag<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            index: self.index.clone(),
        }
    }
}

impl<T> Debug for Bag<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_map();
        for (key, value) in self.pairs() {
            debug.entry(&key, value);
        }
        debug.finish()
    }
}

impl<T> Index<BagKey> for Bag<T> {
    type Output = T;

    fn index(&self, key: BagKey) -> &Self::Output {
        self.get(key).expect("invalid BagKey")
    }
}

impl<T> IndexMut<BagKey> for Bag<T> {
    fn index_mut(&mut self, key: BagKey) -> &mut Self::Output {
        self.get_mut(key).expect("invalid BagKey")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use quickcheck::Arbitrary;
    use quickcheck_macros::quickcheck;

    use super::*;

    #[derive(Clone, Debug)]
    enum BagOp {
        Insert,
        Remove(usize),
        Mutate(usize),
        MutateAll(),
        Compact,
    }

    impl Arbitrary for BagOp {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u32::arbitrary(g) % 6 {
                0..2 => BagOp::Insert,
                2 => BagOp::Remove(usize::arbitrary(g)),
                3 => BagOp::Mutate(usize::arbitrary(g)),
                4 => BagOp::MutateAll(),
                5 => BagOp::Compact,
                choice => unreachable!("Invalid value for BagOp: {}", choice),
            }
        }
    }

    #[test]
    fn test_new() {
        let mut bag: Bag<i32> = Bag::default();
        assert!(bag.is_empty());
        assert_eq!(bag.len(), 0);
        assert_eq!(bag.keys().count(), 0);
        assert_eq!(bag.iter().count(), 0);
        assert_eq!(bag.pairs().count(), 0);
        assert_eq!(bag.iter_mut().count(), 0);
        assert_eq!(bag.pairs_mut().count(), 0);
        bag.verify_index();
    }

    #[test]
    fn test_insert_and_get() {
        let mut bag: Bag<i32> = Bag::default();
        let k1 = bag.insert(1);
        let k2 = bag.insert(2);

        assert_eq!(bag.len(), 2);
        assert_eq!(bag.get(k1), Some(&1));
        assert_eq!(bag.get(k2), Some(&2));
        assert_eq!(bag.keys().collect::<Vec<_>>(), vec![k1, k2]);
        assert_eq!(bag.iter().collect::<Vec<_>>(), vec![&1, &2]);
        assert_eq!(bag.pairs().collect::<Vec<_>>(), vec![(k1, &1), (k2, &2)]);
        bag.verify_index();
    }

    #[test]
    fn test_remove() {
        let mut bag: Bag<i32> = Bag::default();
        let k1 = bag.insert(42);
        let k2 = bag.insert(100);

        assert_eq!(bag.remove(k1), Some(42));
        assert_eq!(bag.len(), 1);
        assert_eq!(bag.get(k2), Some(&100));
        assert_eq!(bag.keys().collect::<Vec<_>>(), vec![k2]);
        assert_eq!(bag.iter().collect::<Vec<_>>(), vec![&100]);
        assert_eq!(bag.pairs().collect::<Vec<_>>(), vec![(k2, &100)]);
        bag.verify_index();
    }

    #[test]
    fn test_insert_and_remove() {
        let mut bag: Bag<i32> = Bag::default();
        let k1 = bag.insert(42);
        let k2 = bag.insert(43);
        let k3 = bag.insert(44);

        assert_eq!(bag.remove(k2), Some(43));
        assert_eq!(bag.len(), 2);
        assert_eq!(bag.get(k1), Some(&42));
        assert_eq!(bag.get(k3), Some(&44));
        bag.verify_index();
    }

    #[test]
    fn test_remove_dead_returns_none() {
        let mut bag: Bag<i32> = Bag::default();
        let k = bag.insert(42);
        bag.remove(k);
        assert!(bag.remove(k).is_none());
        bag.verify_index();
    }

    #[test]
    fn test_clear() {
        let mut bag: Bag<i32> = Bag::default();
        let k1 = bag.insert(1);
        let k2 = bag.insert(2);
        let k3 = bag.insert(3);

        bag.remove(k2);
        assert_eq!(bag.len(), 2);
        assert!(!bag.is_empty());
        bag.verify_index();

        bag.clear();

        assert_eq!(bag.len(), 0);
        assert!(bag.is_empty());
        assert!(bag.get(k1).is_none());
        assert!(bag.get(k3).is_none());
        bag.verify_index();

        let k4 = bag.insert(4);
        assert_eq!(bag.len(), 1);
        assert_eq!(bag.get(k4), Some(&4));
        bag.verify_index();
    }

    #[test]
    fn test_get_mut() {
        let mut bag: Bag<i32> = Bag::default();
        let k = bag.insert(42);

        if let Some(val) = bag.get_mut(k) {
            *val = 100;
        }
        assert_eq!(bag.get(k), Some(&100));
        bag.verify_index();
    }

    #[test]
    fn test_len() {
        let mut bag: Bag<i32> = Bag::default();
        assert_eq!(bag.len(), 0);

        bag.insert(1);
        assert_eq!(bag.len(), 1);

        let k2 = bag.insert(2);
        assert_eq!(bag.len(), 2);

        bag.remove(k2);
        assert_eq!(bag.len(), 1);
        bag.verify_index();
    }

    #[test]
    fn test_is_empty() {
        let mut bag: Bag<i32> = Bag::default();
        assert!(bag.is_empty());

        let k = bag.insert(1);
        assert!(!bag.is_empty());

        bag.remove(k);
        assert!(bag.is_empty());
        bag.verify_index();
    }

    #[test]
    fn test_keys() {
        let mut bag: Bag<i32> = Bag::default();
        let k1 = bag.insert(10);
        let k2 = bag.insert(20);
        let k3 = bag.insert(30);

        bag.remove(k2);

        let expected_keys = HashSet::from([k1, k3]);
        assert_eq!(bag.keys().collect::<HashSet<_>>(), expected_keys);
        bag.verify_index();
    }

    #[test]
    fn test_iter() {
        let mut bag: Bag<i32> = Bag::default();
        bag.insert(1);
        bag.insert(2);
        bag.insert(3);

        let expected_values = HashSet::from([1, 2, 3]);
        let values: HashSet<_> = bag.iter().cloned().collect();
        assert_eq!(values, expected_values);
        bag.verify_index();
    }

    #[test]
    fn test_get_returns_none_on_dead_id() {
        let mut bag: Bag<i32> = Bag::default();
        let k = bag.insert(42);
        bag.remove(k);
        assert!(bag.get(k).is_none());
        bag.verify_index();
    }

    #[test]
    fn test_get_mut_returns_none_on_dead_id() {
        let mut bag: Bag<i32> = Bag::default();
        let k = bag.insert(42);
        bag.remove(k);
        assert!(bag.get_mut(k).is_none());
        bag.verify_index();
    }

    #[test]
    fn test_multiple_inserts_after_removals() {
        let mut bag: Bag<i32> = Bag::default();
        let k1 = bag.insert(1);
        let k2 = bag.insert(2);
        let k3 = bag.insert(3);

        bag.verify_index();
        bag.remove(k1);
        bag.remove(k3);

        bag.verify_index();
        let k4 = bag.insert(4);
        let k5 = bag.insert(5);

        bag.verify_index();
        assert_eq!(bag.len(), 3);
        assert_eq!(bag.get(k2), Some(&2));
        assert_eq!(bag.get(k4), Some(&4));
        assert_eq!(bag.get(k5), Some(&5));
        assert_eq!(
            bag.keys().collect::<HashSet<_>>(),
            HashSet::from([k2, k4, k5])
        );
        assert_eq!(
            bag.iter().collect::<HashSet<_>>(),
            HashSet::from([&2, &4, &5])
        );
        assert_eq!(
            bag.pairs().collect::<HashSet<_>>(),
            HashSet::from([(k2, &2), (k4, &4), (k5, &5)])
        );
    }

    #[quickcheck]
    fn test_aribrary_ops(ops: Vec<BagOp>) {
        let mut bag: Bag<i32> = Bag::default();
        let mut map: HashMap<BagKey, i32> = HashMap::new();
        let mut counter = 0;

        fn get_key(map: &HashMap<BagKey, i32>, idx: usize) -> Option<BagKey> {
            if map.is_empty() {
                None
            } else {
                Some(*map.keys().nth(idx % map.len()).unwrap())
            }
        }

        for op in ops {
            match op {
                BagOp::Insert => {
                    let value = counter;
                    counter += 1;
                    let key = bag.insert(value);
                    map.insert(key, value);
                }
                BagOp::Remove(idx) => {
                    if let Some(key) = get_key(&map, idx) {
                        let removed = bag.remove(key).expect("expected present value");
                        assert_eq!(removed, map.remove(&key).unwrap());
                    }
                }
                BagOp::Mutate(idx) => {
                    if let Some(key) = get_key(&map, idx) {
                        if let Some(val) = bag.get_mut(key) {
                            *val *= 2;
                            map.get_mut(&key).map(|v| *v *= 2);
                        }
                    }
                }
                BagOp::MutateAll() => {
                    for (key, val) in bag.pairs_mut() {
                        *val *= 2;
                        map.get_mut(&key).map(|v| *v *= 2);
                    }
                }
                BagOp::Compact => {
                    let mut key_map = HashMap::new();
                    bag.compact(Some(&mut key_map));
                    let mut new_map = HashMap::new();
                    for (old_key, new_key) in key_map {
                        new_map.insert(
                            new_key,
                            map.remove(&old_key).expect("old key should exist in map"),
                        );
                    }
                    map = new_map;
                }
            }
            bag.verify_index();
            for (key, value) in map.iter() {
                assert_eq!(bag.get(*key), Some(value));
            }
            for (key, value) in bag.pairs() {
                assert_eq!(map.get(&key), Some(value));
            }
            for key in bag.keys() {
                assert!(map.contains_key(&key));
                assert_eq!(bag[key], map[&key]);
            }
        }
    }

    #[test]
    fn test_reserve() {
        let mut bag: Bag<i32> = Bag::default();
        bag.reserve(100);
        assert!(bag.capacity() >= 100);
    }

    #[test]
    fn test_reserve_exact() {
        let mut bag: Bag<i32> = Bag::default();
        bag.reserve_exact(50);
        assert_eq!(bag.capacity(), 50);
    }

    #[test]
    fn test_iter_pairs() {
        let mut bag: Bag<i32> = Bag::default();
        let k1 = bag.insert(10);
        let k2 = bag.insert(20);
        let k3 = bag.insert(30);

        bag.remove(k2);

        assert_eq!(
            bag.pairs().collect::<HashSet<_>>(),
            HashSet::from([(k1, &10), (k3, &30)])
        );
    }

    #[test]
    fn test_iter_pairs_mut() {
        let mut bag: Bag<i32> = Bag::default();
        let k1 = bag.insert(10);
        let k2 = bag.insert(20);
        let k3 = bag.insert(30);

        bag.remove(k2);

        for (k, val) in bag.pairs_mut() {
            *val = (k.to_index() as i32) * 2;
        }

        assert_eq!(
            bag.pairs().collect::<HashSet<_>>(),
            HashSet::from([(k1, &0), (k3, &4)])
        );
    }

    #[test]
    fn test_shrink_to_fit_no_removals() {
        let mut bag: Bag<i32> = Bag::default();
        let id1 = bag.insert(1);
        let id2 = bag.insert(2);
        let id3 = bag.insert(3);

        bag.shrink_to_fit();

        // Old keys should still work
        assert_eq!(bag.get(id1), Some(&1));
        assert_eq!(bag.get(id2), Some(&2));
        assert_eq!(bag.get(id3), Some(&3));
    }

    #[test]
    fn test_bag_large() {
        // Insert a large number of entries, remove many of them in
        // pseudo-random order.
        let mut bag: Bag<i32> = Bag::default();
        let mut map: HashMap<BagKey, i32> = HashMap::new();
        let mut keys = Vec::new();
        let total: usize = 1000;

        for i in 0..total {
            let k = bag.insert(i as i32);
            map.insert(k, i as i32);
            keys.push(k);
        }

        assert_eq!(bag.len(), total);

        let mut key_set: HashSet<_> = keys.into_iter().collect();
        let mut removed = HashSet::new();

        for _ in 0..total {
            assert!(!key_set.is_empty());
            // pick an arbitrary key (HashSet iteration order is effectively random)
            let key = *key_set.iter().next().unwrap();
            key_set.remove(&key);

            let val = bag.remove(key).expect("expected present value");
            assert_eq!(val, map.remove(&key).unwrap());
            removed.insert(key);
        }

        assert_eq!(bag.len(), 0);
    }

    #[test]
    fn test_remove_first() {
        let mut bag = Bag::default();
        let k1 = bag.insert(1);
        let k2 = bag.insert(2);
        let k3 = bag.insert(3);

        assert_eq!(bag.remove(k1), Some(1));
        assert_eq!(bag.len(), 2);
        assert_eq!(bag.get(k1), None);
        assert_eq!(bag.get(k2), Some(&2));
        assert_eq!(bag.get(k3), Some(&3));
    }

    #[test]
    fn test_remove_last() {
        let mut bag = Bag::default();
        let k1 = bag.insert(1);
        let k2 = bag.insert(2);
        let k3 = bag.insert(3);

        assert_eq!(bag.remove(k3), Some(3));
        assert_eq!(bag.len(), 2);
        assert_eq!(bag.get(k3), None);
        assert_eq!(bag.get(k1), Some(&1));
        assert_eq!(bag.get(k2), Some(&2));
    }

    #[test]
    fn test_invalid_key() {
        let mut bag = Bag::default();
        let _k1 = bag.insert(0);

        let invalid_key = BagKey::from_index(999);
        assert_eq!(bag.get(invalid_key), None);
        assert_eq!(bag.get_mut(invalid_key), None);
        assert_eq!(bag.remove(invalid_key), None);
    }

    #[test]
    fn test_multiple_removes() {
        let mut bag = Bag::default();
        let k1 = bag.insert(10);
        let k2 = bag.insert(20);
        let k3 = bag.insert(30);
        let k4 = bag.insert(40);

        assert_eq!(bag.remove(k2), Some(20));
        assert_eq!(bag.remove(k1), Some(10));
        assert_eq!(bag.len(), 2);
        assert_eq!(bag.get(k3), Some(&30));
        assert_eq!(bag.get(k4), Some(&40));
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut bag = Bag::default();
        let k1 = bag.insert(1);
        let k2 = bag.insert(2);
        let k3 = bag.insert(3);

        // Remove some items
        bag.remove(k1);
        bag.remove(k2);

        // Shrink
        bag.shrink_to_fit();

        assert_eq!(bag.len(), 1);
        assert_eq!(bag.get(k3), Some(&3));
    }
}
