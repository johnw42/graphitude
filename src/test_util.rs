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
        DroppableValue(self)
    }
}

pub struct DroppableValue<'a>(&'a DropCounter);

impl<'a> Drop for DroppableValue<'a> {
    fn drop(&mut self) {
        self.0.count.set(self.0.count.get() + 1);
    }
}
