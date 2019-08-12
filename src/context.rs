use super::*;

use core::fmt::{self, Display, Formatter};

use std::error::Error as StdError;

use smallvec::{smallvec, SmallVec};

const SMALLVEC_ARRAY_LEN: usize = 8;

type SmallVecArray<T> = [T; SMALLVEC_ARRAY_LEN];

/// A validation context for collecting validation violations.
#[derive(Clone, Debug)]
pub struct Context<T>
where
    T: Validation,
{
    violations: SmallVec<SmallVecArray<T>>,
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
            violations: smallvec![],
        }
    }

    /// Check if the context has violations.
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Count the number of violations in the context.
    pub fn count_violations(&self) -> usize {
        self.violations.len()
    }

    /// Add a new violation to the context.
    pub fn add_violation(&mut self, violation: impl Into<T>) {
        self.violations.push(violation.into());
    }

    /// Merge with another context.
    pub fn merge_violations(&mut self, other: Self) {
        self.violations.reserve(other.violations.len());
        for error in other.violations.into_iter() {
            self.violations.push(error);
        }
    }

    /// Merge with a validation result.
    pub fn merge_result(&mut self, res: Result<T>) {
        if let Err(other) = res {
            self.merge_violations(other);
        }
    }

    /// Merge with an incompatible validation result.
    pub fn map_and_merge_result<F, V>(&mut self, res: Result<V>, map: F)
    where
        F: Fn(V) -> T,
        V: Validation,
    {
        if let Err(other) = res {
            self.violations.reserve(other.violations.len());
            for v in other.violations.into_iter() {
                self.violations.push(map(v))
            }
        }
    }

    /// Finish the current validation by wrapping the context in a result.
    pub fn finish_validation(self) -> Result<T> {
        if self.violations.is_empty() {
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
    type Item = T;
    type IntoIter = smallvec::IntoIter<SmallVecArray<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.violations.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_context() {
        let context = Context::<()>::new();
        assert!(!context.has_violations());
        assert_eq!(0, context.count_violations());
        assert!(context.finish_validation().is_ok());
    }

    #[test]
    fn add_error() {
        let mut context = Context::<()>::new();
        assert!(!context.has_violations());
        for _ in 0..=SMALLVEC_ARRAY_LEN + 1 {
            let violations_before = context.count_violations();
            context.add_violation(());
            assert!(context.has_violations());
            let violations_after = context.count_violations();
            assert_eq!(violations_after, violations_before + 1);
        }
    }
}
