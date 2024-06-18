//! Specialized read-only maps used as implementation details of frozen maps.
//!
//! In general, you should not need to use these types directly. Instead, use the
//! [`frozen_map!`](crate::frozen_map) macro when you know the items to be stored in the map at compile time, or the
//! [`FrozenMap`](crate::FrozenMap) type when the items are only known at runtime.

pub use common_map::CommonMap;
pub use integer_map::IntegerMap;
pub use integer_range_map::IntegerRangeMap;
pub use iterators::*;
pub use left_slice_map::LeftSliceMap;
pub use length_map::LengthMap;
pub use right_slice_map::RightSliceMap;
pub use scanning_map::ScanningMap;

mod common_map;
mod hash_table;
mod integer_map;
mod integer_range_map;
mod iterators;
mod left_slice_map;
mod length_map;
mod right_slice_map;
mod scanning_map;
