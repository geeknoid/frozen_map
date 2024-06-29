use std::borrow::Borrow;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter, Result};
use std::hash::{BuildHasher, Hash};
use std::hash::RandomState;
use std::ops::{BitAnd, BitOr, BitXor, Sub};

use num_traits::{PrimInt, Unsigned};

use crate::specialized_maps::CommonMap;
use crate::specialized_sets::{IntoIter, Iter, Set};
use crate::traits::len::Len;

/// A general-purpose optimized read-only set.
///
/// # Capacity Constraints
///
/// The `S` generic argument controls the maximum capacity
/// of the set. A `u8` will allow up to 255 elements, `u16`
/// will allow up to 65,535 elements, and `usize` will allow
/// up to `usize::MAX` elements.
#[derive(Clone)]
pub struct CommonSet<T, S = u8, BH = RandomState> {
    map: CommonMap<T, (), S, BH>,
}

impl<T, S, BH> CommonSet<T, S, BH>
where
    T: Hash,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    /// Creates a new set which will use the given hasher to hash values.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::specialized_sets::CommonSet;
    /// use std::hash::RandomState;
    /// use frozen_collections_core::traits::len::Len;
    ///
    /// let set = CommonSet::<_, u8, _>::from_vec_with_hasher(vec![1, 2, 3], RandomState::new());
    ///
    /// assert_eq!(set.len(), 3);
    /// assert!(set.contains(&1));
    /// ```
    ///
    /// # Panics
    ///
    /// If the payload contains more items than the set's capacity
    /// allows. The capacity is determined by the `S` generic argument.
    #[must_use]
    pub fn from_vec_with_hasher(payload: Vec<T>, bh: BH) -> Self {
        Self {
            map: CommonMap::from_iter_with_hasher(payload.into_iter().map(|x| (x, ())), bh),
        }
    }

    /// Creates a new set which will use the given hasher to hash
    /// keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::specialized_sets::CommonSet;
    /// use frozen_collections_core::traits::len::Len;
    /// use std::hash::RandomState;
    ///
    /// let vec = vec![1, 2, 3];
    /// let set = CommonSet::<_, u8, _>::from_iter_with_hasher(vec.iter(), RandomState::new());
    ///
    /// assert_eq!(set.len(), 3);
    /// assert!(set.contains(&1));
    /// ```
    ///
    /// # Panics
    ///
    /// If the payload contains more items than the set's capacity
    /// allows. The capacity is determined by the `S` generic argument.
    #[must_use]
    pub fn from_iter_with_hasher<I: IntoIterator<Item = T>>(iter: I, bh: BH) -> Self {
        Self {
            map: CommonMap::from_iter_with_hasher(iter.into_iter().map(|x| (x, ())), bh),
        }
    }

    /// Creates a new set which will use the given hasher to hash
    /// keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::specialized_sets::CommonSet;
    /// use frozen_collections_core::traits::len::Len;
    /// use std::hash::RandomState;
    ///
    /// let set = CommonSet::<_, u8, _>::with_hasher([1, 2, 3], RandomState::new());
    ///
    /// assert_eq!(set.len(), 3);
    /// assert!(set.contains(&1));
    /// ```
    ///
    /// # Panics
    ///
    /// If the payload contains more items than the set's capacity
    /// allows. The capacity is determined by the `S` generic argument.
    #[must_use]
    pub fn with_hasher<const N: usize>(payload: [T; N], bh: BH) -> Self {
        Self {
            map: CommonMap::from_iter_with_hasher(payload.into_iter().map(|x| (x, ())), bh),
        }
    }
}

impl<T, S, BH> CommonSet<T, S, BH>
where
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    /// Returns a reference to the value in the set, if any, that is equal to the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::specialized_sets::CommonSet;
    ///
    /// let set = CommonSet::<_, u8, _>::from([1, 2, 3]);
    ///
    /// assert_eq!(set.get(&2), Some(&2));
    /// assert_eq!(set.get(&4), None);
    /// ```
    #[inline]
    #[must_use]
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        Some(self.map.get_key_value(value)?.0)
    }

    /// Returns `true` if the set contains a value.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::specialized_sets::CommonSet;
    ///
    /// let set = CommonSet::<_, u8, _>::from([1, 2, 3]);
    ///
    /// assert!(set.contains(&1));
    /// assert!(!set.contains(&4));
    /// ```
    #[inline]
    #[must_use]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.get(value).is_some()
    }
}

impl<T, S, BH> CommonSet<T, S, BH> {
    /// Returns the hasher for this set.
    #[must_use]
    pub const fn hasher(&self) -> &BH {
        self.map.hasher()
    }
}

impl<T, S> CommonSet<T, S, RandomState>
where
    T: Hash,
    S: PrimInt + Unsigned,
{
    /// Creates a new set using the default hasher to hash values.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::specialized_sets::CommonSet;
    ///
    /// let set = CommonSet::<_, u8, _>::from_vec(vec![1, 2, 3]);
    /// ```
    #[must_use]
    pub fn from_vec(payload: Vec<T>) -> Self {
        Self::from_vec_with_hasher(payload, RandomState::new())
    }
}

impl<T, S, BH> Len for CommonSet<T, S, BH> {
    fn len(&self) -> usize {
        self.map.len()
    }
}

impl<T, S, BH> Debug for CommonSet<T, S, BH>
where
    T: Hash + Eq + Debug,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T, S, BH> IntoIterator for CommonSet<T, S, BH>
where
    T: Hash + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self.map.table.entries)
    }
}

impl<'a, T, S, BH> IntoIterator for &'a CommonSet<T, S, BH>
where
    T: Hash + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T, S, const N: usize> From<[T; N]> for CommonSet<T, S, RandomState>
where
    T: Hash,
    S: PrimInt + Unsigned,
{
    fn from(payload: [T; N]) -> Self {
        Self {
            map: CommonMap::from_iter_with_hasher(
                payload.into_iter().map(|x| (x, ())),
                RandomState::new(),
            ),
        }
    }
}

impl<T, S> FromIterator<T> for CommonSet<T, S, RandomState>
where
    T: Hash,
    S: PrimInt + Unsigned,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            map: CommonMap::from_iter_with_hasher(
                iter.into_iter().map(|x| (x, ())),
                RandomState::new(),
            ),
        }
    }
}

impl<T, S, BH> Set<T> for CommonSet<T, S, BH>
where
    T: Hash + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher,
{
    type Iterator<'a> = Iter<'a, T>
    where
        T: 'a,
        S: 'a,
        BH: 'a;

    fn iter(&self) -> Iter<'_, T> {
        Iter::new(&self.map.table.entries)
    }

    fn contains(&self, value: &T) -> bool {
        self.contains(value)
    }
}

impl<T, S, ST, BH> BitOr<&ST> for &CommonSet<T, S, BH>
where
    T: Hash + Eq + Clone,
    S: PrimInt + Unsigned,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    type Output = HashSet<T, BH>;

    fn bitor(self, rhs: &ST) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}

impl<T, S, ST, BH> BitAnd<&ST> for &CommonSet<T, S, BH>
where
    T: Hash + Eq + Clone,
    S: PrimInt + Unsigned,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    type Output = HashSet<T, BH>;

    fn bitand(self, rhs: &ST) -> Self::Output {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T, S, ST, BH> BitXor<&ST> for &CommonSet<T, S, BH>
where
    T: Hash + Eq + Clone,
    S: PrimInt + Unsigned,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    type Output = HashSet<T, BH>;

    fn bitxor(self, rhs: &ST) -> Self::Output {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T, S, ST, BH> Sub<&ST> for &CommonSet<T, S, BH>
where
    T: Hash + Eq + Clone,
    S: PrimInt + Unsigned,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    type Output = HashSet<T, BH>;

    fn sub(self, rhs: &ST) -> Self::Output {
        self.difference(rhs).cloned().collect()
    }
}

impl<T, S, ST, BH> PartialEq<ST> for CommonSet<T, S, BH>
where
    T: Hash + Eq,
    S: PrimInt + Unsigned,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    fn eq(&self, other: &ST) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|value| other.contains(value))
    }
}

impl<T, S, BH> Eq for CommonSet<T, S, BH>
where
    T: Hash + Eq,
    S: PrimInt + Unsigned,
    BH: BuildHasher + Default,
{
}
