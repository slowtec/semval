use core::iter::once;

use crate::{
    smallvec::SmallVec,
    util::{IsEmpty, Mergeable, MergeableSized},
    Invalidity, Validate, ValidationResult,
};

const SMALLVEC_ARRAY_LEN: usize = 8;

type SmallVecArray<V> = [V; SMALLVEC_ARRAY_LEN];

/// A collection of invalidities resulting from a validation
///
/// Collects invalidities that are detected while performing
/// a validation.
#[derive(Clone, Debug, Default)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Context<V>
where
    V: Invalidity,
{
    invalidities: SmallVec<SmallVecArray<V>>,
}

impl<V> IsEmpty for Context<V>
where
    V: Invalidity,
{
    fn is_empty(&self) -> bool {
        self.invalidities.is_empty()
    }
}

impl<V> Mergeable for Context<V>
where
    V: Invalidity,
{
    type Item = V;

    fn empty<H>(capacity_hint: H) -> Self
    where
        H: Into<Option<usize>>,
    {
        let invalidities = Mergeable::empty(capacity_hint);
        Self { invalidities }
    }

    fn merge(mut self, other: Self) -> Self {
        self.invalidities = self.invalidities.merge(other.invalidities);
        self
    }

    fn merge_iter<H, I>(mut self, count_hint: H, iter: I) -> Self
    where
        H: Into<Option<usize>>,
        I: Iterator<Item = Self::Item>,
    {
        self.invalidities = self.invalidities.merge_iter(count_hint, iter);
        self
    }
}

impl<V> MergeableSized for Context<V> where V: Invalidity {}

impl<V> Context<V>
where
    V: Invalidity,
{
    /// Create a new valid and empty context
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::empty(<SmallVecArray<V> as smallvec::Array>::size())
    }

    /// Check if the context is still valid
    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.is_empty()
    }

    /// Record a new invalidity within this context
    #[inline]
    #[must_use]
    pub fn invalidate(self, invalidity: impl Into<V>) -> Self {
        self.merge_iter(1, once(invalidity.into()))
    }

    /// Conditionally record a new invalidity within this context
    #[inline]
    #[must_use]
    pub fn invalidate_if(self, is_invalid: impl Into<bool>, invalidity: impl Into<V>) -> Self {
        if is_invalid.into() {
            self.invalidate(invalidity)
        } else {
            self
        }
    }

    /// Merge the results of another validation
    ///
    /// Needed for collecting results from custom validation functions.
    #[inline]
    #[must_use]
    pub fn merge_result(self, res: ValidationResult<V>) -> Self {
        if let Err(other) = res {
            self.merge(other)
        } else {
            self
        }
    }

    /// Merge the mapped results of another validation
    ///
    /// Needed for collecting results from custom validation functions.
    #[must_use]
    pub fn merge_result_with<F, U>(self, res: ValidationResult<U>, map: F) -> Self
    where
        F: Fn(U) -> V,
        U: Invalidity,
    {
        if let Err(other) = res {
            self.merge_exact_size_iter(other.invalidities.into_iter().map(map))
        } else {
            self
        }
    }

    /// Validate the target and merge the result into this context
    #[inline]
    #[must_use]
    pub fn validate<U>(self, target: &impl Validate<Invalidity = U>) -> Self
    where
        U: Invalidity + Into<V>,
    {
        self.validate_with(target, Into::into)
    }

    /// Validate the target and merge the mapped result into this context
    #[inline]
    #[must_use]
    pub fn validate_with<F, U>(self, target: &impl Validate<Invalidity = U>, map: F) -> Self
    where
        F: Fn(U) -> V,
        U: Invalidity,
    {
        self.merge_result_with(target.validate(), map)
    }

    /// Finish the validation
    ///
    /// Finishes the current validation of this context with a result.
    ///
    /// # Errors
    ///
    /// Returns `Err` with the collected invalidities if one or more
    /// validations failed.
    #[inline]
    pub fn into_result(self) -> ValidationResult<V> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl<V> From<Context<V>> for ValidationResult<V>
where
    V: Invalidity,
{
    fn from(from: Context<V>) -> Self {
        from.into_result()
    }
}

/// Transform the validation context into an iterator
/// that yields all the collected invalidities.
impl<V> IntoIterator for Context<V>
where
    V: Invalidity,
{
    type Item = V;
    // TODO: Replace with an opaque, existential type eventually (if ever possible):
    // type IntoIter = impl Iterator<V>;
    type IntoIter = smallvec::IntoIter<SmallVecArray<V>>;

    fn into_iter(self) -> Self::IntoIter {
        self.invalidities.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_context() {
        let context = Context::<()>::new();
        assert!(context.is_empty());
        assert!(context.is_valid());
        assert!(context.into_result().is_ok());
    }

    #[test]
    fn default_context() {
        assert_eq!(Context::<()>::new(), Context::<()>::default());
    }

    #[test]
    fn invalidate() {
        let mut context = Context::<()>::new();
        assert!(context.is_empty());
        assert!(context.is_valid());
        for _ in 0..=SMALLVEC_ARRAY_LEN {
            let invalidities_before = context.invalidities.len();
            context = context.invalidate(());
            assert!(!context.is_empty());
            assert!(!context.is_valid());
            let invalidities_after = context.invalidities.len();
            assert_eq!(invalidities_after, invalidities_before + 1);
        }
        assert_eq!(SMALLVEC_ARRAY_LEN + 1, context.invalidities.len());
        assert!(context.into_result().is_err());
    }
}
