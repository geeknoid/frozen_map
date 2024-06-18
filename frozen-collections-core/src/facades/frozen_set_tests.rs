use std::collections::HashSet;

use crate::facades::frozen_set::FrozenSet;
use crate::traits::len::Len;

#[test]
fn misc() {
    const SIZES: [usize; 12] = [0, 1, 2, 3, 4, 5, 255, 256, 257, 65535, 65536, 65536];

    for size in SIZES {
        let mut v = Vec::with_capacity(size);
        for i in 0..size {
            v.push(i);
        }

        let s = FrozenSet::from_vec(v);
        assert_eq!(size, s.len());
        assert_eq!(size == 0, s.is_empty());

        for i in 0..size {
            assert_eq!(&i, s.get(&i).unwrap());
            assert!(s.contains(&i));
        }

        let mut ms = HashSet::new();
        for item in &s {
            ms.insert(item);
        }

        for i in 0..size {
            assert!(ms.contains(&i));
        }
    }
}
