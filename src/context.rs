use super::*;

use crate::error::Error;

use core::fmt::{self, Display, Formatter};

use std::error::Error as StdError;

use smallvec::{smallvec, SmallVec};

const SMALLVEC_ERROR_ARRAY_LEN: usize = 8;

type SmallVecErrorArray<T> = [Error<T>; SMALLVEC_ERROR_ARRAY_LEN];

/// A validation context for collecting validation errors.
#[derive(Clone, Debug)]
pub struct Context<T>
where
    T: Validation,
{
    errors: SmallVec<SmallVecErrorArray<T>>,
}

#[cfg(feature = "std")]
impl<T> StdError for Context<T> where T: Validation {}

impl<T> Display for Context<T>
where
    T: Validation,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}", self)
    }
}

impl<T> Context<T>
where
    T: Validation,
{
    pub(crate) fn new() -> Self {
        Self {
            errors: smallvec![],
        }
    }

    /// Check if the context has errors.
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Count the number of errors in the context.
    pub fn count_errors(&self) -> usize {
        self.errors.len()
    }

    /// Add a new error to the context.
    pub fn add_error(&mut self, validation: impl Into<T>, validity: impl Into<Validity>) {
        self.errors.push(Error::new(validation, validity));
    }

    /// Merge with another context.
    pub fn merge_errors(&mut self, other: Self) {
        self.errors.reserve(other.errors.len());
        for error in other.errors.into_iter() {
            self.errors.push(error);
        }
    }

    /// Merge with a validation result.
    pub fn merge_result(&mut self, res: Result<T>) {
        if let Err(other) = res {
            self.merge_errors(other);
        }
    }

    /// Merge with an incompatible validation result.
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

    /// Finish the current validation by wrapping the context in a result.
    pub fn finish_validation(self) -> Result<T> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl<T> IntoIterator for Context<T>
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
    fn default_context() {
        let context = Context::<()>::new();
        assert!(!context.has_errors());
        assert_eq!(0, context.count_errors());
        assert!(context.finish_validation().is_ok());
    }

    #[test]
    fn add_error() {
        let mut context = Context::<()>::new();
        assert!(!context.has_errors());
        for i in 0..10 {
            let errors_before = context.count_errors();
            if i % 2 == 0 {
                context.add_error((), Validity::too_few(i));
            } else {
                context.add_error((), Validity::too_many(i));
            }
            assert!(context.has_errors());
            let errors_after = context.count_errors();
            assert_eq!(errors_after, errors_before + 1);
        }
    }
}
