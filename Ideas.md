# Ideas and Todo Items

* Create a benchmark suite to try and come up with better numbers for the various threshold and percentages
  used in the analysis code.

* In the hash code analyzer, beyond the number of collisions, the logic should factor in how many empty slots are in the
  hash table. A lot of empty slots can slow things down due to cache misses, in addition to wasting memory.

* Consider some hint supplied by the caller for how much time/effort to put into analysis.

* Consider providing an offline tool that performs the analysis on the input data. Being offline, the
  analysis could be more exhaustive. The analysis would produce a little blob of state which would be fed
  into the code to configure things without running analysis code at runtime.

* Consider the use of perfect hashing or minimal perfect hashing.

* Consider introducing dynamic benchmarking as part of the analysis phase. We could build
  several prototype collections, measure effective perf, and then use the benchmark results to
  decide on the optimal collection configuration.

* The collections need to support some notion of Borrow<T>. This is particular important to
  allowing collections where K=String to be queried with &str instead. Unfortunately, given the
  gymnastics the code is doing internally around hashing, it's not obvious how this feature
  could be added.

* Maps should implement iter_mut and values_mut as seen in HashMap.

* Add a specialized set implementation for integer types which uses a bit vector for storage.

* Evaluate hash functions to find the highest performance one

- Bypass hashing for short left-slices or right-slices. When the slices are
  short enough, we should just take the character values as the hash code.

- For integer keys, consider expanding the model for ranges to includes ranges with holes.
  Basically, the array would hold Option<V> instead of just V.

- Add logic to ensure that all the keys are unique during construction of all maps types.
