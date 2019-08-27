use super::*;

use core::fmt::{self, Display, Formatter};

use std::error::Error as StdError;

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
    pub fn valid() -> Self {
        Self {
            invalidities: SmallVec::new(),
        }
    }

    /// Check if the context is still valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.invalidities.is_empty()
    }

    /// Record a new invalidity within this context
    #[inline]
    pub fn invalidate(&mut self, invalidity: impl Into<V>) {
        self.invalidities.push(invalidity.into());
    }

    /// Conditionally record a new invalidity within this context
    #[inline]
    pub fn invalidate_if(&mut self, is_violated: bool, invalidity: impl Into<V>) {
        if is_violated {
            self.invalidate(invalidity);
        }
    }

    // TODO: Make public?
    fn merge(&mut self, other: Self) {
        self.invalidities.reserve(other.invalidities.len());
        for error in other.invalidities.into_iter() {
            self.invalidities.push(error);
        }
    }

    /// Merge the results of another validation
    #[inline]
    pub fn merge_result(&mut self, res: Result<V>) {
        if let Err(other) = res {
            self.merge(other);
        }
    }

    /// Merge the mapped results of another validation
    pub fn map_and_merge_result<F, U>(&mut self, res: Result<U>, map: F)
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
    }

    /// Validate the target and merge the result into this context
    #[inline]
    pub fn validate(&mut self, target: &impl Validate<Invalidity = V>) {
        self.merge_result(target.validate());
    }

    /// Validate the target, map the result, and merge it into this context
    #[inline]
    pub fn validate_and_map<F, U>(&mut self, target: &impl Validate<Invalidity = U>, map: F) where
        F: Fn(U) -> V,
        U: Invalidity,
    {
        self.map_and_merge_result(target.validate(), map);
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
        Self::valid()
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

#[cfg(feature = "std")]
impl<V> StdError for Context<V> where V: Invalidity + Display {}

impl<V> Display for Context<V>
where
    V: Invalidity + Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str("[")?;
        for (i, v) in self.invalidities.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{}", v)?;
        }
        f.write_str("]")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_context() {
        let context = Context::<()>::valid();
        assert!(!context.is_valid());
        assert!(context.invalidities.is_empty());
        assert!(context.into_result().is_ok());
    }

    #[test]
    fn default_context() {
        assert_eq!(Context::<()>::valid(), Context::<()>::default());
    }

    #[test]
    fn invalidate() {
        let mut context = Context::<()>::valid();
        assert!(!context.is_valid());
        for _ in 0..=SMALLVEC_ARRAY_LEN {
            let invalidities_before = context.invalidities.len();
            context.invalidate(());
            assert!(context.is_valid());
            let invalidities_after = context.invalidities.len();
            assert_eq!(invalidities_after, invalidities_before + 1);
        }
        assert_eq!(SMALLVEC_ARRAY_LEN + 1, context.invalidities.len());
        assert!(context.into_result().is_err());
    }
}
