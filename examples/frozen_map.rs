use frozen_collections::frozen_map;
use frozen_collections::specialized_sets::{CommonSet, Set};
use frozen_collections::FrozenMap;
use std::collections::HashSet;

fn main() {
    let fm = FrozenMap::from([(0, 1), (2, 3), (4, 5)]);
    assert!(fm.contains_key(&0));

    test_frozen_map();
    test_frozen_set();
}

fn test_frozen_map() {
    let fm = frozen_map!(
        &str,
        "first_key": (1, "first_value"),
        "second_key": (2, "second_value"),
        "third_key": (3, "third_value"),
        "fourth_key": (4, "fourth_value"),
        "fifth_key": (5, "fifth_value"),
        "sixth_key": (6, "sixth_value"),
        "seventh_key": (7, "seventh_value"),
    );

    dbg!(fm);
}

fn test_frozen_set() {
    let cs = CommonSet::<_, u8, _>::from([1, 2, 3]);
    let hs = HashSet::from([3, 4, 5]);
    let _u = cs.union(&hs);
}
