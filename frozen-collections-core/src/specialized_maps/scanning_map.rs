use core::borrow::Borrow;
use core::fmt::{Debug, Formatter, Result};
use core::intrinsics::transmute;
use core::mem::MaybeUninit;
use core::ops::{Index, IndexMut};

use crate::specialized_maps::{Iter, Keys, Values};
use crate::traits::len::Len;

/// A map that does a linear scan of its entries upon lookup, designed for very small payloads.
#[derive(Clone)]
pub struct ScanningMap<K, V> {
    pub(crate) entries: Box<[(K, V)]>,
}

impl<K, V> ScanningMap<K, V>
where
    K: Eq,
{
    #[must_use]
    pub fn from_vec(payload: Vec<(K, V)>) -> Self {
        Self {
            entries: payload.into_boxed_slice(),
        }
    }
}

impl<K, V> ScanningMap<K, V> {
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        for entry in self.entries.iter() {
            if key.eq(entry.0.borrow()) {
                return Some(&entry.1);
            }
        }

        None
    }

    #[inline]
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        for entry in self.entries.iter_mut() {
            if key.eq(entry.0.borrow()) {
                return Some(&mut entry.1);
            }
        }

        None
    }

    #[inline]
    #[must_use]
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Eq,
    {
        for entry in self.entries.iter() {
            if key.eq(entry.0.borrow()) {
                return Some((&entry.0, &entry.1));
            }
        }

        None
    }

    #[allow(mutable_transmutes)]
    pub fn get_many_mut<Q, const N: usize>(&mut self, keys: [&Q; N]) -> Option<[&mut V; N]>
    where
        K: Borrow<Q>,
        Q: Eq,
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
        Q: Eq,
    {
        self.get(key).is_some()
    }

    #[must_use]
    pub const fn iter(&self) -> Iter<K, V> {
        Iter::new(&self.entries)
    }

    #[must_use]
    pub const fn keys(&self) -> Keys<K, V> {
        Keys::new(&self.entries)
    }

    #[must_use]
    pub const fn values(&self) -> Values<K, V> {
        Values::new(&self.entries)
    }
}

impl<K, V> Len for ScanningMap<K, V> {
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<K, V> Debug for ScanningMap<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let pairs = self.entries.iter().map(|x| (&x.0, &x.1));
        f.debug_map().entries(pairs).finish()
    }
}

impl<Q, K, V> Index<Q> for ScanningMap<K, V>
where
    K: Borrow<Q>,
    Q: Eq,
{
    type Output = V;

    fn index(&self, index: Q) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<Q, K, V> IndexMut<Q> for ScanningMap<K, V>
where
    K: Borrow<Q>,
    Q: Eq,
{
    fn index_mut(&mut self, index: Q) -> &mut V {
        self.get_mut(&index).unwrap()
    }
}

impl<'a, K, V> IntoIterator for &'a ScanningMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V> PartialEq<Self> for ScanningMap<K, V>
where
    K: Eq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter()
            .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<K, V> Eq for ScanningMap<K, V>
where
    K: Eq,
    V: Eq,
{
}

impl<K, V, const N: usize> From<[(K, V); N]> for ScanningMap<K, V>
where
    K: Eq,
{
    fn from(payload: [(K, V); N]) -> Self {
        Self::from_vec(Vec::from_iter(payload))
    }
}

impl<K, V> FromIterator<(K, V)> for ScanningMap<K, V>
where
    K: Eq,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::from_vec(Vec::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::len::Len;

    use super::ScanningMap;

    #[test]
    fn new_creates_scanning_map_with_given_payload() {
        let payload = vec![(10, 20), (30, 40), (50, 60)];
        let map = ScanningMap::<i32, i32>::from_vec(payload.clone());
        assert_eq!(payload.len(), map.len());
    }

    #[test]
    fn get_returns_some_for_existing_keys() {
        let payload = vec![(10, 20), (30, 40), (50, 60)];
        let map = ScanningMap::<i32, i32>::from_vec(payload);
        assert_eq!(&20, map.get(&10).unwrap());
        assert_eq!(&40, map.get(&30).unwrap());
        assert_eq!(&60, map.get(&50).unwrap());
    }

    #[test]
    fn get_returns_none_for_non_existing_keys() {
        let payload = vec![(10, 20), (30, 40), (50, 60)];
        let map = ScanningMap::<i32, i32>::from_vec(payload);
        assert_eq!(None, map.get(&0));
    }

    #[test]
    fn get_mut_returns_some_for_existing_keys() {
        let payload = vec![(10, 20), (30, 40), (50, 60)];
        let mut map = ScanningMap::<i32, i32>::from_vec(payload);
        assert_eq!(&20, map.get_mut(&10).unwrap());
        assert_eq!(&40, map.get_mut(&30).unwrap());
        assert_eq!(&60, map.get_mut(&50).unwrap());
    }

    #[test]
    fn get_mut_returns_none_for_non_existing_keys() {
        let payload = vec![(10, 20), (30, 40), (50, 60)];
        let mut map = ScanningMap::<i32, i32>::from_vec(payload);
        assert_eq!(None, map.get_mut(&0));
    }

    #[test]
    fn get_key_value_returns_some_for_existing_keys() {
        let payload = vec![(10, 20), (30, 40), (50, 60)];
        let map = ScanningMap::<i32, i32>::from_vec(payload);
        assert_eq!((&10, &20), map.get_key_value(&10).unwrap());
        assert_eq!((&30, &40), map.get_key_value(&30).unwrap());
        assert_eq!((&50, &60), map.get_key_value(&50).unwrap());
    }

    #[test]
    fn get_key_value_returns_none_for_non_existing_keys() {
        let payload = vec![(10, 20), (30, 40), (50, 60)];
        let map = ScanningMap::<i32, i32>::from_vec(payload);
        assert_eq!(None, map.get_key_value(&0));
    }

    #[test]
    fn debug_format_is_correct() {
        let payload = vec![(10, 20)];
        let map = ScanningMap::<i32, i32>::from_vec(payload);
        assert_eq!("{10: 20}", format!("{map:?}"));
    }
}
