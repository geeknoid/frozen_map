use core::borrow::Borrow;
use core::fmt::{Debug, Formatter, Result};
use core::hash::BuildHasher;
use core::intrinsics::transmute;
use core::mem::MaybeUninit;
use core::ops::Range;
use core::ops::{Index, IndexMut};
use std::hash::RandomState;

use num_traits::{PrimInt, Unsigned};

use crate::analyzers::hash_code_analyzer::analyze_hash_codes;
use crate::specialized_maps::hash_table::HashTable;
use crate::specialized_maps::{Iter, Keys, Values};
use crate::traits::len::Len;
use crate::traits::slice_hash::SliceHash;

/// A map that hashes right-aligned slices of its keys.
#[derive(Clone)]
pub struct RightSliceMap<K, V, S = u8, BH = RandomState> {
    pub(crate) table: HashTable<K, V, S>,
    bh: BH,
    range: Range<usize>,
}

impl<K, V, S, BH> RightSliceMap<K, V, S, BH>
where
    K: SliceHash + Len + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    #[must_use]
    pub fn from_vec_with_hasher(payload: Vec<(K, V)>, range: Range<usize>, bh: BH) -> Self {
        let codes = payload.iter().map(|entry| {
            let key = &entry.0;
            if key.len() >= range.end {
                key.hash(&bh, key.len() - range.start..key.len() - range.end)
            } else {
                0
            }
        });

        let code_analysis = analyze_hash_codes(codes);
        Self {
            table: HashTable::new(payload.into_iter(), code_analysis.num_hash_slots, |k| {
                k.hash(&bh, k.len() - range.start..k.len() - range.end)
            }),
            bh,
            range,
        }
    }

    #[must_use]
    pub fn from_iter_with_hasher<T: IntoIterator<Item = (K, V)>>(
        iter: T,
        range: Range<usize>,
        bh: BH,
    ) -> Self {
        Self::from_vec_with_hasher(Vec::from_iter(iter), range, bh)
    }

    #[must_use]
    pub fn with_hasher<const N: usize>(payload: [(K, V); N], range: Range<usize>, bh: BH) -> Self {
        Self::from_vec_with_hasher(Vec::from_iter(payload), range, bh)
    }
}

impl<K, V, S, BH> RightSliceMap<K, V, S, BH>
where
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    #[inline]
    #[must_use]
    fn get_hash_info<Q>(&self, key: &Q) -> Range<usize>
    where
        Q: SliceHash + Len,
    {
        let hash_code = if key.len() >= self.range.start {
            key.hash(
                &self.bh,
                key.len() - self.range.start..key.len() - self.range.end,
            )
        } else {
            0
        };

        self.table.get_hash_info(hash_code)
    }

    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: SliceHash + Len + Eq,
    {
        let range = self.get_hash_info(key);
        let entries = unsafe { self.table.entries.get_unchecked(range) };
        for entry in entries {
            if key.eq(entry.0.borrow()) {
                return Some(&entry.1);
            }
        }

        None
    }

    #[inline]
    #[must_use]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: SliceHash + Len + Eq,
    {
        let range = self.get_hash_info(key);
        let entries = unsafe { self.table.entries.get_unchecked(range) };
        for entry in entries {
            if key.eq(entry.0.borrow()) {
                return Some((&entry.0, &entry.1));
            }
        }

        None
    }

    #[inline]
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: SliceHash + Len + Eq,
    {
        let range = self.get_hash_info(key);
        let entries = unsafe { self.table.entries.get_unchecked_mut(range) };
        for entry in entries {
            if key.eq(entry.0.borrow()) {
                return Some(&mut entry.1);
            }
        }

        None
    }

    #[allow(mutable_transmutes)]
    pub fn get_many_mut<Q, const N: usize>(&mut self, keys: [&Q; N]) -> Option<[&mut V; N]>
    where
        K: Borrow<Q>,
        Q: SliceHash + Len + Eq,
    {
        // ensure key uniqueness (assumes "keys" is a relatively small array)
        for i in 0..keys.len() {
            for j in 0..i {
                if keys[j].eq(keys[i]) {
                    return None;
                }
            }
        }

        unsafe {
            let mut result: MaybeUninit<[&mut V; N]> = MaybeUninit::uninit();
            let p = result.as_mut_ptr();

            for (i, key) in keys.iter().enumerate() {
                *(*p).get_unchecked_mut(i) = transmute(self.get(key)?);
            }

            Some(result.assume_init())
        }
    }

    #[inline]
    #[must_use]
    pub const fn get_by_index(&self, index: usize) -> Option<(&K, &V)> {
        self.table.get_by_index(index)
    }

    #[inline]
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: SliceHash + Len + Eq,
    {
        self.get(key).is_some()
    }
}

impl<K, V, S, BH> RightSliceMap<K, V, S, BH> {
    #[must_use]
    pub const fn iter(&self) -> Iter<K, V> {
        Iter::new(&self.table.entries)
    }

    #[must_use]
    pub const fn keys(&self) -> Keys<K, V> {
        Keys::new(&self.table.entries)
    }

    #[must_use]
    pub const fn values(&self) -> Values<K, V> {
        Values::new(&self.table.entries)
    }

    #[must_use]
    pub const fn hasher(&self) -> &BH {
        &self.bh
    }
}

impl<K, V, S> RightSliceMap<K, V, S, RandomState>
where
    K: SliceHash + Len + Eq,
    S: PrimInt + Unsigned,
{
    #[must_use]
    pub fn from_vec(payload: Vec<(K, V)>, range: Range<usize>) -> Self {
        Self::from_vec_with_hasher(payload, range, RandomState::new())
    }

    #[must_use]
    pub fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T, range: Range<usize>) -> Self {
        Self::from_iter_with_hasher(iter, range, RandomState::new())
    }

    #[must_use]
    pub fn from<const N: usize>(payload: [(K, V); N], range: Range<usize>) -> Self {
        Self::with_hasher(payload, range, RandomState::new())
    }
}

impl<K, V, S, BH> Len for RightSliceMap<K, V, S, BH> {
    fn len(&self) -> usize {
        self.table.len()
    }
}

impl<K, V, S, BH> Debug for RightSliceMap<K, V, S, BH>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.table.fmt(f)
    }
}

impl<Q, K, V, S, BH> Index<Q> for RightSliceMap<K, V, S, BH>
where
    K: Borrow<Q>,
    Q: SliceHash + Len + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    type Output = V;

    fn index(&self, index: Q) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<Q, K, V, S, BH> IndexMut<Q> for RightSliceMap<K, V, S, BH>
where
    K: Borrow<Q>,
    Q: SliceHash + Len + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    fn index_mut(&mut self, index: Q) -> &mut V {
        self.get_mut(&index).unwrap()
    }
}

impl<'a, K, V, S, BH> IntoIterator for &'a RightSliceMap<K, V, S, BH> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V, S, BH> PartialEq<Self> for RightSliceMap<K, V, S, BH>
where
    K: SliceHash + Len + Eq,
    V: PartialEq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, S, BH> Eq for RightSliceMap<K, V, S, BH>
where
    K: SliceHash + Len + Eq,
    V: Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
}
