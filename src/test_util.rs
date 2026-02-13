#![cfg(test)]

use std::cell::Cell;

pub struct DropCounter {
    count: Cell<usize>,
}

impl DropCounter {
    pub fn new() -> Self {
        DropCounter {
            count: Cell::new(0),
        }
    }

    pub fn drop_count(&self) -> usize {
        self.count.get()
    }

    pub fn new_value(&self) -> DroppableValue<'_> {
        DroppableValue {
            counter: self,
            dropped: false,
        }
    }
}

pub struct DroppableValue<'a> {
    counter: &'a DropCounter,
    dropped: bool,
}

impl<'a> Drop for DroppableValue<'a> {
    fn drop(&mut self) {
        assert!(!self.dropped, "DroppableValue was dropped more than once");
        self.dropped = true;
        self.counter.count.set(self.counter.count.get() + 1);
    }
}
