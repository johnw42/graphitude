#![allow(unused)]

#[cfg(test)]
use std::cell::RefCell;

use std::{
    collections::HashMap,
    fmt::Debug,
    mem::MaybeUninit,
    ops::{Index, IndexMut, Range},
};

use bitvec::vec::BitVec;

/// An key for an `IdVec`. Stable across insertions and removals, but not
/// across `compact` or `shrink_to_fit` operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IdVecKey(usize);

impl super::trait_def::IdVecKeyTrait for IdVecKey {}

// Needed for AdjacencyGraph implementation.
impl From<IdVecKey> for usize {
    fn from(val: IdVecKey) -> Self {
        val.0
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

impl super::trait_def::IdVecIndexing for IdVecIndexing {
    type Key = IdVecKey;

    fn zero_based_index(&self, index: IdVecKey) -> usize {
        index.0 - self.key_offset
    }

    fn key_from_index(&self, index: usize) -> IdVecKey {
        IdVecKey(index + self.key_offset)
    }
}

/// A map- or bag-like structure that assigns stable keys to inserted values.
/// Keys remain valid across insertions and removals, but not across `compact`
/// or `shrink_to_fit` operations.  Internally uses a `Vec<MaybeUninit<T>>` to
/// store values and a `BitVec` to track live entries.
///
/// Uses an offset-based approach where the key offset can change during compaction.
pub struct OffsetIdVec<T> {
    /// Internal vector storing the values.
    vec: Vec<MaybeUninit<T>>,
    /// BitVec tracking which entries are live (not removed).  This is used to
    /// determine which entries contain an initialized value.
    liveness: BitVec,
    /// The difference between indices stored in keys and actual vector indices.
    key_offset: usize,
}

impl<T> OffsetIdVec<T> {
    /// Helper function to iterate over live indices in the BitVec.  Needs to be
    /// a separate function to satisfy the borrow checker for mutable iterators.
    fn iter_live_indices(liveness: &BitVec) -> impl Iterator<Item = usize> + '_ {
        liveness
            .iter()
            .enumerate()
            .filter_map(move |(i, live)| if *live { Some(i) } else { None })
    }
}

impl<T> Default for OffsetIdVec<T> {
    /// Creates a new, empty OffsetIdVec.
    fn default() -> Self {
        OffsetIdVec {
            vec: Vec::new(),
            liveness: BitVec::new(),
            key_offset: 0,
        }
    }
}

impl<T> super::trait_def::IdVec<T> for OffsetIdVec<T> {
    type Key = IdVecKey;
    type Indexing = IdVecIndexing;

    fn insert(&mut self, value: T) -> IdVecKey {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let next_key = IdVecKey(self.vec.len() + self.key_offset);
        self.vec.push(MaybeUninit::new(value));
        self.liveness.push(true);
        next_key
    }

    fn get(&self, key: IdVecKey) -> Option<&T> {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let index = key.0 - self.key_offset;
        if index < self.liveness.len() && self.liveness[index] {
            Some(unsafe { &*self.vec[index].as_ptr() })
        } else {
            None
        }
    }

    fn get_mut(&mut self, key: IdVecKey) -> Option<&mut T> {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let index = key.0 - self.key_offset;
        if index < self.liveness.len() && self.liveness[index] {
            Some(unsafe { &mut *self.vec[index].as_mut_ptr() })
        } else {
            None
        }
    }

    fn remove(&mut self, key: IdVecKey) -> Option<T> {
        debug_assert_eq!(self.vec.len(), self.liveness.len());
        let index = key.0 - self.key_offset;
        if index < self.vec.len() && self.liveness[index] {
            self.liveness.set(index, false);
            Some(unsafe { self.vec[index].assume_init_read() })
        } else {
            None
        }
    }

    fn len(&self) -> usize {
        self.liveness.count_ones()
    }

    fn clear(&mut self) {
        self.vec.clear();
        self.liveness.clear();
    }

    fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional);
        self.liveness.reserve(additional);
    }

    fn reserve_exact(&mut self, additional: usize) {
        self.vec.reserve_exact(additional);
        self.liveness.reserve_exact(additional);
    }

    fn iter_keys(&self) -> impl Iterator<Item = IdVecKey> {
        Self::iter_live_indices(&self.liveness).map(|index| IdVecKey(index + self.key_offset))
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = &'a T>
    where
        T: 'a,
    {
        self.iter_pairs().map(|(_, value)| value)
    }

    fn iter_mut<'a>(&'a mut self) -> impl Iterator<Item = &'a mut T>
    where
        T: 'a,
    {
        self.iter_pairs_mut().map(|(_, value)| value)
    }

    fn iter_pairs<'a>(&'a self) -> impl Iterator<Item = (IdVecKey, &'a T)>
    where
        T: 'a,
    {
        Self::iter_live_indices(&self.liveness).map(|index| {
            (IdVecKey(index + self.key_offset), unsafe {
                &*self.vec[index].as_ptr()
            })
        })
    }

    fn iter_pairs_mut<'a>(&'a mut self) -> impl Iterator<Item = (IdVecKey, &'a mut T)>
    where
        T: 'a,
    {
        Self::iter_live_indices(&self.liveness).map(|index| {
            (IdVecKey(index + self.key_offset), unsafe {
                &mut *self.vec[index].as_mut_ptr()
            })
        })
    }

    fn indexing(&self) -> IdVecIndexing {
        IdVecIndexing {
            key_offset: self.key_offset,
        }
    }

    fn compact(&mut self) {
        self.compact_with(|_, _| {});
    }

    fn compact_with(&mut self, mut callback: impl FnMut(IdVecKey, Option<IdVecKey>)) {
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
                callback(old_key, Some(new_key));
                di += 1;
            } else {
                callback(old_key, None);
            }
        }
        self.vec.truncate(di);
        self.liveness.truncate(di);
        self.key_offset = new_key_offset;
    }

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit_with(|_, _| {});
    }

    fn shrink_to_fit_with(&mut self, mut callback: impl FnMut(IdVecKey, Option<IdVecKey>)) {
        if self.liveness.all() {
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
                callback(old_key, Some(new_key));
            } else {
                callback(old_key, None);
            }
        }

        self.vec = new_vec;
        self.liveness = new_liveness;
        self.key_offset = new_key_offset;
    }
}

impl<T> Debug for OffsetIdVec<T>
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
    const { RefCell::new(Vec::new()) };
}

impl<T> Drop for OffsetIdVec<T> {
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
    use super::super::trait_def::IdVec;
    use super::*;
    use crate::idvec_tests;

    idvec_tests!(OffsetIdVec<i32>, i32, |i: i32| i);

    #[test]
    #[should_panic]
    fn test_compact_invalidates_all_indices() {
        let mut vec = OffsetIdVec::default();
        let id1 = vec.insert(1);
        let id2 = vec.insert(2);

        vec.remove(id1);
        vec.compact();

        let _ = vec.get(id2);
    }

    #[test]
    fn test_drop() {
        DROPPED_ENTRIES.with_borrow_mut(|dropped_entries| {
            dropped_entries.clear();
        });

        let mut vec = OffsetIdVec::default();
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
}
