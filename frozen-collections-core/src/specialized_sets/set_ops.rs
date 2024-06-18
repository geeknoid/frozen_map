use crate::specialized_sets::Set;

pub fn is_disjoint<'a, S1, S2, T>(s1: &'a S1, s2: &'a S2) -> bool
where
    S1: Set<T>,
    S2: Set<T>,
    T: 'a,
{
    if s1.len() <= s1.len() {
        s1.iter().all(|v| !s2.contains(v))
    } else {
        s2.iter().all(|v| !s1.contains(v))
    }
}

pub fn is_subset<'a, S1, S2, T>(s1: &'a S1, s2: &'a S2) -> bool
where
    S1: Set<T>,
    S2: Set<T>,
    T: 'a,
{
    if s1.len() <= s2.len() {
        s1.iter().all(|v| s2.contains(v))
    } else {
        false
    }
}

pub fn is_superset<'a, S1, S2, T>(s1: &'a S1, s2: &'a S2) -> bool
where
    S1: Set<T>,
    S2: Set<T>,
    T: 'a,
{
    if s2.len() <= s1.len() {
        s2.iter().all(|v| s1.contains(v))
    } else {
        false
    }
}
