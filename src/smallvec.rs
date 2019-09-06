/// Trait implementations for smallvec types
use crate::util::*;

/// Re-exports
pub(crate) use smallvec::{Array, IntoIter, SmallVec};

impl<A> IsEmpty for smallvec::SmallVec<A>
where
    A: smallvec::Array,
{
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<A> Mergeable for smallvec::SmallVec<A>
where
    A: smallvec::Array,
{
    type Item = A::Item;

    fn empty<H>(capacity_hint: H) -> Self
    where
        H: Into<Option<usize>>,
    {
        let capacity = capacity_hint.into().unwrap_or(0);
        if capacity > 0 {
            Self::with_capacity(capacity)
        } else {
            Default::default()
        }
    }

    fn merge(self, other: Self) -> Self {
        // Reuse the instance with greater capacity for accumulation (this)
        // and consume (= drain & drop) the other one (that).
        let (that, mut this) = if self.capacity() < other.capacity() {
            (self, other)
        } else {
            (other, self)
        };
        this.reserve(that.len());
        this.insert_many(this.len(), that.into_iter());
        this
    }

    fn merge_iter<H, I>(mut self, count_hint: H, iter: I) -> Self
    where
        H: Into<Option<usize>>,
        I: Iterator<Item = Self::Item>,
    {
        self.reserve(count_hint.into().unwrap_or(0));
        self.insert_many(self.len(), iter);
        self
    }
}
