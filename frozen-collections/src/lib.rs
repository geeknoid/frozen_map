//! Frozen collections: fast partially immutable collections
//!
//! Frozen collections are designed to trade creation time for improved
//! read performance. They are ideal for use with long-lasting collections
//! which get initialized when an application starts and remain unchanged
//! permanently, or at least extended periods of time. This is a common
//! pattern in service applications.
//!
//! During creation, the frozen collections perform analyzers over the data they
//! will hold to determine the best layout and algorithm for the specific case.
//! This analyzers can take some time. But the value in spending this time up front
//! is that the collections provide blazingly fast read-time performance.

#[doc(inline)]
pub use {
    frozen_collections_core::facades::FrozenMap, frozen_collections_core::facades::FrozenSet,
    frozen_collections_core::traits::*, frozen_collections_macros::*,
};
pub use frozen_collections_core::*;

