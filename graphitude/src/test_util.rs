#![cfg(test)]

use std::{
    marker::PhantomData,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct DropCounter<T = ()> {
    count: Rc<AtomicUsize>,
    value: PhantomData<T>,
}

impl<T> DropCounter<T> {
    pub fn new() -> Self {
        DropCounter {
            count: Rc::new(AtomicUsize::new(0)),
            value: PhantomData,
        }
    }

    pub fn drop_count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    pub fn new_value(&self) -> DroppableValue<T>
    where
        T: Default,
    {
        self.new_value_with(T::default())
    }

    pub fn new_value_with(&self, value: T) -> DroppableValue<T> {
        DroppableValue {
            counter: Rc::clone(&self.count),
            dropped: false,
            value,
        }
    }
}

pub struct DroppableValue<T = ()> {
    counter: Rc<AtomicUsize>,
    dropped: bool,
    value: T,
}

impl<T> DroppableValue<T> {
    #[allow(unused)]
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T> Drop for DroppableValue<T> {
    fn drop(&mut self) {
        assert!(!self.dropped, "DroppableValue was dropped more than once");
        self.dropped = true;
        self.counter.fetch_add(1, Ordering::SeqCst);
    }
}
