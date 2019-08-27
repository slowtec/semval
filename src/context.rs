use super::*;

use core::fmt::{self, Display, Formatter};

use std::error::Error as StdError;

use smallvec::SmallVec;

const SMALLVEC_ARRAY_LEN: usize = 8;

type SmallVecArray<V> = [V; SMALLVEC_ARRAY_LEN];

/// A collection of violations resulting from a validation
///
/// Collects violations that are detected while performing
/// a validation.
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Context<V>
where
    V: Validation,
{
    violations: SmallVec<SmallVecArray<V>>,
}

#[cfg(feature = "std")]
impl<V> StdError for Context<V> where V: Validation + Display {}

impl<V> Display for Context<V>
where
    V: Validation + Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("[")?;
        for (i, v) in self.violations().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{}", v)?;
        }
        f.write_str("]")?;
        Ok(())
    }
}

impl<V> Default for Context<V>
where
    V: Validation,
{
    fn default() -> Self {
        Self::valid()
    }
}

impl<V> Context<V>
where
    V: Validation,
{
    /// Create a new valid context without any violations.
    #[inline]
    pub fn valid() -> Self {
        Self {
            violations: SmallVec::new(),
        }
    }

    /// The violations collected so far
    #[inline]
    pub fn violations(&self) -> impl Iterator<Item = &V> {
        self.violations.iter()
    }

    /// Check if the context already has any violations
    #[inline]
    pub fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Count the number of violations collected so far
    #[inline]
    pub fn count_violations(&self) -> usize {
        self.violations.len()
    }

    /// Add a new violation to the context
    #[inline]
    pub fn add_violation(&mut self, violation: impl Into<V>) {
        self.violations.push(violation.into());
    }

    /// Conditionally add a new violation to the context
    #[inline]
    pub fn add_violation_if(&mut self, cond: bool, violation: impl Into<V>) {
        if cond {
            self.add_violation(violation);
        }
    }

    /// Merge with another context
    fn merge_violations(&mut self, other: Self) {
        self.violations.reserve(other.violations.len());
        for error in other.violations.into_iter() {
            self.violations.push(error);
        }
    }

    /// Merge a validation result into the context
    ///
    /// TODO: This is supposed to become a non-public function.
    /// Use use `validate()` instead.
    #[inline]
    #[deprecated]
    pub fn merge_result(&mut self, res: Result<V>) {
        if let Err(other) = res {
            self.merge_violations(other);
        }
    }

    /// Merge an unrelated validation into the context
    ///
    /// TODO: This is supposed to become a non-public function.
    /// Use use `validate_and_map()` instead.
    #[deprecated]
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

    /// Validate the target tand merge the result into this context
    #[inline]
    pub fn validate(&mut self, target: &impl Validate<Validation = V>) {
        #[allow(deprecated)]
        self.merge_result(target.validate());
    }

    /// Validate the target tand merge the result after mapping into this context
    #[inline]
    pub fn validate_and_map<F, U>(&mut self, target: &impl Validate<Validation = U>, map: F) where
        F: Fn(U) -> V,
        U: Validation,
    {
        #[allow(deprecated)]
        self.map_and_merge_result(target.validate(), map);
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

impl<V> Into<Result<V>> for Context<V>
where
    V: Validation,
{
    fn into(self) -> Result<V> {
        self.into_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_context() {
        let context = Context::<()>::valid();
        assert!(!context.has_violations());
        assert_eq!(0, context.count_violations());
        assert!(context.into_result().is_ok());
    }

    #[test]
    fn default_context() {
        assert_eq!(Context::<()>::valid(), Context::<()>::default());
    }

    #[test]
    fn add_error() {
        let mut context = Context::<()>::valid();
        assert!(!context.has_violations());
        for _ in 0..=SMALLVEC_ARRAY_LEN {
            let violations_before = context.count_violations();
            context.add_violation(());
            assert!(context.has_violations());
            let violations_after = context.count_violations();
            assert_eq!(violations_after, violations_before + 1);
        }
        assert_eq!(SMALLVEC_ARRAY_LEN + 1, context.count_violations());
        assert_eq!(context.count_violations(), context.violations().count());
        assert!(context.into_result().is_err());
    }
}
