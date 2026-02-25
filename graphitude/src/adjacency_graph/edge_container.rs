use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use crate::{
    EdgeMultiplicityTrait, MultipleEdges, SingleEdge,
    automap::{Automap, AutomapTrait},
};

pub trait EdgeContainer<T>: Sized {
    type Index: Clone + Debug + Eq + Hash + Ord + Send + Sync;

    /// Returns the number of items in the container.
    fn len(&self) -> usize;

    /// Returns `true` if the container is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Creates a new container with a single item.  Returns the new container, the key for the item, and any replaced item (if applicable).
    fn append(container: Option<Self>, item: T) -> (Self, Self::Index, Option<T>);

    /// Insert an item, possibly replacing an existing item. Returns the key for the item and any replaced item (if applicable).
    fn insert_or_replace(&mut self, data: T) -> (Self::Index, Option<T>);

    /// Gets a reference to the data associated with the edge from `source` to `target`, if it exists.
    fn get(&self, key: Self::Index) -> Option<&T>;

    /// Gets a mutable reference to the data associated with the edge from `source` to `target`, if it exists.
    fn get_mut(&mut self, key: Self::Index) -> Option<&mut T>;

    /// Removed an item, returning a modified container, or `None` if the container would be empty, plus the removed item, if it exists.
    fn without(self, key: Self::Index) -> (Option<Self>, Option<T>);

    fn iter<'a>(&'a self) -> impl Iterator<Item = (Self::Index, &'a T)> + 'a
    where
        T: 'a;
}

/// A single-item edge container
#[derive(Debug, Clone, PartialEq)]
pub struct SingleItem<T>(T);

impl<T> EdgeContainer<T> for SingleItem<T> {
    type Index = ();

    fn len(&self) -> usize {
        1
    }

    fn append(container: Option<Self>, item: T) -> (Self, Self::Index, Option<T>) {
        (SingleItem(item), (), container.map(|c| c.0))
    }

    fn insert_or_replace(&mut self, data: T) -> (Self::Index, Option<T>) {
        let replaced = Some(std::mem::replace(&mut self.0, data));
        ((), replaced)
    }

    fn get(&self, _key: Self::Index) -> Option<&T> {
        Some(&self.0)
    }

    fn get_mut(&mut self, _key: Self::Index) -> Option<&mut T> {
        Some(&mut self.0)
    }

    fn without(self, _key: Self::Index) -> (Option<Self>, Option<T>) {
        (None, Some(self.0))
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (Self::Index, &'a T)> + 'a
    where
        T: 'a,
    {
        std::iter::once(((), &self.0))
    }
}
/// A multi-item edge container
#[derive(Debug, Clone, PartialEq)]
pub struct MultipleItems<T, A>
where
    A: AutomapTrait<T>,
{
    inner: A,
    phantom: PhantomData<T>,
}

impl<T, A> EdgeContainer<T> for MultipleItems<T, A>
where
    A: AutomapTrait<T> + Default,
{
    type Index = A::Key;

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn append(container: Option<Self>, item: T) -> (Self, Self::Index, Option<T>) {
        let mut inner = container.map(|c| c.inner).unwrap_or_default();
        let key = inner.insert(item);
        (
            MultipleItems {
                inner,
                phantom: PhantomData,
            },
            key,
            None,
        )
    }

    fn insert_or_replace(&mut self, data: T) -> (Self::Index, Option<T>) {
        (self.inner.insert(data), None)
    }

    fn get(&self, key: Self::Index) -> Option<&T> {
        self.inner.get(key)
    }

    fn get_mut(&mut self, key: Self::Index) -> Option<&mut T> {
        self.inner.get_mut(key)
    }

    fn without(mut self, key: Self::Index) -> (Option<Self>, Option<T>) {
        let removed = self.inner.remove(key);
        if self.inner.is_empty() {
            (None, removed)
        } else {
            (Some(self), removed)
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (Self::Index, &'a T)> + 'a
    where
        T: 'a,
    {
        self.inner.iter_pairs()
    }
}

pub trait EdgeContainerSelector: EdgeMultiplicityTrait + Default {
    type Container<T>: EdgeContainer<T>;
}

impl EdgeContainerSelector for SingleEdge {
    type Container<T> = SingleItem<T>;
}

impl EdgeContainerSelector for MultipleEdges {
    type Container<T> = MultipleItems<T, Automap<T>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== SingleItem Tests ====================

    #[test]
    fn test_single_item_append_with_none() {
        let (container, _key, replaced) = SingleItem::append(None, 42);
        assert_eq!(container.0, 42);
        assert_eq!(replaced, None);
    }

    #[test]
    fn test_single_item_append_with_some() {
        let original = SingleItem(10);
        let (container, _key, replaced) = SingleItem::append(Some(original), 42);
        assert_eq!(container.0, 42);
        assert_eq!(replaced, Some(10));
    }

    #[test]
    fn test_single_item_insert_or_replace() {
        let mut container = SingleItem(42);
        let (_, replaced) = container.insert_or_replace(99);
        assert_eq!(container.0, 99);
        assert_eq!(replaced, Some(42));
    }

    #[test]
    fn test_single_item_get() {
        let container = SingleItem(42);
        assert_eq!(container.get(()), Some(&42));
    }

    #[test]
    fn test_single_item_get_mut() {
        let mut container = SingleItem(42);
        if let Some(val) = container.get_mut(()) {
            *val = 99;
        }
        assert_eq!(container.0, 99);
    }

    #[test]
    fn test_single_item_without() {
        let container = SingleItem(42);
        let (remaining, removed) = container.without(());
        assert_eq!(remaining, None);
        assert_eq!(removed, Some(42));
    }

    #[test]
    fn test_single_item_iter() {
        let container = SingleItem(42);
        let items: Vec<_> = container.iter().collect();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], ((), &42));
    }

    // ==================== MultipleItems Tests ====================

    #[test]
    fn test_multiple_items_append_with_none() {
        let (container, key, replaced): (MultipleItems<i32, Automap<i32>>, _, _) =
            MultipleItems::append(None, 42);
        assert_eq!(container.get(key), Some(&42));
        assert_eq!(replaced, None);
    }

    #[test]
    fn test_multiple_items_append_with_some() {
        let original: MultipleItems<i32, Automap<i32>> = MultipleItems::append(None, 10).0;

        let (container, _key, replaced) = MultipleItems::append(Some(original), 42);
        // When appending to an existing MultipleItems, the new item is added
        let items: Vec<_> = container.iter().map(|(_, &v)| v).collect();
        assert!(items.contains(&42));
        assert_eq!(replaced, None);
    }

    #[test]
    fn test_multiple_items_insert_multiple() {
        let mut container: MultipleItems<i32, Automap<i32>> = MultipleItems::append(None, 1).0;
        let (k2, replaced) = container.insert_or_replace(2);
        assert!(replaced.is_none());
        let (k3, replaced) = container.insert_or_replace(3);
        assert!(replaced.is_none());

        assert_eq!(container.get(k2), Some(&2));
        assert_eq!(container.get(k3), Some(&3));
    }

    #[test]
    fn test_multiple_items_get() {
        let container: MultipleItems<i32, Automap<i32>> = MultipleItems::append(None, 42).0;
        let k = container
            .iter()
            .next()
            .map(|(k, _)| k)
            .expect("should have one item");
        assert_eq!(container.get(k), Some(&42));
    }

    #[test]
    fn test_multiple_items_get_mut() {
        let mut container: MultipleItems<i32, Automap<i32>> = MultipleItems::append(None, 42).0;
        let k = container
            .iter()
            .next()
            .map(|(k, _)| k)
            .expect("should have one item");

        if let Some(val) = container.get_mut(k) {
            *val = 99;
        }
        assert_eq!(container.get(k), Some(&99));
    }

    #[test]
    fn test_multiple_items_without_not_empty() {
        let mut container: MultipleItems<i32, Automap<i32>> = MultipleItems::append(None, 1).0;
        let k1 = container
            .iter()
            .next()
            .map(|(k, _)| k)
            .expect("should have one item");
        container.insert_or_replace(2);

        let (remaining, removed) = container.without(k1);
        assert_eq!(removed, Some(1));
        assert!(remaining.is_some());
        if let Some(rem) = remaining {
            assert_eq!(rem.iter().count(), 1);
        }
    }

    #[test]
    fn test_multiple_items_without_empty() {
        let container: MultipleItems<i32, Automap<i32>> = MultipleItems::append(None, 42).0;
        let k = container
            .iter()
            .next()
            .map(|(k, _)| k)
            .expect("should have one item");

        let (remaining, removed) = container.without(k);
        assert!(remaining.is_none());
        assert_eq!(removed, Some(42));
    }

    #[test]
    fn test_multiple_items_iter() {
        let mut container: MultipleItems<i32, Automap<i32>> = MultipleItems::append(None, 1).0;
        container.insert_or_replace(2);
        container.insert_or_replace(3);

        let items: Vec<_> = container.iter().map(|(_, &v)| v).collect();
        assert_eq!(items.len(), 3);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
        assert!(items.contains(&3));
    }

    #[test]
    fn test_multiple_items_get_after_removal() {
        let mut container: MultipleItems<i32, Automap<i32>> = MultipleItems::append(None, 1).0;
        let k1 = container
            .iter()
            .next()
            .map(|(k, _)| k)
            .expect("should have one item");
        container.insert_or_replace(2);

        // Remove the first item
        let (remaining, removed) = container.without(k1);
        assert_eq!(removed, Some(1));

        // After removal, the removed key should not return anything
        if let Some(rem) = remaining {
            assert_eq!(rem.get(k1), None);
        }
    }
}
