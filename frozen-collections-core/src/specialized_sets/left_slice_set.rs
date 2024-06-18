use core::borrow::Borrow;
use core::fmt::{Debug, Formatter, Result};
use core::ops::Range;
use std::hash::{BuildHasher, RandomState};

use num_traits::{PrimInt, Unsigned};

use crate::specialized_maps::LeftSliceMap;
use crate::specialized_sets::{Iter, Set};
use crate::traits::len::Len;
use crate::traits::slice_hash::SliceHash;
// TODO: Implement PartialEq + Eq

/// A set that hashes left-aligned slices of its values.
#[derive(Clone)]
pub struct LeftSliceSet<T, S = u8, BH = RandomState> {
    map: LeftSliceMap<T, (), S, BH>,
}

impl<T, S, BH> LeftSliceSet<T, S, BH>
where
    T: SliceHash + Len + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    #[must_use]
    pub fn from_vec_with_hasher(payload: Vec<T>, range: Range<usize>, bh: BH) -> Self {
        Self {
            map: LeftSliceMap::from_iter_with_hasher(
                payload.into_iter().map(|x| (x, ())),
                range,
                bh,
            ),
        }
    }

    #[must_use]
    pub fn from_iter_with_hasher<I: IntoIterator<Item = T>>(
        iter: I,
        range: Range<usize>,
        bh: BH,
    ) -> Self {
        Self {
            map: LeftSliceMap::from_iter_with_hasher(iter.into_iter().map(|x| (x, ())), range, bh),
        }
    }

    #[must_use]
    pub fn with_hasher<const N: usize>(payload: [T; N], range: Range<usize>, bh: BH) -> Self {
        Self::from_vec_with_hasher(Vec::from_iter(payload), range, bh)
    }
}

impl<T, S, BH> LeftSliceSet<T, S, BH>
where
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: SliceHash + Len + Eq,
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
        Q: SliceHash + Len + Eq,
    {
        self.get(value).is_some()
    }
}

impl<T, S, BH> LeftSliceSet<T, S, BH> {
    #[must_use]
    pub const fn iter(&self) -> Iter<T> {
        Iter::new(&self.map.table.entries)
    }

    #[must_use]
    pub const fn hasher(&self) -> &BH {
        self.map.hasher()
    }
}

impl<T, S> LeftSliceSet<T, S, RandomState>
where
    T: SliceHash + Len + Eq,
    S: PrimInt + Unsigned,
{
    #[must_use]
    pub fn from_vec(payload: Vec<T>, range: Range<usize>) -> Self {
        Self::from_vec_with_hasher(payload, range, RandomState::new())
    }

    pub fn from_iter<I: IntoIterator<Item = T>>(iter: I, range: Range<usize>) -> Self {
        Self::from_iter_with_hasher(iter, range, RandomState::new())
    }

    pub fn from<const N: usize>(payload: [T; N], range: Range<usize>) -> Self {
        Self::with_hasher(payload, range, RandomState::new())
    }
}

impl<T, S, BH> Len for LeftSliceSet<T, S, BH> {
    fn len(&self) -> usize {
        self.map.len()
    }
}

impl<T, S, BH> Debug for LeftSliceSet<T, S, BH>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.map.fmt(f) // TODO: can we do better here?
    }
}

impl<'a, T, S, BH> IntoIterator for &'a LeftSliceSet<T, S, BH> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S, BH> Set<T> for LeftSliceSet<T, S, BH>
where
    T: SliceHash + Len + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    type Iterator<'a> = Iter<'a, T>
    where
        T: 'a,
        S: 'a,
        BH: 'a;

    fn iter(&self) -> Iter<'_, T> {
        self.iter()
    }

    fn contains(&self, value: &T) -> bool {
        self.contains(value)
    }
}
