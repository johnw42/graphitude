#![cfg(test)]

use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct DropCounter {
    count: Rc<AtomicUsize>,
}

impl DropCounter {
    pub fn new() -> Self {
        DropCounter {
            count: Rc::new(AtomicUsize::new(0)),
        }
    }

    pub fn drop_count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    pub fn new_value(&self) -> DroppableValue {
        DroppableValue {
            counter: Rc::clone(&self.count),
            dropped: false,
        }
    }
}

pub struct DroppableValue {
    counter: Rc<AtomicUsize>,
    dropped: bool,
}

impl Drop for DroppableValue {
    fn drop(&mut self) {
        assert!(!self.dropped, "DroppableValue was dropped more than once");
        self.dropped = true;
        self.counter.fetch_add(1, Ordering::SeqCst);
    }
}
