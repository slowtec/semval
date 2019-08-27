#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]
#![cfg_attr(not(feature = "std"), no_std)]

//! # semval
//!
//! A lightweight and versatile toolbox for implementing semantic validation.
//!
//! Please refer to the bundled `reservation.rs` example to get an idea of how it works.
//!
//! Without any macro magic, at least not now.

/// Validation context
pub mod context;

/// The crate's prelude
///
/// A proposed set of imports to ease usage of this crate.
pub mod prelude {
    pub use super::{
        context::Context as ValidationContext, Result as ValidationResult, Validate, Validation,
    };
}

use context::Context;

use core::{any::Any, fmt::Debug};

/// Result of a validation
///
/// The result is ok and empty if the validation succeeded
/// or otherwise a validation context with one or more
/// violations.
pub type Result<V> = core::result::Result<(), Context<V>>;

/// Validation objectives that might be violated
///
/// A validation fails if one or more objectives are violated.
///
/// These types are typically `enum`s with one variant per objective.
/// Some of the variants may recursively wrap dependent validations
/// to trace back the root cause.
pub trait Validation: Any + Debug {}

impl<V> Validation for V where V: Any + Debug {}

/// A trait for validating types
///
/// Validation is expected to be an expensive operation that should
/// only be invoked when crossing boundaries between independent
/// components.
pub trait Validate {
    /// Validation objectives
    type Validation: Validation;

    /// Perform the validation
    fn validate(&self) -> Result<Self::Validation>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    impl Validate for Dummy {
        type Validation = ();

        fn validate(&self) -> Result<Self::Validation> {
            Context::valid().into()
        }
    }

    #[test]
    fn validate_dummy() {
        assert!(Dummy.validate().is_ok());
    }
}
