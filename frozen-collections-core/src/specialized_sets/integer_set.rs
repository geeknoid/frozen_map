use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result};
use std::hash::Hash;
use std::ops::{BitAnd, BitOr, BitXor, Sub};

use num_traits::{AsPrimitive, PrimInt, Unsigned};

use crate::specialized_maps::IntegerMap;
use crate::specialized_sets::{Iter, Set};
use crate::traits::len::Len;

// TODO: implement PartialEq + Eq

/// A set specialized for integer values.
#[derive(Clone)]
pub struct IntegerSet<T, S = u8> {
    map: IntegerMap<T, (), S>,
}

impl<T, S> IntegerSet<T, S>
where
    T: PrimInt + AsPrimitive<u64>,
    S: PrimInt + Unsigned,
{
    #[must_use]
    pub fn from_vec(payload: Vec<T>) -> Self {
        Self {
            map: payload.into_iter().map(|x| (x, ())).collect(),
        }
    }

    #[inline]
    #[must_use]
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: PrimInt + AsPrimitive<u64>,
    {
        Some(self.map.get_key_value(value)?.0)
    }

    #[inline]
    #[must_use]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: PrimInt + AsPrimitive<u64>,
    {
        self.get(value).is_some()
    }
}

impl<T, S> IntegerSet<T, S> {
    #[must_use]
    pub const fn iter(&self) -> Iter<T> {
        Iter::new(&self.map.table.entries)
    }
}

impl<T, S> Len for IntegerSet<T, S> {
    fn len(&self) -> usize {
        self.map.len()
    }
}

impl<T, S> Debug for IntegerSet<T, S>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.map.fmt(f) // TODO: can we do better here?
    }
}

impl<'a, T, S> IntoIterator for &'a IntegerSet<T, S> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S, const N: usize> From<[T; N]> for IntegerSet<T, S>
where
    T: PrimInt + AsPrimitive<u64>,
    S: PrimInt + Unsigned,
{
    fn from(payload: [T; N]) -> Self {
        Self::from_vec(Vec::from_iter(payload))
    }
}

impl<T, S> FromIterator<T> for IntegerSet<T, S>
where
    T: PrimInt + AsPrimitive<u64>,
    S: PrimInt + Unsigned,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_vec(Vec::from_iter(iter))
    }
}

impl<T, S> Set<T> for IntegerSet<T, S>
where
    T: PrimInt + AsPrimitive<u64>,
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

impl<T, S, ST> BitOr<&ST> for &IntegerSet<T, S>
where
    T: PrimInt + AsPrimitive<u64> + Clone + Hash,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    type Output = HashSet<T>;

    fn bitor(self, rhs: &ST) -> Self::Output {
        self.union(rhs).copied().collect()
    }
}

impl<T, S, ST> BitAnd<&ST> for &IntegerSet<T, S>
where
    T: PrimInt + AsPrimitive<u64> + Clone + Hash,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    type Output = HashSet<T>;

    fn bitand(self, rhs: &ST) -> Self::Output {
        self.intersection(rhs).copied().collect()
    }
}

impl<T, S, ST> BitXor<&ST> for &IntegerSet<T, S>
where
    T: PrimInt + AsPrimitive<u64> + Clone + Hash,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    type Output = HashSet<T>;

    fn bitxor(self, rhs: &ST) -> Self::Output {
        self.symmetric_difference(rhs).copied().collect()
    }
}

impl<T, S, ST> Sub<&ST> for &IntegerSet<T, S>
where
    T: PrimInt + AsPrimitive<u64> + Clone + Hash,
    S: PrimInt + Unsigned,
    ST: Set<T>,
{
    type Output = HashSet<T>;

    fn sub(self, rhs: &ST) -> Self::Output {
        self.difference(rhs).copied().collect()
    }
}
