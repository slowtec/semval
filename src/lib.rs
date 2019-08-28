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
///
/// Implementations are required to implement `Debug` to enable analysis
/// and low-level logging of those recursively wrapped sum types.
///
/// The trait bound `Any` is implicitly implemented for most types and
/// enables basic type inspection and downcasting for generically handling
/// validation results though runtime reflection.
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

/// Validate `Some` or otherwise implicitly evaluate to `Ok`
/// in case of `None`
///
/// If the absence of an optional value is considered a validation
/// error this must be checked separately.
impl<V> Validate for Option<V>
where
    V: Validate,
{
    type Invalidity = V::Invalidity;

    fn validate(&self) -> Result<Self::Invalidity> {
        if let Some(ref some) = self {
            some.validate()
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysValid;

    impl Validate for AlwaysValid {
        type Invalidity = ();

        fn validate(&self) -> Result<Self::Invalidity> {
            Context::new().into()
        }
    }

    struct AlwaysInvalid;

    impl Validate for AlwaysInvalid {
        type Invalidity = ();

        fn validate(&self) -> Result<Self::Invalidity> {
            Context::new().invalidate(()).into()
        }
    }

    #[test]
    fn validate() {
        assert!(AlwaysValid.validate().is_ok());
        assert!(AlwaysInvalid.validate().is_err());
    }

    #[test]
    fn validate_option_none() {
        assert!((None as Option<AlwaysValid>).validate().is_ok());
        assert!((None as Option<AlwaysInvalid>).validate().is_ok());
    }

    #[test]
    fn validate_option_some() {
        assert!(Some(AlwaysValid).validate().is_ok());
        assert!(Some(AlwaysInvalid).validate().is_err());
    }
}
