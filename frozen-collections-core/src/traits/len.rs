use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque};
use std::ffi::{CStr, CString, OsStr, OsString};
use std::rc::Rc;
use std::sync::Arc;

/// A trait for describing the length of a collection.
///
/// The amount of data stored in a collection, i.e. the amount of space it requires in memory, is
/// directly proportional to its length. For this reason, `str` and other types measure their
/// lengths in code values (e.g. `u8`), not code points (e.g. `char`).
///
/// Obtaining the length of the collection must take a constant amount of time and space.
pub trait Len {
    /// Returns the length of a collection.
    fn len(&self) -> usize;

    /// Returns whether a collection is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T, S> Len for HashSet<T, S> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<K, V, S> Len for HashMap<K, V, S> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Len for String {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Len for str {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Len for CStr {
    fn len(&self) -> usize {
        self.to_bytes().len()
    }
}

impl Len for CString {
    fn len(&self) -> usize {
        self.as_bytes().len()
    }
}

impl<T> Len for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T: ?Sized + Len> Len for Box<T> {
    fn len(&self) -> usize {
        T::len(self)
    }
}

impl<T: ?Sized + Len> Len for Rc<T> {
    fn len(&self) -> usize {
        T::len(self)
    }
}

impl<T: ?Sized + Len> Len for Arc<T> {
    fn len(&self) -> usize {
        T::len(self)
    }
}

impl<K, V> Len for BTreeMap<K, V> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> Len for BTreeSet<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> Len for BinaryHeap<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> Len for LinkedList<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> Len for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<T> Len for VecDeque<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Len for OsStr {
    fn len(&self) -> usize {
        self.len()
    }
}

impl Len for OsString {
    fn len(&self) -> usize {
        self.as_os_str().len()
    }
}
