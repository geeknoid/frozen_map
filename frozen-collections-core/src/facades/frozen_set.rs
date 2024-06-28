use core::fmt::{Debug, Formatter, Result};
use core::hash::{BuildHasher, Hash};
use core::ops::{BitAnd, BitOr, BitXor, Sub};
use std::any::type_name;
use std::collections::HashSet;
use std::hash::RandomState;
use std::intrinsics::transmute;

use bitvec::macros::internal::funty::Fundamental;

use crate::analyzers::int_key_analyzer::{analyze_int_keys, IntKeyAnalysisResult};
use crate::analyzers::slice_key_analyzer::{analyze_slice_keys, SliceKeyAnalysisResult};
use crate::specialized_sets::{
    CommonSet, IntegerRangeSet, IntegerSet, Iter, LeftSliceSet, LengthSet, RightSliceSet,
    ScanningSet, Set,
};
use crate::traits::len::Len;

/// The different implementations available for use, depending on the type and content of the payload.
#[derive(Clone)]
enum SetTypes<T, BH> {
    Scanning(ScanningSet<T>),

    CommonSmall(CommonSet<T, u8, BH>),
    CommonLarge(CommonSet<T, usize, BH>),

    U32Small(IntegerSet<u32, u8>),
    U32Large(IntegerSet<u32, usize>),

    U32Range(IntegerRangeSet<u32>),

    LeftStringSliceSmall(LeftSliceSet<String, u8, BH>),
    LeftStringSliceLarge(LeftSliceSet<String, usize, BH>),

    RightStringSliceSmall(RightSliceSet<String, u8, BH>),
    RightStringSliceLarge(RightSliceSet<String, usize, BH>),

    StringLengthSmall(LengthSet<String, u8>),
}

/// A set optimized for fast read access.
///
/// A frozen set differs from the traditional [`HashSet`] type in three key ways. First, creating
/// a mew frozen set can take a relatively long time, especially for very large sets. Second,
/// once created, instances of frozen sets are immutable. And third, probing a frozen set is
/// typically considerably faster.
///
/// The reason creating a frozen set can take some time is due to the extensive analyzers that is
/// performed on the set's values in order to determine the best set implementation to use, and
/// the best data layout to use. These analyzers are what enables frozen sets to be faster later when
/// probing the set.
///
/// Frozen sets are intended for long-lived sets, where the cost of creating the set is made up
/// over time by the faster probing performance.
///
/// A `FrozenSet` requires that the elements
/// implement the [`Eq`] and [`Hash`] traits. This can frequently be achieved by
/// using `#[derive(PartialEq, Eq, Hash)]`. If you implement these yourself,
/// it is important that the following property holds:
///
/// ```text
/// k1 == k2 -> hash(k1) == hash(k2)
/// ```
///
/// In other words, if two keys are equal, their hashes must be equal.
/// Violating this property is a logic error.
///
/// It is also a logic error for a key to be modified in such a way that the key's
/// hash, as determined by the [`Hash`] trait, or its equality, as determined by
/// the [`Eq`] trait, changes while it is in the set. This is normally only
/// possible through [`Cell`], [`RefCell`], global state, I/O, or unsafe code.
///
/// The behavior resulting from either logic error is not specified, but will
/// be encapsulated to the `FrozenSet` that observed the logic error and not
/// result in undefined behavior. This could include panics, incorrect results,
/// aborts, memory leaks, and non-termination.
///
/// # Examples
///
/// ```
/// use std::hash::RandomState;
/// use frozen_collections_core::facades::FrozenSet;
/// use frozen_collections_core::traits::len::Len;
///
/// let books = FrozenSet::from_vec(vec!(
///     "A Dance With Dragons".to_string(),
///     "To Kill a Mockingbird".to_string(),
///     "The Odyssey".to_string(),
///     "The Great Gatsby".to_string()));
///
/// // Check for a specific one.
/// if !books.contains(&"The Winds of Winter".to_string()) {
///     println!("We have {} books, but The Winds of Winter ain't one.",
///              books.len());
/// }
///
/// // Iterate over everything.
/// for book in &books {
///     println!("{book}");
/// }
/// ```
///
/// The easiest way to use `FrozenSet` with a custom type is to derive
/// [`Eq`] and [`Hash`]. We must also derive [`PartialEq`],
/// which is required if [`Eq`] is derived.
///
/// ```
/// use frozen_collections_core::facades::FrozenSet;
///
/// #[derive(Hash, Eq, PartialEq, Debug)]
/// struct Viking {
///     name: String,
///     power: usize,
/// }
///
/// let vikings = FrozenSet::from([
///     Viking {name: "Einar".to_string(), power: 9 },
///     Viking { name: "Olaf".to_string(), power: 4 },
///     Viking { name: "Harald".to_string(), power: 8 }]);
///
/// // Use derived implementation to print the vikings.
/// for x in &vikings {
///     println!("{x:?}");
/// }
/// ```
///
/// [`HashSet`]: HashSet
/// [`HashMap`]: std::collections::HashMap
/// [`RefCell`]: std::cell::RefCell
/// [`Cell`]: std::cell::Cell
#[derive(Clone)]
#[allow(clippy::module_name_repetitions)]
pub struct FrozenSet<T, BH = RandomState> {
    set_impl: SetTypes<T, BH>,
}

impl<T, BH> FrozenSet<T, BH>
where
    T: Hash + Eq,
    BH: BuildHasher,
{
    /// Creates a new frozen set which will use the given hasher to hash values.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::facades::FrozenSet;
    /// use std::hash::RandomState;
    /// use frozen_collections_core::traits::len::Len;
    ///
    /// let set = FrozenSet::from_vec_with_hasher(vec![1, 2, 3], RandomState::new());
    ///
    /// assert_eq!(set.len(), 3);
    /// assert!(set.contains(&1));
    /// ```
    #[must_use]
    pub fn from_vec_with_hasher(payload: Vec<T>, bh: BH) -> Self {
        Self::new(payload, bh)
    }

    /// Creates a new frozen set which will use the given hasher to hash
    /// keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::facades::FrozenSet;
    /// use std::hash::RandomState;
    /// use frozen_collections_core::traits::len::Len;
    ///
    /// let vec = vec![1, 2, 3];
    /// let set = FrozenSet::from_iter_with_hasher(vec, RandomState::new());
    ///
    /// assert_eq!(set.len(), 3);
    /// assert!(set.contains(&1));
    /// ```
    #[must_use]
    pub fn from_iter_with_hasher<U: IntoIterator<Item = T>>(iter: U, bh: BH) -> Self {
        Self::new(Vec::from_iter(iter), bh)
    }

    /// Creates a new frozen set which will use the given hasher to hash
    /// keys.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::facades::FrozenSet;
    /// use std::hash::RandomState;
    ///
    /// let set = FrozenSet::with_hasher([1, 2, 3], RandomState::new());
    /// ```
    #[must_use]
    pub fn with_hasher<const N: usize>(payload: [T; N], bh: BH) -> Self {
        Self::new(Vec::from_iter(payload), bh)
    }

    fn new(payload: Vec<T>, bh: BH) -> Self {
        Self {
            set_impl: if payload.len() < 4 {
                SetTypes::Scanning(ScanningSet::from_vec(payload))
            } else if type_name::<T>() == type_name::<u32>() {
                Self::new_u32_set(payload)
            } else if type_name::<T>() == type_name::<String>() {
                Self::new_string_set(payload, bh)
            } else {
                Self::new_common_set(payload, bh)
            },
        }
    }

    #[allow(clippy::transmute_undefined_repr)]
    fn new_u32_set(payload: Vec<T>) -> SetTypes<T, BH> {
        let payload: Vec<u32> = unsafe { transmute(payload) };

        let key_analysis = analyze_int_keys(payload.iter().copied());

        match key_analysis {
            IntKeyAnalysisResult::Range => SetTypes::U32Range(IntegerRangeSet::from_vec(payload)),
            IntKeyAnalysisResult::Normal => {
                if payload.len() <= u8::MAX.as_usize() {
                    SetTypes::U32Small(IntegerSet::from_vec(payload))
                } else {
                    SetTypes::U32Large(IntegerSet::from_vec(payload))
                }
            }
        }
    }

    #[allow(clippy::transmute_undefined_repr)]
    fn new_string_set(payload: Vec<T>, bh: BH) -> SetTypes<T, BH> {
        let payload: Vec<String> = unsafe { transmute(payload) };

        let key_analysis = analyze_slice_keys(payload.iter().map(String::as_bytes), &bh);

        if payload.len() <= u8::MAX.as_usize() {
            match key_analysis {
                SliceKeyAnalysisResult::Normal => SetTypes::CommonSmall(
                    CommonSet::from_vec_with_hasher(unsafe { transmute(payload) }, bh),
                ),

                SliceKeyAnalysisResult::LeftHandSubslice {
                    subslice_index,
                    subslice_len,
                } => SetTypes::LeftStringSliceSmall(LeftSliceSet::from_vec_with_hasher(
                    payload,
                    subslice_index..subslice_index + subslice_len,
                    bh,
                )),

                SliceKeyAnalysisResult::RightHandSubslice {
                    subslice_index,
                    subslice_len,
                } => SetTypes::RightStringSliceSmall(RightSliceSet::from_vec_with_hasher(
                    payload,
                    subslice_index..subslice_index + subslice_len,
                    bh,
                )),

                SliceKeyAnalysisResult::Length => {
                    SetTypes::StringLengthSmall(LengthSet::from_vec(payload))
                }
            }
        } else {
            match key_analysis {
                SliceKeyAnalysisResult::Length | SliceKeyAnalysisResult::Normal => {
                    SetTypes::CommonLarge(CommonSet::from_vec_with_hasher(
                        unsafe { transmute(payload) },
                        bh,
                    ))
                }

                SliceKeyAnalysisResult::LeftHandSubslice {
                    subslice_index,
                    subslice_len,
                } => SetTypes::LeftStringSliceLarge(LeftSliceSet::from_vec_with_hasher(
                    payload,
                    subslice_index..subslice_index + subslice_len,
                    bh,
                )),

                SliceKeyAnalysisResult::RightHandSubslice {
                    subslice_index,
                    subslice_len,
                } => SetTypes::RightStringSliceLarge(RightSliceSet::from_vec_with_hasher(
                    payload,
                    subslice_index..subslice_index + subslice_len,
                    bh,
                )),
            }
        }
    }

    fn new_common_set(payload: Vec<T>, bh: BH) -> SetTypes<T, BH> {
        if payload.len() <= u8::MAX.as_usize() {
            SetTypes::CommonSmall(CommonSet::from_vec_with_hasher(payload, bh))
        } else {
            SetTypes::CommonLarge(CommonSet::from_vec_with_hasher(payload, bh))
        }
    }

    /// Returns `true` if the set contains a value.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::facades::FrozenSet;
    ///
    /// let set = FrozenSet::from([1, 2, 3]);
    /// assert!(set.contains(&1));
    /// assert!(!set.contains(&4));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        match &self.set_impl {
            SetTypes::Scanning(s) => s.contains(value),
            SetTypes::CommonSmall(s) => s.contains(value),
            SetTypes::CommonLarge(s) => s.contains(value),
            SetTypes::U32Small(s) => s.contains(unsafe { transmute(value) }),
            SetTypes::U32Large(s) => s.contains(unsafe { transmute(value) }),
            SetTypes::U32Range(s) => s.contains(unsafe { transmute(value) }),
            SetTypes::LeftStringSliceSmall(s) => {
                let v: &String = unsafe { transmute(value) };
                s.contains(v)
            }
            SetTypes::LeftStringSliceLarge(s) => {
                let v: &String = unsafe { transmute(value) };
                s.contains(v)
            }
            SetTypes::RightStringSliceSmall(s) => {
                let v: &String = unsafe { transmute(value) };
                s.contains(v)
            }
            SetTypes::RightStringSliceLarge(s) => {
                let v: &String = unsafe { transmute(value) };
                s.contains(v)
            }
            SetTypes::StringLengthSmall(s) => {
                let v: &String = unsafe { transmute(value) };
                s.contains(v)
            }
        }
    }

    /// Returns `true` if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::facades::FrozenSet;
    ///
    /// let x = FrozenSet::from([1, 2, 3]);
    /// assert!(!x.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// An iterator visiting all elements in arbitrary order.
    /// The iterator element type is `&'a T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::facades::FrozenSet;
    ///
    /// let set = FrozenSet::from(["a".to_string(), "b".to_string()]);
    ///
    /// // Will print in an arbitrary order.
    /// for x in set.iter() {
    ///     println!("{x}");
    /// }
    /// ```
    pub const fn iter(&self) -> Iter<T> {
        match &self.set_impl {
            SetTypes::Scanning(s) => s.iter(),
            SetTypes::CommonSmall(s) => s.iter(),
            SetTypes::CommonLarge(s) => s.iter(),
            SetTypes::U32Small(s) => unsafe { transmute(s.iter()) },
            SetTypes::U32Large(s) => unsafe { transmute(s.iter()) },
            SetTypes::U32Range(s) => unsafe { transmute(s.iter()) },
            SetTypes::LeftStringSliceSmall(s) => unsafe { transmute(s.iter()) },
            SetTypes::LeftStringSliceLarge(s) => unsafe { transmute(s.iter()) },
            SetTypes::RightStringSliceSmall(s) => unsafe { transmute(s.iter()) },
            SetTypes::RightStringSliceLarge(s) => unsafe { transmute(s.iter()) },
            SetTypes::StringLengthSmall(s) => unsafe { transmute(s.iter()) },
        }
    }

    /// Returns a reference to the value in the set, if any, that is equal to the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::facades::FrozenSet;
    ///
    /// let set = FrozenSet::from([1, 2, 3]);
    /// assert_eq!(set.get(&2), Some(&2));
    /// assert_eq!(set.get(&4), None);
    /// ```
    pub fn get(&self, value: &T) -> Option<&T> {
        match &self.set_impl {
            SetTypes::Scanning(s) => s.get(value),
            SetTypes::CommonSmall(s) => s.get(value),
            SetTypes::CommonLarge(s) => s.get(value),
            SetTypes::U32Small(s) => unsafe { transmute(s.get(transmute(value))) },
            SetTypes::U32Large(s) => unsafe { transmute(s.get(transmute(value))) },
            SetTypes::U32Range(s) => unsafe { transmute(s.get(transmute(value))) },
            SetTypes::LeftStringSliceSmall(s) => unsafe {
                let v: &String = transmute(value);
                transmute(s.get(v))
            },
            SetTypes::LeftStringSliceLarge(s) => unsafe {
                let v: &String = transmute(value);
                transmute(s.get(v))
            },
            SetTypes::RightStringSliceSmall(s) => unsafe {
                let v: &String = transmute(value);
                transmute(s.get(v))
            },
            SetTypes::RightStringSliceLarge(s) => unsafe {
                let v: &String = transmute(value);
                transmute(s.get(v))
            },
            SetTypes::StringLengthSmall(s) => unsafe {
                let v: &String = transmute(value);
                transmute(s.get(v))
            },
        }
    }
}

impl<T> FrozenSet<T, RandomState>
where
    T: Hash + Eq,
{
    /// Creates a new frozen set using the default hasher to hash values.
    ///
    /// # Examples
    ///
    /// ```
    /// use frozen_collections_core::facades::FrozenSet;
    /// use std::hash::RandomState;
    ///
    /// let set = FrozenSet::from_vec(vec!(1, 2, 3));
    /// ```
    #[must_use]
    pub fn from_vec(payload: Vec<T>) -> Self {
        Self::new(payload, RandomState::new())
    }
}

impl<T, const N: usize> From<[T; N]> for FrozenSet<T, RandomState>
where
    T: Hash + Eq,
{
    fn from(payload: [T; N]) -> Self {
        Self::new(Vec::from_iter(payload), RandomState::new())
    }
}

impl<T> FromIterator<T> for FrozenSet<T, RandomState>
where
    T: Hash + Eq,
{
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        Self::new(Vec::from_iter(iter), RandomState::new())
    }
}

impl<T, BH> Default for FrozenSet<T, BH>
where
    T: Hash + Eq,
    BH: BuildHasher + Default,
{
    fn default() -> Self {
        Self {
            set_impl: SetTypes::Scanning(ScanningSet::<T>::from_vec(vec![])),
        }
    }
}

impl<T, BH> Debug for FrozenSet<T, BH>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.set_impl {
            SetTypes::Scanning(s) => s.fmt(f),
            SetTypes::CommonSmall(s) => s.fmt(f),
            SetTypes::CommonLarge(s) => s.fmt(f),
            SetTypes::U32Small(s) => s.fmt(f),
            SetTypes::U32Large(s) => s.fmt(f),
            SetTypes::U32Range(s) => s.fmt(f),
            SetTypes::LeftStringSliceSmall(s) => s.fmt(f),
            SetTypes::LeftStringSliceLarge(s) => s.fmt(f),
            SetTypes::RightStringSliceSmall(s) => s.fmt(f),
            SetTypes::RightStringSliceLarge(s) => s.fmt(f),
            SetTypes::StringLengthSmall(s) => s.fmt(f),
        }
    }
}

impl<T, BH> PartialEq<Self> for FrozenSet<T, BH>
where
    T: Hash + Eq,
    BH: BuildHasher,
{
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|value| other.get(value).is_some())
    }
}

impl<T, BH> Eq for FrozenSet<T, BH>
where
    T: Hash + Eq,
    BH: BuildHasher,
{
}

impl<T, ST, BH> BitOr<&ST> for &FrozenSet<T, BH>
where
    T: Hash + Eq + Clone,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    type Output = HashSet<T, BH>;

    fn bitor(self, rhs: &ST) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}

impl<T, ST, BH> BitAnd<&ST> for &FrozenSet<T, BH>
where
    T: Hash + Eq + Clone,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    type Output = HashSet<T, BH>;

    fn bitand(self, rhs: &ST) -> Self::Output {
        self.intersection(rhs).cloned().collect()
    }
}

impl<T, ST, BH> BitXor<&ST> for &FrozenSet<T, BH>
where
    T: Hash + Eq + Clone,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    type Output = HashSet<T, BH>;

    fn bitxor(self, rhs: &ST) -> Self::Output {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<T, ST, BH> Sub<&ST> for &FrozenSet<T, BH>
where
    T: Hash + Eq + Clone,
    ST: Set<T>,
    BH: BuildHasher + Default,
{
    type Output = HashSet<T, BH>;

    fn sub(self, rhs: &ST) -> Self::Output {
        self.difference(rhs).cloned().collect()
    }
}

impl<'a, T, BH> IntoIterator for &'a FrozenSet<T, BH>
where
    T: Hash + Eq,
    BH: BuildHasher,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<T, BH> Len for FrozenSet<T, BH> {
    fn len(&self) -> usize {
        match &self.set_impl {
            SetTypes::Scanning(s) => Len::len(s),
            SetTypes::CommonSmall(s) => Len::len(s),
            SetTypes::CommonLarge(s) => Len::len(s),
            SetTypes::U32Small(s) => Len::len(s),
            SetTypes::U32Large(s) => Len::len(s),
            SetTypes::U32Range(s) => Len::len(s),
            SetTypes::LeftStringSliceSmall(s) => Len::len(s),
            SetTypes::LeftStringSliceLarge(s) => Len::len(s),
            SetTypes::RightStringSliceSmall(s) => Len::len(s),
            SetTypes::RightStringSliceLarge(s) => Len::len(s),
            SetTypes::StringLengthSmall(s) => Len::len(s),
        }
    }
}

impl<T, BH> Set<T> for FrozenSet<T, BH>
where
    T: Hash + Eq,
    BH: BuildHasher,
{
    type Iterator<'a> = Iter<'a, T>
    where
        T: 'a,
        BH: 'a;

    fn iter(&self) -> Iter<'_, T> {
        self.iter()
    }

    fn contains(&self, value: &T) -> bool {
        self.contains(value)
    }
}
