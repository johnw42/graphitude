#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdVecKey(usize);

impl super::trait_def::IdVecKeyTrait for IdVecKey {}

/// Helper struct for indexing into an `IndexedIdVec`.
pub struct IndexedIdVecIndexing;

impl super::trait_def::IdVecIndexing for IndexedIdVecIndexing {
    type Key = IdVecKey;

    /// For IndexedIdVec, keys are already zero-based indices.
    fn zero_based_index(&self, index: IdVecKey) -> usize {
        index.0
    }

    /// For IndexedIdVec, keys are already zero-based indices.
    fn key_from_index(&self, index: usize) -> IdVecKey {
        IdVecKey(index)
    }
}

/// An `IdVec` implementation that maintains a mapping from stable keys to
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
pub struct IndexedIdVec<T> {
    /// The data with reverse indices, stored in arbitrary order.
    /// Each element is (value, reverse_index) where reverse_index maps back to the logical position.
    data: Vec<(T, usize)>,
    /// Maps from logical indices to positions in the data vector. None indicates the key was removed.
    index: Vec<Option<usize>>,
}

impl<T> Default for IndexedIdVec<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            index: Vec::new(),
        }
    }
}

impl<T> super::trait_def::IdVec<T> for IndexedIdVec<T> {
    type Key = IdVecKey;
    type Indexing = IndexedIdVecIndexing;

    fn insert(&mut self, value: T) -> IdVecKey {
        let id = self.index.len();
        self.data.push((value, id));
        self.index.push(Some(self.data.len() - 1));
        IdVecKey(id)
    }

    fn get(&self, key: IdVecKey) -> Option<&T> {
        self.index
            .get(key.0)
            .and_then(|opt_data_idx| opt_data_idx.map(|data_idx| &self.data[data_idx].0))
    }

    fn get_mut(&mut self, key: IdVecKey) -> Option<&mut T> {
        if key.0 >= self.index.len() {
            return None;
        }
        match self.index[key.0] {
            Some(data_idx) => Some(&mut self.data[data_idx].0),
            None => None,
        }
    }

    fn remove(&mut self, key: IdVecKey) -> Option<T> {
        if key.0 >= self.index.len() {
            return None;
        }

        // Mark as removed first
        let data_idx = self.index[key.0].take()?;
        let (removed_value, _) = self.data.swap_remove(data_idx);

        // If an element was moved from the end, update its index mapping
        if data_idx < self.data.len() {
            let moved_logical_id = self.data[data_idx].1;
            self.index[moved_logical_id] = Some(data_idx);
        }

        Some(removed_value)
    }

    fn len(&self) -> usize {
        self.data.len()
    }

    fn clear(&mut self) {
        self.data.clear();
        self.index.clear();
    }

    fn capacity(&self) -> usize {
        self.data.capacity()
    }

    fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
        self.index.reserve(additional);
    }

    fn reserve_exact(&mut self, additional: usize) {
        self.data.reserve_exact(additional);
        self.index.reserve_exact(additional);
    }

    fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
        self.index.shrink_to_fit();
    }

    fn shrink_to_fit_with(&mut self, callback: impl FnMut(Self::Key, Option<Self::Key>)) {
        self.shrink_to_fit();
    }

    fn iter_keys(&self) -> impl Iterator<Item = IdVecKey> {
        self.index
            .iter()
            .enumerate()
            .filter_map(|(i, opt)| opt.as_ref().map(|_| IdVecKey(i)))
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.data.iter().map(|(value, _)| value)
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        self.data.iter_mut().map(|(value, _)| value)
    }

    fn iter_pairs<'a>(&'a self) -> impl Iterator<Item = (IdVecKey, &'a T)>
    where
        T: 'a,
    {
        self.data
            .iter()
            .map(|(value, logical_id)| (IdVecKey(*logical_id), value))
    }

    fn iter_pairs_mut<'a>(&'a mut self) -> impl Iterator<Item = (IdVecKey, &'a mut T)>
    where
        T: 'a,
    {
        self.data
            .iter_mut()
            .map(|(value, logical_id)| (IdVecKey(*logical_id), value))
    }

    fn indexing(&self) -> IndexedIdVecIndexing {
        IndexedIdVecIndexing
    }

    fn compact(&mut self) {}

    fn compact_with(&mut self, callback: impl FnMut(Self::Key, Option<Self::Key>)) {}
}

#[cfg(test)]
mod tests {
    use super::super::trait_def::IdVec;
    use super::*;
    use crate::idvec_tests;

    idvec_tests!(IndexedIdVec<i32>, i32, |i: i32| i);

    #[test]
    fn test_remove_first() {
        let mut vec = IndexedIdVec::default();
        let k1 = vec.insert(1);
        let k2 = vec.insert(2);
        let k3 = vec.insert(3);

        assert_eq!(vec.remove(k1), Some(1));
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(k1), None);
        assert_eq!(vec.get(k2), Some(&2));
        assert_eq!(vec.get(k3), Some(&3));
    }

    #[test]
    fn test_remove_last() {
        let mut vec = IndexedIdVec::default();
        let k1 = vec.insert(1);
        let k2 = vec.insert(2);
        let k3 = vec.insert(3);

        assert_eq!(vec.remove(k3), Some(3));
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(k3), None);
        assert_eq!(vec.get(k1), Some(&1));
        assert_eq!(vec.get(k2), Some(&2));
    }

    #[test]
    fn test_invalid_key() {
        let mut vec = IndexedIdVec::default();
        let _k1 = vec.insert(0);

        let invalid_key = IdVecKey(999);
        assert_eq!(vec.get(invalid_key), None);
        assert_eq!(vec.get_mut(invalid_key), None);
        assert_eq!(vec.remove(invalid_key), None);
    }

    #[test]
    fn test_multiple_removes() {
        let mut vec = IndexedIdVec::default();
        let k1 = vec.insert(10);
        let k2 = vec.insert(20);
        let k3 = vec.insert(30);
        let k4 = vec.insert(40);

        assert_eq!(vec.remove(k2), Some(20));
        assert_eq!(vec.remove(k1), Some(10));
        assert_eq!(vec.len(), 2);
        assert_eq!(vec.get(k3), Some(&30));
        assert_eq!(vec.get(k4), Some(&40));
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut vec = IndexedIdVec::default();
        let k1 = vec.insert(1);
        let k2 = vec.insert(2);
        let k3 = vec.insert(3);

        // Remove some items
        vec.remove(k1);
        vec.remove(k2);

        // Shrink
        vec.shrink_to_fit();

        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(k3), Some(&3));
    }
}
