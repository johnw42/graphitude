/// An iterator wrapper that provides an exact size hint, even if the underlying iterator does not.
pub struct KnownSizeIterator<I> {
    iter: I,
    len: usize,
}

impl<I> KnownSizeIterator<I> {
    pub fn new(iter: I, len: usize) -> Self {
        Self { iter, len }
    }
}

impl<I> Iterator for KnownSizeIterator<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next();
        if next.is_some() {
            self.len -= 1;
        }
        next
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<I> ExactSizeIterator for KnownSizeIterator<I>
where
    I: Iterator,
{
    fn len(&self) -> usize {
        self.len
    }
}
