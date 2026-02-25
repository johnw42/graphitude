use std::{fmt::Debug, hash::Hash};

use derivative::Derivative;

use crate::{EdgeMultiplicityTrait, MultipleEdges, SingleEdge};

#[allow(clippy::len_without_is_empty)]
pub trait EdgeContainer<T>: Sized {
    type Index: Clone + Copy + Debug + Eq + Hash + Ord + Send + Sync;

    /// Returns the number of items in the container.
    fn len(&self) -> usize;

    /// Create a new container by adding an item to an existing container, returning the new container, the index of the added item, and the replaced item, if any.
    fn new(container: Option<Self>, item: T) -> (Self, Self::Index, Option<T>);

    /// Gets a reference to the data associated with the edge from `source` to `target`, if it exists.
    fn get(&self, key: Self::Index) -> Option<&T>;

    /// Gets a mutable reference to the data associated with the edge from `source` to `target`, if it exists.
    fn get_mut(&mut self, key: Self::Index) -> Option<&mut T>;

    /// Removes an item, returning a modified container, or `None` if the container would be empty, plus the removed item, if it exists.
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

    fn new(container: Option<Self>, item: T) -> (Self, Self::Index, Option<T>) {
        (SingleItem(item), (), container.map(|c| c.0))
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
pub struct MultipleItems<T>(Box<MultipleItemsNode<T>>);

pub struct MultipleItemsNode<T> {
    data: T,
    next: Option<MultipleItems<T>>,
}

#[derive(Derivative)]
#[derivative(
    Clone(bound = ""),
    Copy(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Hash(bound = ""),
    Ord(bound = ""),
    PartialOrd(bound = "")
)]
pub struct MultipleItemsIndex<T>(*const MultipleItemsNode<T>);

impl<T> Debug for MultipleItemsIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultipleItemsIndex({:p})", self.0)
    }
}

unsafe impl<T> Send for MultipleItemsIndex<T> {}
unsafe impl<T> Sync for MultipleItemsIndex<T> {}

impl<T> EdgeContainer<T> for MultipleItems<T> {
    type Index = MultipleItemsIndex<T>;

    fn len(&self) -> usize {
        self.iter().count()
    }

    fn new(container: Option<Self>, item: T) -> (Self, Self::Index, Option<T>) {
        let new_node = Box::new(MultipleItemsNode {
            data: item,
            next: container,
        });
        let index = MultipleItemsIndex(&*new_node);
        (MultipleItems(new_node), index, None)
    }

    fn get(&self, key: Self::Index) -> Option<&T> {
        let mut node = self;
        loop {
            if std::ptr::eq(&*node.0, key.0) {
                return Some(&node.0.data);
            }
            node = node.0.next.as_ref()?;
        }
    }

    fn get_mut(&mut self, key: Self::Index) -> Option<&mut T> {
        let mut node = self;
        loop {
            if std::ptr::eq(&*node.0, key.0) {
                return Some(&mut node.0.data);
            }
            node = node.0.next.as_mut()?;
        }
    }

    fn without(mut self, key: Self::Index) -> (Option<Self>, Option<T>) {
        if std::ptr::eq(&*self.0, key.0) {
            // Removing the head of the list
            return (self.0.next, Some(self.0.data));
        }

        let mut current = &mut self;
        loop {
            if current.0.next.is_none() {
                return (Some(self), None); // Reached the end of the list without finding the item
            }
            if current
                .0
                .next
                .as_ref()
                .map(|next| std::ptr::eq(&*next.0, key.0))
                .unwrap_or(false)
            {
                // Found the item to remove
                let MultipleItemsNode { data, next } = *current.0.next.take().unwrap().0;
                current.0.next = next;
                return (Some(self), Some(data));
            }
            current = current.0.next.as_mut().unwrap();
        }
    }

    fn iter<'a>(&'a self) -> impl Iterator<Item = (Self::Index, &'a T)> + 'a
    where
        T: 'a,
    {
        MultipleItemsIterator {
            current: Some(self),
        }
    }
}

pub struct MultipleItemsIterator<'a, T> {
    current: Option<&'a MultipleItems<T>>,
}

impl<'a, T> Iterator for MultipleItemsIterator<'a, T>
where
    T: 'a,
{
    type Item = (MultipleItemsIndex<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current.take() {
            let index = MultipleItemsIndex(current.0.as_ref());
            let value = &current.0.data;
            self.current = current.0.next.as_ref();
            Some((index, value))
        } else {
            None
        }
    }
}

pub trait EdgeContainerSelector: EdgeMultiplicityTrait + Default {
    type Container<T>: EdgeContainer<T>;
}

impl EdgeContainerSelector for SingleEdge {
    type Container<T> = SingleItem<T>;
}

impl EdgeContainerSelector for MultipleEdges {
    type Container<T> = MultipleItems<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== SingleItem Tests ====================

    #[test]
    fn test_single_item_new_with_none() {
        let (container, _key, replaced) = SingleItem::new(None, 42);
        assert_eq!(container.0, 42);
        assert_eq!(replaced, None);
    }

    #[test]
    fn test_single_item_new_with_some() {
        let original = SingleItem(10);
        let (container, _key, replaced) = SingleItem::new(Some(original), 42);
        assert_eq!(container.0, 42);
        assert_eq!(replaced, Some(10));
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

    fn new_container<T>(items: Vec<T>) -> (MultipleItems<T>, Vec<MultipleItemsIndex<T>>) {
        let mut keys = Vec::new();
        let container = items.into_iter().fold(None, |container, item| {
            let (new_container, key, removed) = MultipleItems::new(container, item);
            assert!(
                removed.is_none(),
                "new_container should not replace existing items"
            );
            keys.push(key);
            Some(new_container)
        });
        (container.unwrap(), keys)
    }

    #[test]
    fn test_multiple_items_new_with_none() {
        let (container, keys) = new_container(vec![42]);
        assert_eq!(container.get(keys[0]), Some(&42));
    }

    #[test]
    fn test_multiple_items_insert_multiple() {
        let (container, keys) = new_container(vec![1, 2, 3]);
        assert_eq!(container.len(), 3);
        assert_eq!(container.get(keys[0]), Some(&1));
        assert_eq!(container.get(keys[1]), Some(&2));
        assert_eq!(container.get(keys[2]), Some(&3));
    }

    #[test]
    fn test_multiple_items_get() {
        let (container, keys) = new_container(vec![42]);
        assert_eq!(container.get(keys[0]), Some(&42));
    }

    #[test]
    fn test_multiple_items_get_mut() {
        let (mut container, keys) = new_container(vec![42]);
        if let Some(val) = container.get_mut(keys[0]) {
            *val = 99;
        }
        assert_eq!(container.get(keys[0]), Some(&99));
    }

    #[test]
    fn test_multiple_items_without_not_empty() {
        let (container, keys) = new_container(vec![1, 2]);

        let (remaining, removed) = container.without(keys[0]);
        assert_eq!(removed, Some(1));
        assert!(remaining.is_some());
        if let Some(rem) = remaining {
            assert_eq!(rem.iter().count(), 1);
        }
    }

    #[test]
    fn test_multiple_items_without_empty() {
        let (container, keys) = new_container(vec![42]);

        let (remaining, removed) = container.without(keys[0]);
        assert!(remaining.is_none());
        assert_eq!(removed, Some(42));
    }

    #[test]
    fn test_multiple_items_iter() {
        let (container, _keys) = new_container(vec![1, 2, 3]);

        let items: Vec<_> = container.iter().map(|(_, &v)| v).collect();
        assert_eq!(items.len(), 3);
        assert!(items.contains(&1));
        assert!(items.contains(&2));
        assert!(items.contains(&3));
    }

    #[test]
    fn test_multiple_items_get_after_removal() {
        let (container, keys) = new_container(vec![1, 2]);

        // Remove the first item
        let (remaining, removed) = container.without(keys[0]);
        assert_eq!(removed, Some(1));

        // After removal, the removed key should not return anything
        if let Some(rem) = remaining {
            assert_eq!(rem.get(keys[0]), None);
        }
    }
}
