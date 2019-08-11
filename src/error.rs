use super::*;

use core::fmt::{self, Display, Formatter};

use std::error::Error as StdError;

use smallvec::{smallvec, SmallVec};

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
    fn new(validation: impl Into<T>, validity: impl Into<Validity>) -> Self {
        Self {
            validation: validation.into(),
            validity: validity.into(),
        }
    }

    fn map_validation<F, U>(self, map: &F) -> Error<U>
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

const SMALLVEC_ERROR_ARRAY_LEN: usize = 8;

type SmallVecErrorArray<T> = [Error<T>; SMALLVEC_ERROR_ARRAY_LEN];

/// A collection of validation errors.
#[derive(Clone, Debug)]
pub struct Errors<T>
where
    T: Validation,
{
    errors: SmallVec<SmallVecErrorArray<T>>,
}

impl<T> Default for Errors<T>
where
    T: Validation,
{
    fn default() -> Self {
        Self {
            errors: smallvec![],
        }
    }
}

#[cfg(feature = "std")]
impl<T> StdError for Errors<T> where T: Validation {}

impl<T> Display for Errors<T>
where
    T: Validation,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}", self)
    }
}

impl<T> Errors<T>
where
    T: Validation,
{
    /// Check if the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Count the number of errors in this collection.
    pub fn count(&self) -> usize {
        self.errors.len()
    }

    /// Create a new collection with a single error.
    pub fn error(validation: impl Into<T>, validity: impl Into<Validity>) -> Self {
        Self {
            errors: smallvec![Error::new(validation, validity)],
        }
    }

    /// Add a new error to this collection.
    pub fn add_error(&mut self, validation: impl Into<T>, validity: impl Into<Validity>) {
        self.errors.push(Error::new(validation, validity));
    }

    /// Merge and clear another collection of errors into this collection.
    pub fn merge_errors(&mut self, other: Self) {
        self.errors.reserve(other.errors.len());
        for error in other.errors.into_iter() {
            self.errors.push(error);
        }
    }

    /// Merge a validation result into this collection.
    pub fn merge_result(&mut self, res: Result<T>) {
        if let Err(other) = res {
            self.merge_errors(other);
        }
    }

    /// Merge errors from an incompatible validation result into this
    /// collection. The mapping will typically result in wrapping the
    /// foreign validation context.
    pub fn map_and_merge_result<F, U>(&mut self, res: Result<U>, map: F)
    where
        F: Fn(U) -> T,
        U: Validation,
    {
        if let Err(other) = res {
            self.errors.reserve(other.errors.len());
            for e in other.errors.into_iter() {
                self.errors.push(e.map_validation(&map))
            }
        }
    }

    /// Convert this collection into a validation result.
    pub fn into_result(self) -> Result<T> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl<T> IntoIterator for Errors<T>
where
    T: Validation,
{
    type Item = Error<T>;
    type IntoIter = smallvec::IntoIter<SmallVecErrorArray<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.errors.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_errors() {
        let errors = Errors::<()>::default();
        assert!(errors.is_empty());
        assert_eq!(0, errors.count());
        assert!(errors.into_result().is_ok());
    }

    #[test]
    fn single_error() {
        let errors = Errors::<()>::error((), Validity::Invalid);
        assert!(!errors.is_empty());
        assert_eq!(1, errors.count());
        assert!(errors.into_result().is_err());
    }

    #[test]
    fn add_error() {
        let mut errors = Errors::<()>::default();
        for i in 0..10 {
            let count_before = errors.count();
            if i % 2 == 0 {
                errors.add_error((), Validity::too_few(i));
            } else {
                errors.add_error((), Validity::too_many(i));
            }
            let count_after = errors.count();
            assert_eq!(count_before + 1, count_after);
        }
    }
}
