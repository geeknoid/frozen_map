use num_traits::PrimInt;

/// How to treat integer keys for best performance.
#[derive(PartialEq, Eq, Debug)]
pub enum IntKeyAnalysisResult {
    /// Normal hashing
    Normal,

    /// All keys are in a continuous range
    Range,
}

/// Look for well-known patterns we can optimize for with integer map keys.
pub fn analyze_int_keys<K, I>(keys: I) -> IntKeyAnalysisResult
where
    K: PrimInt,
    I: Iterator<Item = K>,
{
    let mut min = K::max_value();
    let mut max = K::min_value();
    let mut count = K::zero();
    for key in keys {
        min = min.min(key);
        max = max.max(key);
        count = count + K::one();
    }

    if count == K::zero() {
        IntKeyAnalysisResult::Normal
    } else if max.sub(min) == count - K::one() {
        IntKeyAnalysisResult::Range
    } else {
        IntKeyAnalysisResult::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_int_keys_normal() {
        let keys = vec![1, 3, 5, 7, 9];
        let result = analyze_int_keys(keys.into_iter());
        assert_eq!(result, IntKeyAnalysisResult::Normal);
    }

    #[test]
    fn test_analyze_int_keys_range() {
        let keys = vec![1, 2, 3, 4, 5];
        let result = analyze_int_keys(keys.into_iter());
        assert_eq!(result, IntKeyAnalysisResult::Range);
    }

    #[test]
    fn test_analyze_int_keys_empty() {
        let keys: Vec<i32> = vec![];
        let result = analyze_int_keys(keys.into_iter());
        assert_eq!(result, IntKeyAnalysisResult::Normal);
    }

    #[test]
    fn test_analyze_int_keys_single() {
        let keys = vec![1];
        let result = analyze_int_keys(keys.into_iter());
        assert_eq!(result, IntKeyAnalysisResult::Range);
    }
}
