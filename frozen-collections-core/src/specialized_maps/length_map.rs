use core::borrow::Borrow;
use core::fmt::{Debug, Formatter, Result};
use core::intrinsics::transmute;
use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut};
use core::ops::Range;

use bitvec::macros::internal::funty::Fundamental;
use num_traits::{PrimInt, Unsigned};

use crate::analyzers::hash_code_analyzer::analyze_hash_codes;
use crate::specialized_maps::{Iter, Keys, Values};
use crate::specialized_maps::hash_table::HashTable;
use crate::traits::len::Len;

/// A map that uses key lengths as hash codes, in order to avoid hashing overhead.
#[derive(Clone)]
pub struct LengthMap<K, V, S = u8> {
    pub(crate) table: HashTable<K, V, S>,
}

impl<K, V, S> LengthMap<K, V, S>
where
    K: Len + Eq,
    S: PrimInt + Unsigned,
{
    #[must_use]
    pub fn from_vec(payload: Vec<(K, V)>) -> Self {
        let code_analysis = analyze_hash_codes(payload.iter().map(|entry| entry.0.len().as_u64()));

        Self {
            table: HashTable::new(payload, code_analysis.num_hash_slots, |k| k.len() as u64),
        }
    }
}

impl<K, V, S> LengthMap<K, V, S>
where
    S: PrimInt + Unsigned,
{
    #[inline]
    #[must_use]
    fn get_hash_info<Q>(&self, key: &Q) -> Range<usize>
    where
        Q: Len,
    {
        let hash_code = key.len().as_u64();
        self.table.get_hash_info(hash_code)
    }

    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Len + Eq,
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
        Q: Len + Eq,
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
        Q: Len + Eq,
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
        Q: Len + Eq,
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
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Len + Eq,
    {
        self.get(key).is_some()
    }
}

impl<K, V, S> LengthMap<K, V, S> {
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
}

impl<K, V, S> Len for LengthMap<K, V, S> {
    fn len(&self) -> usize {
        self.table.len()
    }
}

impl<K, V, S> Debug for LengthMap<K, V, S>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.table.fmt(f)
    }
}

impl<Q, K, V, S> Index<Q> for LengthMap<K, V, S>
where
    K: Borrow<Q>,
    Q: Len + Eq,
    S: PrimInt + Unsigned,
{
    type Output = V;

    fn index(&self, index: Q) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<Q, K, V, S> IndexMut<Q> for LengthMap<K, V, S>
where
    K: Borrow<Q>,
    Q: Len + Eq,
    S: PrimInt + Unsigned,
{
    fn index_mut(&mut self, index: Q) -> &mut V {
        self.get_mut(&index).unwrap()
    }
}

impl<'a, K, V, S> IntoIterator for &'a LengthMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V, S> PartialEq<Self> for LengthMap<K, V, S>
where
    K: Len + Eq,
    V: PartialEq,
    S: PrimInt + Unsigned,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V, S> Eq for LengthMap<K, V, S>
where
    K: Len + Eq,
    V: Eq,
    S: PrimInt + Unsigned,
{
}

impl<K, V, const N: usize> From<[(K, V); N]> for LengthMap<K, V>
where
    K: Len + Eq,
{
    fn from(payload: [(K, V); N]) -> Self {
        Self::from_vec(Vec::from_iter(payload))
    }
}

impl<K, V> FromIterator<(K, V)> for LengthMap<K, V>
where
    K: Len + Eq,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::from_vec(Vec::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_empty() {
        let pairs: [(String, i32); 0] = [];
        let map = LengthMap::<String, i32>::from(pairs);
        assert!(map.is_empty());
    }

    #[test]
    fn test_from_single() {
        let pairs = [("key1".to_string(), 1)];
        let map = LengthMap::<String, i32>::from(pairs);
        assert_eq!(map.get(&"key1".to_string()), Some(&1));
    }

    #[test]
    fn test_from_multiple() {
        let pairs = [
            ("key1".to_string(), 1),
            ("key2".to_string(), 2),
            ("key3".to_string(), 3),
        ];
        let map = LengthMap::<String, i32>::from(pairs);
        assert_eq!(map.get(&"key1".to_string()), Some(&1));
        assert_eq!(map.get(&"key2".to_string()), Some(&2));
        assert_eq!(map.get(&"key3".to_string()), Some(&3));
    }

    #[test]
    fn test_from_iter_empty() {
        let pairs: Vec<(String, i32)> = vec![];
        let map: LengthMap<String, i32> = pairs.into_iter().collect();
        assert!(map.is_empty());
    }

    #[test]
    fn test_from_iter_single() {
        let pairs = vec![("key1".to_string(), 1)];
        let map: LengthMap<String, i32> = pairs.into_iter().collect();
        assert_eq!(map.get(&"key1".to_string()), Some(&1));
    }

    #[test]
    fn test_from_iter_multiple() {
        let pairs = vec![
            ("key1".to_string(), 1),
            ("key2".to_string(), 2),
            ("key3".to_string(), 3),
        ];
        let map: LengthMap<String, i32> = pairs.into_iter().collect();
        assert_eq!(map.get(&"key1".to_string()), Some(&1));
        assert_eq!(map.get(&"key2".to_string()), Some(&2));
        assert_eq!(map.get(&"key3".to_string()), Some(&3));
    }
}
