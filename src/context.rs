use super::*;

use smallvec::SmallVec;

const SMALLVEC_ARRAY_LEN: usize = 8;

type SmallVecArray<V> = [V; SMALLVEC_ARRAY_LEN];

/// A collection of invalidities resulting from a validation
///
/// Collects invalidities that are detected while performing
/// a validation.
#[derive(Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Context<V>
where
    V: Invalidity,
{
    invalidities: SmallVec<SmallVecArray<V>>,
}

impl<V> Context<V>
where
    V: Invalidity,
{
    /// Create a new valid and empty context
    #[inline]
    pub fn new() -> Self {
        Self {
            invalidities: SmallVec::new(),
        }
    }

    /// Check if the context is still valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.invalidities.is_empty()
    }

    /// Record a new invalidity within this context
    #[inline]
    pub fn invalidate(mut self, invalidity: impl Into<V>) -> Self {
        self.invalidities.push(invalidity.into());
        self
    }

    /// Conditionally record a new invalidity within this context
    #[inline]
    pub fn invalidate_if(self, is_violated: bool, invalidity: impl Into<V>) -> Self {
        if is_violated {
            self.invalidate(invalidity)
        } else {
            self
        }
    }

    // TODO: Make public?
    fn merge(mut self, other: Self) -> Self {
        self.invalidities.reserve(other.invalidities.len());
        for error in other.invalidities.into_iter() {
            self.invalidities.push(error);
        }
        self
    }

    /// Merge the results of another validation
    ///
    /// Needed for collecting results from custom validation functions.
    #[inline]
    pub fn merge_result(self, res: Result<V>) -> Self {
        if let Err(other) = res {
            self.merge(other)
        } else {
            self
        }
    }

    /// Merge the mapped results of another validation
    ///
    /// Needed for collecting results from custom validation functions.
    pub fn map_and_merge_result<F, U>(mut self, res: Result<U>, map: F) -> Self
    where
        F: Fn(U) -> V,
        U: Invalidity,
    {
        if let Err(other) = res {
            self.invalidities.reserve(other.invalidities.len());
            for v in other.invalidities.into_iter() {
                self.invalidities.push(map(v))
            }
        }
        self
    }

    /// Validate the target and merge the result into this context
    #[inline]
    pub fn validate<U>(self, target: &impl Validate<Invalidity = U>) -> Self
    where
        U: Invalidity + Into<V>,
    {
        self.map_and_merge_result(target.validate(), |u| u.into())
    }

    /// Validate the target, map the result, and merge it into this context
    #[inline]
    pub fn validate_and_map<F, U>(self, target: &impl Validate<Invalidity = U>, map: F) -> Self
    where
        F: Fn(U) -> V,
        U: Invalidity,
    {
        self.map_and_merge_result(target.validate(), map)
    }

    /// Finish the current validation of this context with a result
    #[inline]
    pub fn into_result(self) -> Result<V> {
        if self.invalidities.is_empty() {
            Ok(())
        } else {
            Err(self)
        }
    }
}

impl<V> Default for Context<V>
where
    V: Invalidity,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Into<Result<V>> for Context<V>
where
    V: Invalidity,
{
    fn into(self) -> Result<V> {
        self.into_result()
    }
}

/// Transform the validation context into an iterator
/// that yields all the collected invalidities.
impl<V> IntoIterator for Context<V>
where
    V: Invalidity,
{
    type Item = V;
    // TODO: Replace with an opaque, existantial type eventually (if ever possible):
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
        assert!(context.is_valid());
        assert!(context.invalidities.is_empty());
        assert!(context.into_result().is_ok());
    }

    #[test]
    fn default_context() {
        assert_eq!(Context::<()>::new(), Context::<()>::default());
    }

    #[test]
    fn invalidate() {
        let mut context = Context::<()>::new();
        assert!(context.is_valid());
        for _ in 0..=SMALLVEC_ARRAY_LEN {
            let invalidities_before = context.invalidities.len();
            context = context.invalidate(());
            assert!(!context.is_valid());
            let invalidities_after = context.invalidities.len();
            assert_eq!(invalidities_after, invalidities_before + 1);
        }
        assert_eq!(SMALLVEC_ARRAY_LEN + 1, context.invalidities.len());
        assert!(context.into_result().is_err());
    }
}
