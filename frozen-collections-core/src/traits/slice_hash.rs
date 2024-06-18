use core::hash::{BuildHasher, Hasher};
use core::ops::Range;

/// Enables hashing over a slice of an input.
pub trait SliceHash {
    /// Hash only a slice.
    #[must_use]
    fn hash<BH: BuildHasher>(&self, bh: &BH, range: Range<usize>) -> u64;
}

impl SliceHash for String {
    #[inline]
    fn hash<BH: BuildHasher>(&self, bh: &BH, range: Range<usize>) -> u64 {
        let mut h = bh.build_hasher();
        let b = unsafe { &self.as_bytes().get_unchecked(range) };
        h.write(b);
        h.finish()
    }
}

impl SliceHash for [u8] {
    #[inline]
    fn hash<BH: BuildHasher>(&self, bh: &BH, range: Range<usize>) -> u64 {
        let mut h = bh.build_hasher();
        let b = unsafe { &self.get_unchecked(range) };
        h.write(b);
        h.finish()
    }
}
