use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result};
use std::hash::{Hash, RandomState};
use std::ops::{BitAnd, BitOr, BitXor, Sub};

use num_traits::{PrimInt, Unsigned};

use crate::specialized_maps::LengthMap;
use crate::specialized_sets::{IntoIter, Iter, Set};
use crate::traits::len::Len;

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

impl<T, S> IntoIterator for LengthSet<T, S> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.map.table.entries)
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

impl<T, S, ST> BitOr<&ST> for &LengthSet<T, S>
where
    T: Hash + Eq + Len + Clone,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    type Output = HashSet<T, RandomState>;

    fn bitor(self, rhs: &ST) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}

impl<T, S, ST> BitAnd<&ST> for &LengthSet<T, S>
where
    T: Hash + Eq + Len + Clone,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    type Output = HashSet<T, RandomState>;

    fn bitand(self, rhs: &ST) -> Self::Output {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T, S, ST> BitXor<&ST> for &LengthSet<T, S>
where
    T: Hash + Eq + Len + Clone,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    type Output = HashSet<T, RandomState>;

    fn bitxor(self, rhs: &ST) -> Self::Output {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T, S, ST> Sub<&ST> for &LengthSet<T, S>
where
    T: Hash + Eq + Len + Clone,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    type Output = HashSet<T, RandomState>;

    fn sub(self, rhs: &ST) -> Self::Output {
        self.difference(rhs).cloned().collect()
    }
}

impl<T, S, ST> PartialEq<ST> for LengthSet<T, S>
where
    T: Hash + Eq + Len,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    fn eq(&self, other: &ST) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|value| other.contains(value))
    }
}

impl<T, S> Eq for LengthSet<T, S>
where
    T: Hash + Eq + Len,
    S: PrimInt + Unsigned,
{
}
