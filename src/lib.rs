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
        context::Context as ValidationContext, IntoValidated, Invalidity,
        Result as ValidationResult, Validate, ValidatedFrom, ValidatedResult,
    };
}

mod smallvec;
mod util;

use self::{context::Context, util::*};

use core::{any::Any, fmt::Debug, result::Result as CoreResult};

/// Result of a validation
///
/// The result is `Ok` and empty if the validation succeeded. It is
/// a validation context wrapped into `Err` that carries one or more
/// invalidities.
///
/// In contrast to common results the actual payload is carried by
/// the error variant while a successful result is just the unit type.
pub type Result<V> = UnitResult<Context<V>>;

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

/// Result of a value-to-value conversion with post-validation of the output value
pub type ValidatedResult<V> = CoreResult<V, (V, Context<<V as Validate>::Invalidity>)>;

/// Value-to-value conversion with post-validation of the output value
///
/// On success the output value is returned. On validation errors the output
/// value is returned together with all invalidities.
///
/// If validation of the output value has failed clients
///  - may discard the output and abort,
///  - are able to handle or fix the validation errors and continue, or
///  - accept the output despite validation errors and continue.
///
/// The initial value-to-value conversion from input to output must always succeed.
///
/// The validation is performed on the output value after the input value has
/// been consumed during the conversion. This *post-validation* approach should
/// be applicable and sufficient for most use cases. It simplifies the validated
/// result type by always returning the converted output independent of whether
/// the validation succeeded or failed.
pub trait ValidatedFrom<T>: Validate + Sized {
    /// Convert input value into `Self` and validate `self`
    fn validated_from(from: T) -> ValidatedResult<Self>;
}

impl<T> ValidatedFrom<T> for T
where
    T: Validate,
{
    fn validated_from(from: T) -> ValidatedResult<Self> {
        if let Err(err) = from.validate() {
            Err((from, err))
        } else {
            Ok(from)
        }
    }
}

/// Value-to-value conversion with post-validation of the output value
///
/// Prefer to implement [ValidatedFrom](trait.ValidatedFrom.html) for types inside
/// the current crate. All types that implement [ValidatedFrom](trait.ValidatedFrom.html)
/// implicitly implement this trait.
pub trait IntoValidated<V: Validate> {
    /// Convert `self` into output value and validate the output
    fn into_validated(self) -> ValidatedResult<V>;
}

impl<T, V> IntoValidated<V> for T
where
    V: ValidatedFrom<T>,
{
    fn into_validated(self) -> ValidatedResult<V> {
        V::validated_from(self)
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

    #[test]
    fn validated_from() {
        assert!(AlwaysValid::validated_from(AlwaysValid).is_ok());
        assert!(AlwaysInvalid::validated_from(AlwaysInvalid).is_err());
    }

    #[test]
    fn into_validated() {
        assert!(IntoValidated::<AlwaysValid>::into_validated(AlwaysValid).is_ok());
        assert!(IntoValidated::<AlwaysInvalid>::into_validated(AlwaysInvalid).is_err());
    }
}
