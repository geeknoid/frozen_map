use bitvec::prelude::*;

/// Results of running an analyzers over a collection of hash code values.
pub struct HashCodeAnalysisResult {
    /// The recommended hash table size. This is not necessarily optimal, but it's good enough.
    pub num_hash_slots: usize,

    /// The number of collisions when using the recommended table size.
    pub _num_hash_collisions: usize,
}

/// Given a collection of hash codes, figures out the best hash table size to use to minimize both table size snd collisions.
#[allow(clippy::cast_possible_truncation)]
pub fn analyze_hash_codes<I>(hash_codes: I) -> HashCodeAnalysisResult
where
    I: Iterator<Item = u64>,
{
    // What is a satisfactory rate of hash collisions?
    const ACCEPTABLE_COLLISION_PERCENTAGE: usize = 5;

    // By how much do we shrink the acceptable # collisions per iteration?
    const ACCEPTABLE_COLLISION_PERCENTAGE_OF_REDUCTION: usize = 20;

    // thresholds to categorize input sizes
    const MEDIUM_INPUT_SIZE_THRESHOLD: usize = 128;
    const LARGE_INPUT_SIZE_THRESHOLD: usize = 1000;

    // amount by which the table can be larger than the input
    const MAX_SMALL_INPUT_MULTIPLIER: usize = 16;
    const MAX_MEDIUM_INPUT_MULTIPLIER: usize = 10;
    const MAX_LARGE_INPUT_MULTIPLIER: usize = 3;

    // Table of prime numbers to use as hash table sizes for medium-sized inputs
    const PRIMES: [usize; 60] = [
        131, 163, 197, 239, 293, 353, 431, 521, 631, 761, 919, 1103, 1327, 1597, 1931, 2333, 2801,
        3371, 4049, 4861, 5839, 7013, 8419, 10_103, 12_143, 14_591, 17_519, 21_023, 25_229, 30_293,
        36_353, 43_627, 52_361, 62_851, 75_431, 90_523, 108_631, 130_363, 156_437, 187_751,
        225_307, 270_371, 324_449, 389_357, 467_237, 560_689, 672_827, 807_403, 968_897, 1_162_687,
        1_395_263, 1_674_319, 2_009_191, 2_411_033, 2_893_249, 3_471_899, 4_166_287, 4_999_559,
        5_999_471, 7_199_369,
    ];

    let hash_codes: Vec<u64> = hash_codes.collect();
    let mut acceptable_collisions = if hash_codes.len() < MEDIUM_INPUT_SIZE_THRESHOLD {
        // for small enough inputs, we try for perfection
        0
    } else {
        (hash_codes.len() / 100) * ACCEPTABLE_COLLISION_PERCENTAGE
    };

    // the minimum table size we can tolerate, given the acceptable collision rate
    let min_size = hash_codes.len() - acceptable_collisions;

    // the maximum table size we consider, given a scaled growth factor for different input sizes
    let max_size = if hash_codes.len() < MEDIUM_INPUT_SIZE_THRESHOLD {
        hash_codes.len() * MAX_SMALL_INPUT_MULTIPLIER
    } else if hash_codes.len() < LARGE_INPUT_SIZE_THRESHOLD {
        hash_codes.len() * MAX_MEDIUM_INPUT_MULTIPLIER
    } else {
        hash_codes.len() * MAX_LARGE_INPUT_MULTIPLIER
    };

    let mut use_table: BitVec = BitVec::with_capacity(max_size);
    use_table.resize(max_size, false);

    let mut best_size = 0;
    let mut best_num_collisions = hash_codes.len();

    let mut sizes = Vec::new();

    // always try the exact size first to optimally handle cases where the keys are unique integers
    sizes.push(hash_codes.len());

    if max_size < MEDIUM_INPUT_SIZE_THRESHOLD {
        sizes.extend(min_size..=max_size);
    } else if min_size < PRIMES[PRIMES.len() - 1] {
        // For medium input sizes, we only consider a predefined set of prime numbers rather than being exhaustive as in the
        // case for smaller input sizes. This is to constrain the total amount of compute time that gets spent in this code.
        sizes.extend(PRIMES);
    } else {
        // For very large input sizes, we try a few multiples of the input size
        let mut size = min_size;
        let increment = hash_codes.len() / 3;
        while size <= max_size {
            sizes.push(size);

            size += increment;

            // find next prime
            size |= 1;
            while !is_prime(size as u64) {
                size += 2;
            }
        }
    }

    for size in sizes {
        if size < min_size {
            continue;
        } else if size > max_size {
            break;
        }

        use_table.fill(false);
        let mut num_collisions = 0;

        for code in &hash_codes {
            let slot = (code % (size as u64)) as usize;
            if use_table[slot] {
                num_collisions += 1;
                if num_collisions >= best_num_collisions {
                    break;
                }
            } else {
                use_table.set(slot, true);
            }
        }

        if num_collisions < best_num_collisions {
            if best_size == 0 || num_collisions <= acceptable_collisions {
                best_num_collisions = num_collisions;
                best_size = size;
            }

            if num_collisions <= acceptable_collisions {
                // we have a winner!
                break;
            }
        }

        if acceptable_collisions > 0 {
            // The larger the table, the fewer collisions we tolerate. The idea
            // here is to reduce the risk of a table getting very big and still
            // having a relatively high count of collisions.
            acceptable_collisions =
                (acceptable_collisions / 100) * ACCEPTABLE_COLLISION_PERCENTAGE_OF_REDUCTION;
        }
    }

    HashCodeAnalysisResult {
        num_hash_slots: best_size,
        _num_hash_collisions: best_num_collisions,
    }
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_precision_loss)]
fn is_prime(candidate: u64) -> bool {
    if candidate % 3 == 0 || candidate % 5 == 0 {
        return false;
    }

    let limit = f64::sqrt(candidate as f64) as u64;
    let mut divisor = 3;
    while divisor <= limit {
        if candidate % divisor == 0 {
            return false;
        }

        divisor += 2;
    }

    true
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    use super::*;

    struct AnalysisTestCase {
        num_hash_codes: usize,
        randomize_hash_codes: bool,
        expected_num_hash_slots: usize,
        expected_num_hash_collisions: usize,
    }

    #[test]
    #[allow(clippy::used_underscore_binding)]
    fn analyze_hash_codes_test() {
        const ANALYSIS_TEST_CASES: [AnalysisTestCase; 5] = [
            AnalysisTestCase {
                num_hash_codes: 0,
                randomize_hash_codes: true,
                expected_num_hash_slots: 0,
                expected_num_hash_collisions: 0,
            },
            AnalysisTestCase {
                num_hash_codes: 2,
                randomize_hash_codes: true,
                expected_num_hash_slots: 2,
                expected_num_hash_collisions: 0,
            },
            AnalysisTestCase {
                num_hash_codes: 1000,
                randomize_hash_codes: true,
                expected_num_hash_slots: 1000,
                expected_num_hash_collisions: 359,
            },
            AnalysisTestCase {
                num_hash_codes: 8_000_000,
                randomize_hash_codes: false,
                expected_num_hash_slots: 8_000_000,
                expected_num_hash_collisions: 0,
            },
            AnalysisTestCase {
                num_hash_codes: 8_000_000,
                randomize_hash_codes: true,
                expected_num_hash_slots: 8_000_000,
                expected_num_hash_collisions: 2_941_169,
            },
        ];

        for (count, case) in ANALYSIS_TEST_CASES.iter().enumerate() {
            println!("Test case #{count}");

            let mut rng = StdRng::seed_from_u64(42);
            let mut hash_codes = Vec::with_capacity(case.num_hash_codes);

            if case.randomize_hash_codes {
                for _ in 0..case.num_hash_codes {
                    hash_codes.push(rng.gen());
                }
            } else {
                for count in 0..case.num_hash_codes {
                    hash_codes.push(count as u64);
                }
            }

            let result = analyze_hash_codes(hash_codes.iter().copied());

            assert_eq!(case.expected_num_hash_slots, result.num_hash_slots);
            assert_eq!(
                case.expected_num_hash_collisions,
                result._num_hash_collisions
            );
        }
    }
}
