use std::{
    fmt::Debug,
    num::NonZero,
    ops::{Index, IndexMut},
};

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

/// A bag implementation that maintains a mapping from stable keys to
/// values using a dense index mapping. This allows for O(1) insertions and
/// removals by swapping removed elements with the last element in the data
/// vector, while keeping track of the logical position of each element through
/// a separate index vector.  The `data` vector stores the values along with
/// their logical IDs, while the `index` vector maps logical IDs to their
/// positions in the `data` vector. When an element is removed, its logical ID
/// is marked as `None` in the `index` vector, and the last element in the
/// `data` vector is moved to fill the gap, with its logical ID updated
/// accordingly. This approach allows for efficient memory usage and fast
/// operations while maintaining stable keys for accessing values.
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

    pub fn insert(&mut self, value: T) -> BagKey {
        let id = BagKey::from_index(self.index.len());
        let data_key = BagKey::from_index(self.data.len());
        self.data.push((value, id));
        self.index.push(Some(data_key));
        id
    }

    pub fn get(&self, key: BagKey) -> Option<&T> {
        let data_key = (*self.index.get(key.to_index())?)?;
        Some(&self.data[data_key.to_index()].0)
    }

    pub fn get_mut(&mut self, key: BagKey) -> Option<&mut T> {
        if key.to_index() >= self.index.len() {
            return None;
        }
        match self.index[key.to_index()] {
            Some(data_key) => Some(&mut self.data[data_key.to_index()].0),
            None => None,
        }
    }

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

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.index.clear();
    }

    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
        self.index.reserve(additional);
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
        self.index.reserve_exact(additional);
    }

    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.index.shrink_to_fit();
    }

    pub fn iter_keys(&self) -> impl Iterator<Item = BagKey> {
        self.index
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|_| BagKey::from_index(i)))
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.data.iter().map(|(value, _)| value)
    }

    pub fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        self.data.iter_mut().map(|(value, _)| value)
    }

    pub fn iter_pairs<'a>(&'a self) -> impl Iterator<Item = (BagKey, &'a T)>
    where
        T: 'a,
    {
        self.data
            .iter()
            .map(|(value, logical_id)| (*logical_id, value))
    }

    pub fn iter_pairs_mut<'a>(&'a mut self) -> impl Iterator<Item = (BagKey, &'a mut T)>
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
        for (key, value) in self.iter_pairs() {
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
    }

    impl Arbitrary for BagOp {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u32::arbitrary(g) % 5 {
                0..2 => BagOp::Insert,
                2 => BagOp::Remove(usize::arbitrary(g)),
                3 => BagOp::Mutate(usize::arbitrary(g)),
                4 => BagOp::MutateAll(),
                choice => unreachable!("Invalid value for BagOp: {}", choice),
            }
        }
    }

    #[test]
    fn test_new() {
        let mut bag: Bag<i32> = Bag::default();
        assert!(bag.is_empty());
        assert_eq!(bag.len(), 0);
        assert_eq!(bag.iter_keys().count(), 0);
        assert_eq!(bag.iter().count(), 0);
        assert_eq!(bag.iter_pairs().count(), 0);
        assert_eq!(bag.iter_mut().count(), 0);
        assert_eq!(bag.iter_pairs_mut().count(), 0);
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
        assert_eq!(bag.iter_keys().collect::<Vec<_>>(), vec![k1, k2]);
        assert_eq!(bag.iter().collect::<Vec<_>>(), vec![&1, &2]);
        assert_eq!(
            bag.iter_pairs().collect::<Vec<_>>(),
            vec![(k1, &1), (k2, &2)]
        );
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
        assert_eq!(bag.iter_keys().collect::<Vec<_>>(), vec![k2]);
        assert_eq!(bag.iter().collect::<Vec<_>>(), vec![&100]);
        assert_eq!(bag.iter_pairs().collect::<Vec<_>>(), vec![(k2, &100)]);
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
    fn test_iter_keys() {
        let mut bag: Bag<i32> = Bag::default();
        let _k1 = bag.insert(10);
        let k2 = bag.insert(20);
        let _k3 = bag.insert(30);

        bag.remove(k2);

        let indices: Vec<_> = bag.iter_keys().collect();
        assert_eq!(indices.len(), 2);
        bag.verify_index();
    }

    #[test]
    fn test_iter() {
        let mut bag: Bag<i32> = Bag::default();
        bag.insert(1);
        bag.insert(2);
        bag.insert(3);

        let count = bag.iter().count();
        assert_eq!(count, 3);
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
        assert_eq!(bag.iter_keys().collect::<Vec<_>>(), vec![k2, k4, k5]);
        assert_eq!(bag.iter().collect::<Vec<_>>(), vec![&2, &4, &5]);
        assert_eq!(
            bag.iter_pairs().collect::<Vec<_>>(),
            vec![(k2, &2), (k4, &4), (k5, &5)]
        );
        dbg!(&bag);
        panic!();
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
                    for (key, val) in bag.iter_pairs_mut() {
                        *val *= 2;
                        map.get_mut(&key).map(|v| *v *= 2);
                    }
                }
            }
            bag.verify_index();
            for (key, value) in map.iter() {
                assert_eq!(bag.get(*key), Some(value));
            }
            for (key, value) in bag.iter_pairs() {
                assert_eq!(map.get(&key), Some(value));
            }
            for key in bag.iter_keys() {
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
        let _k1 = bag.insert(10);
        let k2 = bag.insert(20);
        let _k3 = bag.insert(30);

        bag.remove(k2);

        let pairs: Vec<_> = bag.iter_pairs().collect();
        assert_eq!(pairs.len(), 2);
    }

    #[test]
    fn test_iter_pairs_mut() {
        let mut bag: Bag<i32> = Bag::default();
        let _k1 = bag.insert(10);
        let k2 = bag.insert(20);
        let _k3 = bag.insert(30);

        bag.remove(k2);

        for (_k, _val) in bag.iter_pairs_mut() {
            // Just verify iteration works
        }

        let pairs: Vec<_> = bag.iter_pairs().collect();
        assert_eq!(pairs.len(), 2);
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
