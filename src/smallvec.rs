/// Trait implementations for smallvec types
use crate::util::*;

/// Re-exports
pub(crate) use smallvec::{IntoIter, SmallVec};

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

    fn merge(self, other: Self) -> Self {
        // Reuse the instance with greater capacity for accumulation (sink)
        // and consume (= drain & drop) the other one (source).
        let (source, mut sink) = if self.capacity() < other.capacity() {
            (self, other)
        } else {
            (other, self)
        };
        sink.reserve(source.len());
        sink.insert_many(sink.len(), source.into_iter());
        sink
    }

    fn from_iter<I, M>(reserve: usize, from_iter: I, map_from: M) -> Self
    where
        I: Iterator,
        M: Fn(<I as Iterator>::Item) -> Self::Item,
    {
        let mut new = Self::with_capacity(reserve);
        new.insert_many(new.len(), from_iter.map(map_from));
        new
    }

    fn merge_from_iter<I, M>(mut self, reserve: usize, from_iter: I, map_from: M) -> Self
    where
        I: Iterator,
        M: Fn(<I as Iterator>::Item) -> Self::Item,
    {
        self.reserve(reserve);
        self.insert_many(self.len(), from_iter.map(map_from));
        self
    }
}
