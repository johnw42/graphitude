use std::{collections::HashMap, hash::Hash};

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

    // fn try_into_hash_map_collector(self) -> Result<Box<dyn HashMapCollector<'a, T>>, Self>
    // where
    //     Self: Sized,
    //     T: Hash + Eq,
    // {
    //     Err(self)
    // }
}

pub trait HashMapCollector<T>: MapCollector<T>
where
    T: Hash + Eq,
{
    fn hash_map(&mut self) -> &mut HashMap<T, T>;
}

impl<T> MapCollector<T> for &mut HashMap<T, T>
where
    T: Hash + Eq,
{
    fn insert(&mut self, old_key: T, new_key: T) {
        (*self).insert(old_key, new_key);
    }

    // fn try_into_hash_map_collector(self) -> Result<Box<dyn HashMapCollector<'a, T> + 'a>, Self>
    // where
    //     T: Hash + Eq,
    // {
    //     if self.is_empty() {
    //         Ok(Box::new(self))
    //     } else {
    //         Err(self)
    //     }
    // }
}

/// Implementation allowing a `&mut HashMap` to be used directly as a MapCollector.
impl<T> HashMapCollector<T> for &mut HashMap<T, T>
where
    T: Hash + Eq,
{
    fn hash_map(&mut self) -> &mut HashMap<T, T> {
        self
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

// // A wrapper that implements `HashMapCollector` by forwarding to an underlying `MapCollector` and storing the mappings in a `HashMap`.
// pub struct ForwardingHashMapCollector<T, C> {
//     target: Option<C>,
//     map: HashMap<T, T>,
// }

// impl<'a, T, C> ForwardingHashMapCollector<T, C>
// where
//     C: MapCollector<'a, T>,
// {
//     pub fn new_from(collector: Option<C>) -> Box<dyn HashMapCollector<'a, T>>
//     where
//         Self: Sized,
//         C: MapCollector<'a, T> + Sized,
//         T: Clone + Hash + Eq + 'a,
//     {
//         if let Some(collector) = collector {
//             match collector.try_into_hash_map_collector() {
//                 Ok(hash_map_collector) => hash_map_collector,
//                 Err(target) => Box::new(ForwardingHashMapCollector {
//                     target: Some(target),
//                     map: HashMap::new(),
//                 }),
//             }
//         } else {
//             Box::new(ForwardingHashMapCollector {
//                 target: Option::<C>::None,
//                 map: HashMap::new(),
//             })
//         }
//     }
// }

// impl<'a, T, C> MapCollector<'a, T> for ForwardingHashMapCollector<T, C>
// where
//     C: MapCollector<'a, T>,
//     T: Clone + Hash + Eq + 'a,
// {
//     fn insert(&mut self, old_key: T, new_key: T) {
//         if let Some(target) = &mut self.target {
//             target.insert(old_key.clone(), new_key.clone());
//         }
//         self.map.insert(old_key, new_key);
//     }

//     fn try_into_hash_map_collector(self) -> Result<Box<dyn HashMapCollector<'a, T>>, Self>
//     where
//         Self: Sized,
//         T: Hash + Eq,
//     {
//         Ok(Box::new(self))
//     }
// }

// impl<'a, T, C> HashMapCollector<'a, T> for ForwardingHashMapCollector<T, C>
// where
//     C: MapCollector<'a, T>,
//     T: Clone + Hash + Eq + 'a,
// {
//     fn hash_map(&mut self) -> &mut HashMap<T, T> {
//         &mut self.map
//     }
// }
