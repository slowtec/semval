#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(broken_intra_doc_links)]
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
        context::Context as ValidationContext, IntoValidated, Invalidity, IsValid,
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

/// A utility trait for boolean validity checks.
pub trait IsValid {
    /// Check if this instance is valid, discarding all further
    /// information why if not.
    fn is_valid(&self) -> bool;
}

impl<T> IsValid for T
where
    T: Validate,
{
    fn is_valid(&self) -> bool {
        self.validate().is_ok()
    }
}

/// `Validate` is implemented for any reference of a type
/// that implements `Validate`.
impl<'a, V> Validate for &'a V
where
    V: Validate,
{
    type Invalidity = V::Invalidity;

    fn validate(&self) -> Result<Self::Invalidity> {
        (*self).validate()
    }
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

/// Validate all elements of a slice
impl<V> Validate for [V]
where
    V: Validate,
{
    type Invalidity = V::Invalidity;

    fn validate(&self) -> Result<Self::Invalidity> {
        self.iter()
            .fold(Context::new(), |ctx, elem| ctx.validate(elem))
            .into()
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
///
/// # Example
/// ```
/// # use semval::prelude::*;
///
/// struct UnvalidatedEmail(String);
///
/// struct Email(String);
///
/// #[derive(Debug)]
/// enum EmailInvalidity {
///     MinLength,
///     Format,
/// }
///
/// impl Validate for Email {
///     type Invalidity = EmailInvalidity;
/// #
///     fn validate(&self) -> ValidationResult<Self::Invalidity> {
///         // ...custom implementation...
/// #        ValidationContext::new()
/// #            .invalidate_if(
/// #                self.0.len() < 3,
/// #                EmailInvalidity::MinLength,
/// #            )
/// #            .invalidate_if(
/// #                self.0.chars().filter(|c| *c == '@').count() != 1,
/// #                EmailInvalidity::Format,
/// #            )
/// #            .into()
///     }
/// }
///
/// impl ValidatedFrom<UnvalidatedEmail> for Email {
///     fn validated_from(from: UnvalidatedEmail) -> ValidatedResult<Self> {
///         // 1st step: Convert value
///         let from = Email(from.0);
///         // 2nd step: Validate converted value
///         if let Err(err) = from.validate() {
///             Err((from, err))
///         } else {
///             Ok(from)
///         }
///     }
/// }
///
/// let unvalidated_email = UnvalidatedEmail("test@example.com".to_string());
/// match Email::validated_from(unvalidated_email) {
///     Ok(email) => println!("Valid e-mail address: {}", email.0),
///     Err((email, context)) => println!("Invalid e-mail address: {} {:?}", email.0, context),
/// }
/// ```
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

    struct Dummy {
        is_valid: bool,
    }

    impl Dummy {
        pub fn valid() -> Self {
            Self { is_valid: true }
        }

        pub fn invalid() -> Self {
            Self { is_valid: false }
        }
    }

    impl Validate for Dummy {
        type Invalidity = ();

        fn validate(&self) -> Result<Self::Invalidity> {
            Context::new().invalidate_if(!self.is_valid, ()).into()
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
    fn validate_option_ref_none() {
        assert!((None as Option<&AlwaysValid>).validate().is_ok());
        assert!((None as Option<&AlwaysInvalid>).validate().is_ok());
    }

    #[test]
    fn validate_option_some() {
        assert!(Some(AlwaysValid).validate().is_ok());
        assert!(Some(AlwaysInvalid).validate().is_err());
    }

    #[test]
    fn validate_option_ref_some() {
        assert!(Some(&AlwaysValid).validate().is_ok());
        assert!(Some(&AlwaysInvalid).validate().is_err());
    }

    #[test]
    fn validate_slices() {
        assert!(vec![Dummy::valid(), Dummy::valid()].validate().is_ok());
        assert_eq!(
            1,
            vec![Dummy::valid(), Dummy::invalid()]
                .validate()
                .unwrap_err()
                .into_iter()
                .count()
        );
        assert_eq!(
            1,
            vec![Dummy::invalid(), Dummy::valid()]
                .validate()
                .unwrap_err()
                .into_iter()
                .count()
        );
        assert_eq!(
            2,
            vec![Dummy::invalid(), Dummy::invalid()]
                .validate()
                .unwrap_err()
                .into_iter()
                .count()
        );
    }

    #[test]
    fn validate_slices_ref() {
        let valid = Dummy::valid();
        let invalid = Dummy::invalid();
        assert!(vec![&valid, &valid].validate().is_ok());
        assert_eq!(
            1,
            vec![&valid, &invalid]
                .validate()
                .unwrap_err()
                .into_iter()
                .count()
        );
        assert_eq!(
            1,
            vec![&invalid, &valid]
                .validate()
                .unwrap_err()
                .into_iter()
                .count()
        );
        assert_eq!(
            2,
            vec![&invalid, &invalid]
                .validate()
                .unwrap_err()
                .into_iter()
                .count()
        );
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

    #[test]
    fn is_valid() {
        assert!(AlwaysValid.is_valid());
        assert!(!AlwaysInvalid.is_valid());
    }
}
