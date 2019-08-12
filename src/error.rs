use super::*;

use core::fmt::{self, Display, Formatter};

use std::error::Error as StdError;

#[derive(Clone, Debug)]
pub struct Error<T>
where
    T: Validation,
{
    /// The validation context.
    pub validation: T,

    /// The actual cause of this error.
    pub validity: Validity,
}

impl<T> Error<T>
where
    T: Validation,
{
    pub(crate) fn new(validation: impl Into<T>, validity: impl Into<Validity>) -> Self {
        Self {
            validation: validation.into(),
            validity: validity.into(),
        }
    }

    pub(crate) fn map_validation<F, U>(self, map: &F) -> Error<U>
    where
        F: Fn(T) -> U,
        U: Validation,
    {
        Error {
            validation: map(self.validation),
            validity: self.validity,
        }
    }
}

impl<T> Display for Error<T>
where
    T: Validation,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}: {}", self.validation, self.validity)
    }
}

#[cfg(feature = "std")]
impl<T> StdError for Error<T> where T: Validation {}
