#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]

#![cfg_attr(not(feature = "std"), no_std)]

//! # semval
//!
//! A lightweight, composable framework for semantic validation in Rust.
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
pub type Result<T> = core::result::Result<(), Context<T>>;

/// Objectives that might be violated
///
/// A validation fails if one or more objectives are violated.
///
/// These types are typically `enum`s with one variant per objective.
/// Some of the variants may recursively wrap dependent validations
/// to trace back the root cause.
pub trait Validation: Any + Debug {}

impl<T> Validation for T where T: Any + Debug {}

/// A trait for validating types
///
/// Validation is expected to be an expensive operation that should
/// only be invoked when crossing boundaries between independent
/// components.
pub trait Validate<T>
where
    T: Validation,
{
    /// Perform the validation
    fn validate(&self) -> Result<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    impl Validate<()> for Dummy {
        fn validate(&self) -> Result<()> {
            Context::default().into_result()
        }
    }

    #[test]
    fn validate_dummy() {
        assert!(Dummy.validate().is_ok());
    }
}
