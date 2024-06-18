use core::borrow::Borrow;
use core::fmt::{Debug, Formatter, Result};

use crate::specialized_maps::ScanningMap;
use crate::specialized_sets::{Iter, Set};
use crate::traits::len::Len;

// TODO: Implement PartialEq + Eq

/// A set that does a linear scan of its values upon lookup, designed for very small payloads.
#[derive(Clone)]
pub struct ScanningSet<T> {
    map: ScanningMap<T, ()>,
}

impl<T> ScanningSet<T>
where
    T: Eq,
{
    #[must_use]
    pub fn from_vec(payload: Vec<T>) -> Self {
        Self {
            map: payload.into_iter().map(|x| (x, ())).collect(),
        }
    }
}

impl<T> ScanningSet<T> {
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Eq,
    {
        Some(self.map.get_key_value(value)?.0)
    }

    #[inline]
    #[must_use]
    pub fn get_by_index(&self, index: usize) -> Option<&T> {
        Some(self.map.get_by_index(index)?.0)
    }

    #[inline]
    #[must_use]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq,
    {
        self.get(value).is_some()
    }

    #[must_use]
    pub const fn iter(&self) -> Iter<T> {
        Iter::new(&self.map.entries)
    }
}

impl<T> Len for ScanningSet<T> {
    fn len(&self) -> usize {
        self.map.len()
    }
}

impl<T> Debug for ScanningSet<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.map.fmt(f) // TODO: can we do better here?
    }
}

impl<'a, T> IntoIterator for &'a ScanningSet<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, const N: usize> From<[T; N]> for ScanningSet<T>
where
    T: Eq,
{
    fn from(payload: [T; N]) -> Self {
        Self::from_vec(Vec::from_iter(payload))
    }
}

impl<T> FromIterator<T> for ScanningSet<T>
where
    T: Eq,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(Vec::from_iter(iter))
    }
}

impl<T> Set<T> for ScanningSet<T>
where
    T: Eq,
{
    type Iterator<'a> = Iter<'a, T>
    where
        T: 'a;

    fn iter(&self) -> Iter<'_, T> {
        self.iter()
    }

    fn contains(&self, value: &T) -> bool {
        self.contains(value)
    }
}
