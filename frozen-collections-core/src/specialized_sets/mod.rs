//! Specialized read-only sets used as implementation details of frozen sets.
//!  
//! In general, you should not need to use these types directly. Instead, use the
//! [`frozen_set!`](crate::frozen_set) macro when you know the items to be stored in the set at compile time, or the
//! [`FrozenSet`](crate::FrozenSet) type when the items are only known at runtime.

pub use common_set::CommonSet;
pub use integer_range_set::IntegerRangeSet;
pub use integer_set::IntegerSet;
pub use iterators::*;
pub use left_slice_set::LeftSliceSet;
pub use length_set::LengthSet;
pub use right_slice_set::RightSliceSet;
pub use scanning_set::ScanningSet;
pub use set::*;

mod common_set;
mod integer_range_set;
mod integer_set;
mod iterators;
mod left_slice_set;
mod length_set;
mod right_slice_set;
mod scanning_set;
mod set;
mod set_ops;
