use core::borrow::Borrow;
use core::fmt::{Debug, Formatter, Result};

use num_traits::{PrimInt, Unsigned};

use crate::specialized_maps::LengthMap;
use crate::specialized_sets::{Iter, Set};
use crate::traits::len::Len;
// TODO: implement PartialEq + Eq

/// A set specialized for integer values.
#[derive(Clone)]
pub struct LengthSet<T, S = u8> {
    map: LengthMap<T, (), S>,
}

impl<T, S> LengthSet<T, S>
where
    T: Len + Eq,
    S: PrimInt + Unsigned,
{
    #[must_use]
    pub fn from_vec(payload: Vec<T>) -> Self {
        Self {
            map: LengthMap::from_vec(payload.into_iter().map(|x| (x, ())).collect()),
        }
    }
}

impl<T, S> LengthSet<T, S>
where
    S: PrimInt + Unsigned,
{
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Len + Eq,
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
        Q: Len + Eq,
    {
        self.get(value).is_some()
    }
}

impl<T, S> LengthSet<T, S> {
    #[must_use]
    pub const fn iter(&self) -> Iter<T> {
        Iter::new(&self.map.table.entries)
    }
}

impl<T, S> Len for LengthSet<T, S> {
    fn len(&self) -> usize {
        self.map.len()
    }
}

impl<T, S> Debug for LengthSet<T, S>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.map.fmt(f) // TODO: can we do better here?
    }
}

impl<'a, T, S> IntoIterator for &'a LengthSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S, const N: usize> From<[T; N]> for LengthSet<T, S>
where
    T: Len + Eq,
    S: PrimInt + Unsigned,
{
    fn from(payload: [T; N]) -> Self {
        Self::from_vec(Vec::from_iter(payload))
    }
}

impl<T, S> FromIterator<T> for LengthSet<T, S>
where
    T: Len + Eq,
    S: PrimInt + Unsigned,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(Vec::from_iter(iter))
    }
}

impl<T, S> Set<T> for LengthSet<T, S>
where
    T: Len + Eq,
    S: PrimInt + Unsigned,
{
    type Iterator<'a> = Iter<'a, T>
    where
        T: 'a,
        S: 'a;

    fn iter(&self) -> Iter<'_, T> {
        self.iter()
    }

    fn contains(&self, value: &T) -> bool {
        self.contains(value)
    }
}
