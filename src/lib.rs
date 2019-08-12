#![cfg_attr(not(feature = "std"), no_std)]

pub mod context;

pub mod prelude {
    pub use super::{
        context::Context as ValidationContext, Result as ValidationResult, Validate, Validation,
    };
}

use context::Context;

use core::{any::Any, fmt::Debug};

pub type Result<T> = core::result::Result<(), Context<T>>;

/// A `Validation` defines the context for validating certain objectives.
/// These types are typically an `enum`s with one variant per objective.
/// Some of these variants may recursively wrap dependent validations to
/// trace back the root cause of a validation error.
///
/// For an anonymous or innermost context use the unit type `()`,
/// e.g. when validating non-composite types without the need for
/// any distinctive objectives.
pub trait Validation: Any + Debug {}

impl<T> Validation for T where T: Any + Debug {}

/// A trait for validating types. Validation is expected to be an expensive
/// operation that should only be invoked when crossing boundaries.
pub trait Validate<T>
where
    T: Validation,
{
    /// Perform validation.
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
