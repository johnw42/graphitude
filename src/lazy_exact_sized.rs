use std::{cell::UnsafeCell, mem::MaybeUninit};

pub struct LazyExactSizedIterator<I, T>(UnsafeCell<MaybeUninit<Inner<I, T>>>);

type EagerIterator<T> = <Vec<T> as IntoIterator>::IntoIter;

enum Inner<I, T> {
    Lazy(I),
    Eager(EagerIterator<T>),
}

impl<I, T> LazyExactSizedIterator<I, T>
where
    I: Iterator<Item = T>,
{
    pub fn new(iter: I) -> Self {
        Self(UnsafeCell::new(MaybeUninit::new(Inner::Lazy(iter))))
    }
}

impl<I, T> Iterator for LazyExactSizedIterator<I, T>
where
    I: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: We are replacing the inner value with an uninitialized one,
        // but we immediately replace it again with a fully initialized value
        // before any other code can access it. This is safe because we never
        // expose the uninitialized value to any other code.
        unsafe {
            let mut inner = (*self.0.get()).assume_init_read();
            let result = match &mut inner {
                Inner::Lazy(iter) => iter.next(),
                Inner::Eager(eager_iter) => eager_iter.next(),
            };
            *self.0.get() = MaybeUninit::new(inner);
            result
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl<I, T> ExactSizeIterator for LazyExactSizedIterator<I, T>
where
    I: Iterator<Item = T>,
{
    fn len(&self) -> usize {
        // SAFETY: We are replacing the inner value with an uninitialized one,
        // but we immediately replace it again with a fully initialized value
        // before any other code can access it. This is safe because we never
        // expose the uninitialized value to any other code.
        let inner = unsafe { self.0.get().replace(MaybeUninit::uninit()).assume_init() };
        let eager_iter = match inner {
            Inner::Lazy(iter) => iter.collect::<Vec<T>>().into_iter(),
            Inner::Eager(eager_iter) => eager_iter,
        };
        let len = eager_iter.len();
        unsafe {
            *self.0.get() = MaybeUninit::new(Inner::Eager(eager_iter));
        }
        len
    }
}

impl<I, T> Drop for LazyExactSizedIterator<I, T> {
    fn drop(&mut self) {
        // SAFETY: The value inside the UnsafeCell is always initialized, so we
        // can safely drop it.
        unsafe {
            (*self.0.get()).assume_init_drop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lazy_exact_sized_iterator() {
        let iter = LazyExactSizedIterator::new(0..5);
        assert_eq!(iter.len(), 5);
        assert_eq!(iter.collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
    }
}
