use core::cmp::{max, min};
use core::hash::{BuildHasher, Hash};
use std::collections::{HashMap, HashSet};

/// How to treat keys which are slices for best performance.
#[derive(PartialEq, Eq, Debug)]
pub enum SliceKeyAnalysisResult {
    /// Normal hashing
    Normal,

    /// Hash left-justified subslices
    LeftHandSubslice {
        subslice_index: usize,
        subslice_len: usize,
    },

    /// Hash right-justified subslices
    RightHandSubslice {
        subslice_index: usize,
        subslice_len: usize,
    },

    /// Use the length of the slices as hash codes, instead of hashing the slices
    Length,
}

/// Look for well-known patterns we can optimize for map keys.
///
/// The idea here is to find the shortest subslice across all the input slices which are maximally unique. A corresponding
/// subslice range is then applied to incoming slices being hashed to perform lookups. Keeping the subslices as
/// short as possible minimizes the number of bytes involved in hashing, speeding up the whole process.
///
/// What we do here is pretty simple. We loop over the input slices, looking for the shortest subslice with a good
/// enough uniqueness factor. We look at all the slices both left-justified and right-justified as this maximizes
/// the opportunities to find unique subslices, especially in the case of many slices with the same prefix or suffix.
///
/// We also analyze the length of the input slices. If the length of the slices are sufficiently unique,
/// we can totally skip hashing and just use their lengths as hash codes.
pub fn analyze_slice_keys<'a, K, I, BH>(keys: I, bh: &BH) -> SliceKeyAnalysisResult
where
    K: Hash + 'a,
    I: Iterator<Item = &'a [K]>,
    BH: BuildHasher,
{
    let keys = keys.collect();

    // first, see if we can just use slice lengths as hash codes
    let result = analyze_lengths(&keys);

    if result == SliceKeyAnalysisResult::Normal {
        // if we can't use slice lengths, look for suitable subslices
        analyze_subslices(&keys, bh)
    } else {
        result
    }
}

/// See if we can use slice lengths instead of hashing
fn analyze_lengths<T>(keys: &Vec<&[T]>) -> SliceKeyAnalysisResult {
    const MAX_IDENTICAL_LENGTHS: usize = 3;
    const MAX_SLICES: usize = 255;

    if keys.len() > MAX_SLICES {
        // if there are a lof of slices, assume we'll get too many length collisions
        return SliceKeyAnalysisResult::Normal;
    }

    let mut lengths = HashMap::new();
    for s in keys {
        let v = lengths.get(&s.len());
        if let Some(count) = v {
            if count == &MAX_IDENTICAL_LENGTHS {
                return SliceKeyAnalysisResult::Normal;
            }

            lengths.insert(s.len(), count + 1);
        } else {
            lengths.insert(s.len(), 1);
        }
    }

    SliceKeyAnalysisResult::Length
}

/// See if we can use subslices to reduce the time spent hashing
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
fn analyze_subslices<T, BH>(keys: &Vec<&[T]>, bh: &BH) -> SliceKeyAnalysisResult
where
    T: Hash,
    BH: BuildHasher,
{
    // constrain the amount of work we do in this code
    const MAX_SUBSLICE_LENGTH_LIMIT: usize = 16;
    const ACCEPTABLE_DUPLICATE_PERCENT: f64 = 0.05;

    let mut min_len = usize::MAX;
    let mut max_len = 0;
    for s in keys {
        min_len = min(min_len, s.len());
        max_len = max(max_len, s.len());
    }

    // tolerate a certain amount of duplicate subslices
    let acceptable_duplicates = ((keys.len() as f64) * ACCEPTABLE_DUPLICATE_PERCENT) as usize;

    // this set is reused for each call to is_sufficiently_unique
    let mut set = HashSet::with_capacity(keys.len());

    // for each subslice length, prefer the shortest length that provides enough uniqueness
    let max_subslice_len = min(min_len, MAX_SUBSLICE_LENGTH_LIMIT);

    let mut subslice_len = 1;
    while subslice_len <= max_subslice_len {
        // For each index, get a uniqueness factor for the left-justified subslices.
        // If any is above our threshold, we're done.
        let mut subslice_index = 0;
        while subslice_index <= min_len - subslice_len {
            if is_sufficiently_unique(
                keys,
                subslice_index,
                subslice_len,
                true,
                &mut set,
                acceptable_duplicates,
                bh,
            ) {
                return if subslice_len == max_len {
                    SliceKeyAnalysisResult::Normal
                } else {
                    SliceKeyAnalysisResult::LeftHandSubslice {
                        subslice_index,
                        subslice_len,
                    }
                };
            }

            subslice_index += 1;
        }

        // There were no left-justified slices of this length available.
        // If all the slices are of the same length, then just checking left-justification is sufficient.
        // But if any slices are of different lengths, then we'll get different alignments for left- vs
        // right-justified subslices, and so we also check right-justification.
        if min_len != max_len {
            // For each index, get a uniqueness factor for the right-justified subslices.
            // If any is above our threshold, we're done.
            subslice_index = 0;
            while subslice_index <= min_len - subslice_len {
                if is_sufficiently_unique(
                    keys,
                    subslice_index,
                    subslice_len,
                    false,
                    &mut set,
                    acceptable_duplicates,
                    bh,
                ) {
                    return SliceKeyAnalysisResult::RightHandSubslice {
                        subslice_index,
                        subslice_len,
                    };
                }

                subslice_index += 1;
            }
        }

        subslice_len += 1;
    }

    // could not find a subslice that was good enough.
    SliceKeyAnalysisResult::Normal
}

fn is_sufficiently_unique<T, BH>(
    keys: &Vec<&[T]>,
    subslice_index: usize,
    subslice_len: usize,
    left_justified: bool,
    set: &mut HashSet<u64>,
    acceptable_duplicates: usize,
    bh: &BH,
) -> bool
where
    T: Hash,
    BH: BuildHasher,
{
    set.clear();

    let mut acceptable_duplicates = acceptable_duplicates;
    for s in keys {
        let sub = if left_justified {
            &s[subslice_index..subslice_index + subslice_len]
        } else {
            let start = s.len() - subslice_index - 1;
            &s[start..start + subslice_len]
        };

        if !set.insert(bh.hash_one(sub)) {
            if acceptable_duplicates == 0 {
                return false;
            }

            acceptable_duplicates -= 1;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use std::hash::RandomState;

    use super::*;

    struct AnalysisTestCase<'a> {
        slices: &'a [&'a str],
        expected: SliceKeyAnalysisResult,
    }

    #[test]
    fn analyze_string_keys_test() {
        const ANALYSIS_TEST_CASES: [AnalysisTestCase; 9] = [
            AnalysisTestCase {
                slices: &[
                    "AAA", "ABB", "ACC", "ADD", "AEE", "AFF", "AGG", "AHH", "AII", "AJJ", "AKK",
                    "ALL", "AMM", "ANN", "AOO", "APP", "AQQ", "ARR", "ASS", "ATT", "AUU",
                ],
                expected: SliceKeyAnalysisResult::LeftHandSubslice {
                    subslice_index: 1,
                    subslice_len: 1,
                },
            },
            AnalysisTestCase {
                slices: &["A00", "B00", "C00", "D00"],
                expected: SliceKeyAnalysisResult::LeftHandSubslice {
                    subslice_index: 0,
                    subslice_len: 1,
                },
            },
            AnalysisTestCase {
                slices: &["A", "B", "C", "D", "E2"],
                expected: SliceKeyAnalysisResult::LeftHandSubslice {
                    subslice_index: 0,
                    subslice_len: 1,
                },
            },
            AnalysisTestCase {
                slices: &["A", "B", "C", "D", "E2", ""],
                expected: SliceKeyAnalysisResult::Normal,
            },
            AnalysisTestCase {
                slices: &["XA", "XB", "XC", "XD", "XE2"],
                expected: SliceKeyAnalysisResult::LeftHandSubslice {
                    subslice_index: 1,
                    subslice_len: 1,
                },
            },
            AnalysisTestCase {
                slices: &["XXA", "XXB", "XXC", "XXD", "XXX", "XXXE"],
                expected: SliceKeyAnalysisResult::RightHandSubslice {
                    subslice_index: 0,
                    subslice_len: 1,
                },
            },
            AnalysisTestCase {
                slices: &["ABC", "DEFG", "HIJKL", "MNOPQR", "STUVWX", "YZ"],
                expected: SliceKeyAnalysisResult::Length,
            },
            AnalysisTestCase {
                slices: &[
                    "ABC", "DEFG", "HIJKL", "MNOPQR", "STUVWX", "YZ", "D2", "D3", "D4",
                ],
                expected: SliceKeyAnalysisResult::LeftHandSubslice {
                    subslice_index: 1,
                    subslice_len: 1,
                },
            },
            AnalysisTestCase {
                slices: &["AAA", "1AA", "A1A", "AA1", "BBB", "1BB", "B1B", "BB1"],
                expected: SliceKeyAnalysisResult::Normal,
            },
        ];

        for (count, case) in ANALYSIS_TEST_CASES.into_iter().enumerate() {
            println!("Test case #{count}");

            let keys = case.slices.iter().map(|x| x.as_bytes());
            assert_eq!(case.expected, analyze_slice_keys(keys, &RandomState::new()));
        }
    }
}
