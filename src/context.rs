use super::*;

use core::fmt::{self, Display, Formatter};

use std::error::Error as StdError;

use smallvec::{smallvec, SmallVec};

const SMALLVEC_ARRAY_LEN: usize = 8;

type SmallVecArray<V> = [V; SMALLVEC_ARRAY_LEN];

/// A collection of violations resulting from a validation
///
/// Collects violations that are detected while performing
/// a validation.
#[derive(Clone, Debug)]
pub struct Context<V>
where
    V: Validation,
{
    violations: SmallVec<SmallVecArray<V>>,
}

#[cfg(feature = "std")]
impl<V> StdError for Context<V> where V: Validation {}

impl<V> Display for Context<V>
where
    V: Validation,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // TODO
        write!(f, "{:?}", self)
    }
}

impl<V> Default for Context<V>
where
    V: Validation,
{
    fn default() -> Self {
         Self {
            violations: smallvec![],
        }
   }
}

impl<V> Context<V>
where
    V: Validation,
{
    /// The violations collected so far
    pub fn violations(&self) -> impl Iterator<Item = &V> {
        self.violations.iter()
    }

    /// Check if the context already has any violations
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Count the number of violations collected so far
    pub fn count_violations(&self) -> usize {
        self.violations.len()
    }

    /// Add a new violation to the context
    pub fn add_violation(&mut self, violation: impl Into<V>) {
        self.violations.push(violation.into());
    }

    /// Merge with another context
    fn merge_violations(&mut self, other: Self) {
        self.violations.reserve(other.violations.len());
        for error in other.violations.into_iter() {
            self.violations.push(error);
        }
    }

    /// Merge a validation result into the context
    pub fn merge_result(&mut self, res: Result<V>) {
        if let Err(other) = res {
            self.merge_violations(other);
        }
    }

    /// Merge an unrelated validation into the context
    pub fn map_and_merge_result<F, U>(&mut self, res: Result<U>, map: F)
    where
        F: Fn(U) -> V,
        U: Validation,
    {
        if let Err(other) = res {
            self.violations.reserve(other.violations.len());
            for v in other.violations.into_iter() {
                self.violations.push(map(v))
            }
        }
    }

    /// Finish the current validation with a result
    ///
    /// Returns `Err` with the context's violations or `Ok` if
    /// the context does not have any violations.
    pub fn into_result(self) -> Result<V> {
        if self.violations.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl<V> IntoIterator for Context<V>
where
    V: Validation,
{
    type Item = V;
    type IntoIter = smallvec::IntoIter<SmallVecArray<V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.violations.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_context() {
        let context = Context::<()>::default();
        assert!(!context.has_violations());
        assert_eq!(0, context.count_violations());
        assert!(context.into_result().is_ok());
    }

    #[test]
    fn add_error() {
        let mut context = Context::<()>::default();
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
