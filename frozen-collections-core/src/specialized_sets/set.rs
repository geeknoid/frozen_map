use crate::specialized_sets::set_ops::{is_disjoint, is_subset, is_superset};
use crate::specialized_sets::{Difference, Intersection, SymmetricDifference, Union};
use crate::traits::len::Len;
use core::hash::{BuildHasher, Hash};
use std::collections::hash_set::Iter;
use std::collections::{BTreeSet, HashSet};

pub trait Set<T>: Len {
    type Iterator<'a>: Iterator<Item = &'a T>
    where
        Self: 'a,
        T: 'a;

    fn iter(&self) -> Self::Iterator<'_>;
    fn contains(&self, value: &T) -> bool;

    /// Visits the values representing the union,
    /// i.e., all the values in `self` or `other`, without duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use frozen_collections_core::facades::FrozenSet;
    /// use frozen_collections_core::specialized_sets::Set;
    ///
    /// let a = FrozenSet::from([1, 2, 3]);
    /// let b = HashSet::from([4, 2, 3, 4]);
    ///
    /// // Print 1, 2, 3, 4 in arbitrary order.
    /// for x in a.union(&b) {
    ///     println!("{x}");
    /// }
    ///
    /// let union: HashSet<_> = a.union(&b).collect();
    /// assert_eq!(union, [1, 2, 3, 4].iter().collect());
    /// ```
    fn union<'a, ST>(&'a self, other: &'a ST) -> Union<'a, Self, ST, T>
    where
        ST: Set<T>,
        Self: Sized,
    {
        Union::new(self, other)
    }

    /// Visits the values representing the symmetric difference,
    /// i.e., the values that are in `self` or in `other` but not in both.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use frozen_collections_core::facades::FrozenSet;
    /// use frozen_collections_core::specialized_sets::Set;
    ///
    /// let a = FrozenSet::from([1, 2, 3]);
    /// let b = HashSet::from([4, 2, 3, 4]);
    ///
    /// // Print 1, 4 in arbitrary order.
    /// for x in a.symmetric_difference(&b) {
    ///     println!("{x}");
    /// }
    /// ```
    fn symmetric_difference<'a, ST>(&'a self, other: &'a ST) -> SymmetricDifference<'a, Self, ST, T>
    where
        ST: Set<T>,
        Self: Sized,
    {
        SymmetricDifference::new(self, other)
    }

    /// Visits the values representing the difference,
    /// i.e., the values that are in `self` but not in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use frozen_collections_core::facades::FrozenSet;
    /// use frozen_collections_core::specialized_sets::Set;
    ///
    /// let a = FrozenSet::from([1, 2, 3]);
    /// let b = HashSet::from([4, 2, 3, 4]);
    ///
    /// // Can be seen as `a - b`.
    /// for x in a.difference(&b) {
    ///     println!("{x}"); // Print 1
    /// }
    ///
    /// let diff: HashSet<_> = a.difference(&b).collect();
    /// assert_eq!(diff, [1].iter().collect());
    /// ```
    fn difference<'a, ST>(&'a self, other: &'a ST) -> Difference<'a, Self, ST, T>
    where
        ST: Set<T>,
        Self: Sized,
    {
        Difference::new(self, other)
    }

    /// Visits the values representing the intersection,
    /// i.e., the values that are both in `self` and `other`.
    ///
    /// When an equal element is present in `self` and `other`
    /// then the resulting `Intersection` may yield references to
    /// one or the other. This can be relevant if `T` contains fields which
    /// are not compared by its `Eq` implementation, and may hold different
    /// value between the two equal copies of `T` in the two sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    /// use frozen_collections_core::facades::FrozenSet;
    /// use frozen_collections_core::specialized_sets::Set;
    ///
    /// let a = FrozenSet::from([1, 2, 3]);
    /// let b = HashSet::from([4, 2, 3, 4]);
    ///
    /// // Print 2, 3 in arbitrary order.
    /// for x in a.intersection(&b) {
    ///     println!("{x}");
    /// }
    ///
    /// let intersection: HashSet<_> = a.intersection(&b).collect();
    /// assert_eq!(intersection, [2, 3].iter().collect());
    /// ```
    fn intersection<'a, ST>(&'a self, other: &'a ST) -> Intersection<'a, Self, ST, T>
    where
        ST: Set<T>,
        Self: Sized,
    {
        Intersection::new(self, other)
    }

    /// Returns `true` if `self` has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    fn is_disjoint<'a, ST>(&'a self, other: &'a ST) -> bool
    where
        ST: Set<T>,
        Self: Sized,
    {
        is_disjoint(self, other)
    }

    /// Returns `true` if the set is a subset of another,
    /// i.e., `other` contains at least all the values in `self`.
    fn is_subset<'a, ST>(&'a self, other: &'a ST) -> bool
    where
        ST: Set<T>,
        Self: Sized,
    {
        is_subset(self, other)
    }

    /// Returns `true` if the set is a superset of another,
    /// i.e., `self` contains at least all the values in `other`.
    fn is_superset<'a, ST>(&'a self, other: &'a ST) -> bool
    where
        ST: Set<T>,
        Self: Sized,
    {
        is_superset(self, other)
    }
}

impl<T, BH> Set<T> for HashSet<T, BH>
where
    T: Hash + Eq,
    BH: BuildHasher,
{
    type Iterator<'a> = Iter<'a, T>
    where
        T: 'a,
        BH: 'a;

    fn iter(&self) -> Self::Iterator<'_> {
        Self::iter(self)
    }

    fn contains(&self, value: &T) -> bool {
        Self::contains(self, value)
    }
}

impl<T> Set<T> for BTreeSet<T>
where
    T: Ord,
{
    type Iterator<'a> = std::collections::btree_set::Iter<'a, T>
    where
        T: 'a;

    fn iter(&self) -> Self::Iterator<'_> {
        Self::iter(self)
    }

    fn contains(&self, value: &T) -> bool {
        Self::contains(self, value)
    }
}
