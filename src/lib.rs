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

/// Invalidity context
pub mod context;

/// The crate's prelude
///
/// A proposed set of imports to ease usage of this crate.
pub mod prelude {
    pub use super::{
        context::Context as ValidationContext, Invalidity, Result as ValidationResult, Validate,
    };
}

use context::Context;

use core::{any::Any, fmt::Debug};

/// Result of a validation
///
/// The result is `Ok` and empty if the validation succeeded. It is
/// a validation context wrapped into `Err` that carries one or more
/// invalidities.
///
/// In contrast to common results the actual payload is carried by
/// the error variant while a successful result is just the unit type.
pub type Result<V> = core::result::Result<(), Context<V>>;

/// Invalidities that cause validation failures
///
/// Validations fail if one or more objectives are considered invalid.
/// These invalidity objectives are typically represented by sum types
/// (`enum`) with one variant per objective. Some of the variants may
/// recursively wrap an invalidity of a subordinate validation to trace
/// back root causes.
pub trait Invalidity: Any + Debug {}

impl<V> Invalidity for V where V: Any + Debug {}

/// A trait for validating types
///
/// Validation is expected to be an expensive operation that should
/// only be invoked when crossing boundaries between independent
/// components.
pub trait Validate {
    /// Invalidity objectives
    type Invalidity: Invalidity;

    /// Perform the validation
    fn validate(&self) -> Result<Self::Invalidity>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    impl Validate for Dummy {
        type Invalidity = ();

        fn validate(&self) -> Result<Self::Invalidity> {
            Context::new().into()
        }
    }

    #[test]
    fn validate_dummy() {
        assert!(Dummy.validate().is_ok());
    }
}
