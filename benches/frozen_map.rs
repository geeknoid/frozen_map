use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use frozen_collections::FrozenMap;

fn u32_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("u32_keys");

    group.bench_function("FrozenMap", |b| {
        let map = FrozenMap::from([(0u32, 1), (2, 3), (4, 5), (6, 7), (8, 9)]);
        b.iter(|| {
            _ = black_box(map.get(&4));
        });
    });

    let map = HashMap::from([(0u32, 1), (2, 3), (4, 5), (6, 7), (8, 9)]);
    group.bench_function("HashMap", |b| {
        b.iter(|| {
            _ = black_box(map.get(&4));
        });
    });

    group.finish();
}

fn u32_keys_range(c: &mut Criterion) {
    let mut group = c.benchmark_group("u32_keys_range");

    group.bench_function("FrozenMap", |b| {
        let map = FrozenMap::from([(0u32, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
        b.iter(|| {
            _ = black_box(map.get(&4));
        });
    });

    let map = HashMap::from([(0u32, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
    group.bench_function("HashMap", |b| {
        b.iter(|| {
            _ = black_box(map.get(&4));
        });
    });

    group.finish();
}

fn i32_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("i32_keys");

    group.bench_function("FrozenMap", |b| {
        let map = FrozenMap::from([(0, 1), (2, 3), (4, 5), (6, 7), (8, 9)]);
        b.iter(|| {
            _ = black_box(map.get(&4));
        });
    });

    let map = HashMap::from([(0, 1), (2, 3), (4, 5), (6, 7), (8, 9)]);
    group.bench_function("HashMap", |b| {
        b.iter(|| {
            _ = black_box(map.get(&4));
        });
    });

    group.finish();
}

fn string_keys_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_keys_length");
    let keys = [
        "Red".to_string(),
        "Green".to_string(),
        "Blue".to_string(),
        "Cyan".to_string(),
        "Magenta".to_string(),
        "Purple".to_string(),
    ];
    let kvs = [
        ("Red".to_string(), 1),
        ("Green".to_string(), 2),
        ("Green".to_string(), 3),
        ("Cyan".to_string(), 4),
        ("Magenta".to_string(), 5),
        ("Purple".to_string(), 6),
    ];

    group.bench_function("FrozenMap", |b| {
        let map = FrozenMap::from(kvs.clone());
        b.iter(|| {
            _ = black_box(map.get(&keys[3]));
        });
    });

    let map = HashMap::from(kvs.clone());
    group.bench_function("HashMap", |b| {
        b.iter(|| {
            _ = black_box(map.get(&keys[3]));
        });
    });

    group.finish();
}

fn string_keys_subslice(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_keys_subslice");
    let keys = [
        "abcdefghi0".to_string(),
        "abcdefghi1".to_string(),
        "abcdefghi2".to_string(),
        "abcdefghi3".to_string(),
        "abcdefghi4".to_string(),
        "abcdefghi5".to_string(),
    ];
    let kvs = [
        ("abcdefghi0".to_string(), 1),
        ("abcdefghi1".to_string(), 2),
        ("abcdefghi2".to_string(), 3),
        ("abcdefghi3".to_string(), 4),
        ("abcdefghi4".to_string(), 5),
        ("abcdefghi5".to_string(), 6),
    ];

    group.bench_function("FrozenMap", |b| {
        let map = FrozenMap::from(kvs.clone());
        b.iter(|| {
            _ = black_box(map.get(&keys[3]));
        });
    });

    let map = HashMap::from(kvs.clone());
    group.bench_function("HashMap", |b| {
        b.iter(|| {
            _ = black_box(map.get(&keys[3]));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    string_keys_length,
    string_keys_subslice,
    u32_keys,
    u32_keys_range,
    i32_keys
);
criterion_main!(benches);
