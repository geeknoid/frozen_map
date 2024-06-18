use std::collections::HashMap;

use crate::facades::frozen_map::FrozenMap;

#[test]
fn test_empty_map() {
    type FM = FrozenMap<i32, i32>;

    let m = FM::default();
    assert_eq!(m.len(), 0);
}

#[test]
fn test_i32_map() {
    let m = FrozenMap::<i32, i32>::from([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5), (6, 6)]);
    assert_eq!(m.get(&6), Some(&6));
}

#[test]
fn basic_u32_map() {
    let max_entries = [1, 2, 3, 4, 5, 6, 255, 256, 65535, 65536];

    for max in max_entries {
        let mut m = HashMap::<u32, String>::new();
        for i in 0..max {
            m.insert(i, format!("V{i}"));
        }

        let fm = m
            .iter()
            .map(|x| (*x.0, x.1.clone()))
            .collect::<FrozenMap<_, _>>();
        assert_eq!(m.len(), fm.len());
        assert_eq!(m.is_empty(), fm.is_empty());

        for pair in &m {
            assert!(fm.contains_key(pair.0));
            assert_eq!(m.get(pair.0).unwrap(), fm.get(pair.0).unwrap());
            assert_eq!(
                m.get_key_value(pair.0).unwrap(),
                fm.get_key_value(pair.0).unwrap()
            );
        }

        let mut m = HashMap::<u32, String>::new();
        for i in (0..max).map(|x| x * 2) {
            m.insert(i, "V{i}".to_string());
        }

        let fd = m
            .iter()
            .map(|x| (*x.0, x.1.clone()))
            .collect::<FrozenMap<_, _>>();
        assert_eq!(m.len(), fd.len());
        assert_eq!(m.is_empty(), fd.is_empty());

        for pair in &m {
            assert!(fd.contains_key(pair.0));
            assert_eq!(m.get(pair.0).unwrap(), fd.get(pair.0).unwrap());
            assert_eq!(
                m.get_key_value(pair.0).unwrap(),
                fd.get_key_value(pair.0).unwrap()
            );
        }
    }
}

#[test]
fn test_iter() {
    let mut m = HashMap::new();
    m.insert(1, 10);
    m.insert(2, 20);
    m.insert(3, 30);
    m.insert(4, 40);
    let m = m.iter().collect::<FrozenMap<_, _>>();

    let mut iter = m.iter();
    println!("{iter:?}");
    iter.next();
    println!("{iter:?}");
}
