use core::borrow::Borrow;
use core::fmt::{Debug, Formatter, Result};
use core::ops::{Index, IndexMut};
use std::intrinsics::transmute;
use std::mem::MaybeUninit;

use num_traits::PrimInt;

use crate::specialized_maps::{Iter, Keys, Values};
use crate::traits::len::Len;

/// A map whose keys are a continuous range of integers.
#[derive(Clone)]
pub struct IntegerRangeMap<K, V> {
    min: K,
    max: K,
    pub(crate) entries: Box<[(K, V)]>,
}

impl<K, V> IntegerRangeMap<K, V>
where
    K: PrimInt,
{
    /// # Panics
    ///
    /// Panics if the keys aren't all in a contiguous range.
    #[must_use]
    pub fn from_vec(mut payload: Vec<(K, V)>) -> Self {
        if payload.is_empty() {
            return Self {
                min: K::zero(),
                max: K::zero(),
                entries: Box::new([]),
            };
        }

        payload.sort_by_key(|x| x.0);

        let min = payload[0].0;
        let max = payload[payload.len() - 1].0;

        assert_eq!(
            max.sub(min).to_usize().unwrap(),
            payload.len() - 1,
            "IntegerRangeMap requires that the map keys be in a continuous range"
        );

        Self {
            min,
            max,
            entries: payload.into_boxed_slice(),
        }
    }
}

impl<K, V> IntegerRangeMap<K, V> {
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: PrimInt,
    {
        if *key >= *self.min.borrow() && *key <= *self.max.borrow() {
            let index = (*key - *self.min.borrow()).to_usize()?;
            Some(&self.entries[index].1)
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: PrimInt,
    {
        if *key >= *self.min.borrow() && *key <= *self.max.borrow() {
            let index = (*key - *self.min.borrow()).to_usize()?;
            Some(&mut self.entries[index].1)
        } else {
            None
        }
    }

    #[allow(mutable_transmutes)]
    pub fn get_many_mut<Q, const N: usize>(&mut self, keys: [&Q; N]) -> Option<[&mut V; N]>
    where
        K: Borrow<Q>,
        Q: PrimInt,
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
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: PrimInt,
    {
        if *key >= *self.min.borrow() && *key <= *self.max.borrow() {
            let index = (*key - *self.min.borrow()).to_usize()?;
            Some((&self.entries[index].0, &self.entries[index].1))
        } else {
            None
        }
    }

    #[inline]
    #[must_use]
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: PrimInt,
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

impl<K, V> Len for IntegerRangeMap<K, V> {
    fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<K, V> Debug for IntegerRangeMap<K, V>
where
    K: Debug,
    V: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let pairs = self.entries.iter().map(|x| (&x.0, &x.1));
        f.debug_map().entries(pairs).finish()
    }
}

impl<Q, K, V> Index<Q> for IntegerRangeMap<K, V>
where
    K: Borrow<Q>,
    Q: PrimInt,
{
    type Output = V;

    fn index(&self, index: Q) -> &Self::Output {
        self.get(&index).unwrap()
    }
}

impl<Q, K, V> IndexMut<Q> for IntegerRangeMap<K, V>
where
    K: Borrow<Q>,
    Q: PrimInt,
{
    fn index_mut(&mut self, index: Q) -> &mut V {
        self.get_mut(&index).unwrap()
    }
}

impl<'a, K, V> IntoIterator for &'a IntegerRangeMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<K, V> PartialEq<Self> for IntegerRangeMap<K, V>
where
    K: PrimInt,
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

impl<K, V> Eq for IntegerRangeMap<K, V>
where
    K: PrimInt,
    V: Eq,
{
}

impl<K, V, const N: usize> From<[(K, V); N]> for IntegerRangeMap<K, V>
where
    K: PrimInt,
{
    fn from(payload: [(K, V); N]) -> Self {
        Self::from_vec(Vec::from_iter(payload))
    }
}

impl<K, V> FromIterator<(K, V)> for IntegerRangeMap<K, V>
where
    K: PrimInt,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::from_vec(Vec::from_iter(iter))
    }
}

#[cfg(test)]
mod test {
    use crate::traits::len::Len;

    use super::IntegerRangeMap;

    #[test]
    fn range_map_test() {
        const MIN: [i32; 5] = [-11, -10, -9, 0, 1];

        for min in MIN {
            let mut v = Vec::new();
            for i in 0..10 {
                v.push((min + i, i));
            }

            let mut m = IntegerRangeMap::<i32, i32>::from_vec(v);

            assert_eq!(10, m.len());
            for i in 0..9 {
                let index = min + i;
                assert_eq!(i, *m.get(&index).unwrap());
                assert_eq!(i, *m.get_mut(&index).unwrap());

                let (k, v) = m.get_key_value(&index).unwrap();
                assert_eq!((index, i), (*k, *v));
            }

            let below = min - 1;
            assert_eq!(None, m.get(&below));
            assert_eq!(None, m.get_mut(&below));
            assert_eq!(None, m.get_key_value(&below));

            let above = min + 10;
            assert_eq!(None, m.get(&above));
            assert_eq!(None, m.get_mut(&above));
            assert_eq!(None, m.get_key_value(&above));

            if min == -11 {
                assert_eq!(
                    "{-11: 0, -10: 1, -9: 2, -8: 3, -7: 4, -6: 5, -5: 6, -4: 7, -3: 8, -2: 9}",
                    format!("{m:?}")
                );
            }
        }
    }
}
