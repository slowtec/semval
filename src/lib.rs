#![cfg_attr(not(feature = "std"), no_std)]

pub mod context;
pub mod error;

pub mod prelude {
    pub use super::{
        error::Error as ValidationError, context::Context as ValidationContext,
        Result as ValidationResult, Validate, Validation, Validity,
    };
}

use context::Context;

use core::{any::Any, fmt::{self, Debug, Display, Formatter}};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MinSize(pub usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MaxSize(pub usize);

/// Non-exaustive enumeration of reasons why a validation has failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
//#[non_exhaustive]
pub enum Validity {
    // Cardinality
    Missing,
    TooFew(MinSize),
    TooMany(MaxSize),

    // String length
    Empty,
    TooShort(MinSize),
    TooLong(MaxSize),

    // Value
    LowerBound,
    UpperBound,
    OutOfRange,

    // Generic
    Ambiguous,
    Inconsistent,
    Forbidden,
    Invalid,

    // Custom code for extensibility
    Custom(&'static str),
}

impl Validity {
    pub const fn min_size(min: usize) -> MinSize {
        MinSize(min)
    }

    pub const fn max_size(max: usize) -> MaxSize {
        MaxSize(max)
    }

    pub const fn too_few(min: usize) -> Self {
        Validity::TooFew(Self::min_size(min))
    }

    pub const fn too_many(max: usize) -> Self {
        Validity::TooMany(Self::max_size(max))
    }

    pub const fn too_short(min: usize) -> Self {
        Validity::TooShort(Self::min_size(min))
    }

    pub const fn too_long(max: usize) -> Self {
        Validity::TooLong(Self::max_size(max))
    }
}

impl Display for Validity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use Validity::*;
        match self {
            Missing => f.write_str("Missing"),
            TooFew(MinSize(size)) => write!(f, "TooFew({})", size),
            TooMany(MaxSize(size)) => write!(f, "TooMany({})", size),
            Empty => f.write_str("Empty"),
            TooShort(MinSize(size)) => write!(f, "TooShort({})", size),
            TooLong(MaxSize(size)) => write!(f, "TooLong({})", size),
            LowerBound => f.write_str("LowerBound"),
            UpperBound => f.write_str("UpperBound"),
            OutOfRange => f.write_str("OutOfRange"),
            Ambiguous => f.write_str("Ambiguous"),
            Inconsistent => f.write_str("Inconsistent"),
            Forbidden => f.write_str("Forbidden"),
            Invalid => f.write_str("Invalid"),
            Custom(code) => write!(f, "Custom({})", code),
        }
    }
}

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

    fn start_validation() -> Context<T> {
        Context::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    impl Validate<()> for Dummy {
        fn validate(&self) -> Result<()> {
            Self::start_validation().finish_validation()
        }
    }

    #[test]
    fn validate_dummy() {
        assert!(Dummy.validate().is_ok());
    }
}
