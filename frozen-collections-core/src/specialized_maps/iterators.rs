use std::fmt::{Debug, Formatter, Result};
use std::iter::FusedIterator;

/// An iterator over the entries of a map.
pub struct Iter<'a, K, V> {
    entries: &'a [(K, V)],
    index: usize,
}

impl<'a, K, V> Iter<'a, K, V> {
    #[must_use]
    pub const fn new(entries: &'a [(K, V)]) -> Self {
        Self { entries, index: 0 }
    }
}

impl<'a, K, V> Clone for Iter<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries,
            index: self.index,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.entries.len() {
            let entry = &self.entries[self.index];
            self.index += 1;
            Some((&entry.0, &entry.1))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }
}

impl<'a, K, V> ExactSizeIterator for Iter<'a, K, V> {
    fn len(&self) -> usize {
        self.entries.len() - self.index
    }
}

impl<'a, K, V> FusedIterator for Iter<'a, K, V> {}

impl<'a, K, V> Debug for Iter<'a, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// An iterator over the keys of a map.
pub struct Keys<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Keys<'a, K, V> {
    #[must_use]
    pub const fn new(entries: &'a [(K, V)]) -> Self {
        Self {
            inner: Iter::new(entries),
        }
    }
}

impl<'a, K, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, K, V> Iterator for Keys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.0)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn count(self) -> usize {
        self.inner.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, |acc, (k, _)| f(acc, k))
    }
}

impl<'a, K, V> ExactSizeIterator for Keys<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K, V> FusedIterator for Keys<'a, K, V> {}

impl<'a, K, V> Debug for Keys<'a, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list().entries(self.clone()).finish()
    }
}

/// An iterator over the values of a map.
pub struct Values<'a, K, V> {
    inner: Iter<'a, K, V>,
}

impl<'a, K, V> Values<'a, K, V> {
    #[must_use]
    pub const fn new(entries: &'a [(K, V)]) -> Self {
        Self {
            inner: Iter::new(entries),
        }
    }
}

impl<'a, K, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<'a, K, V> Iterator for Values<'a, K, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|x| x.1)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    fn count(self) -> usize {
        self.inner.count()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.inner.fold(init, |acc, (_, v)| f(acc, v))
    }
}

impl<'a, K, V> ExactSizeIterator for Values<'a, K, V> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a, K, V> FusedIterator for Values<'a, K, V> {}

impl<'a, K, V> Debug for Values<'a, K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list().entries(self.clone()).finish()
    }
}
