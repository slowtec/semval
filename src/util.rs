/// Traits and utilities for internal usage

///////////////////////////////////////////////////////////////////////////////
/// IsEmpty
///////////////////////////////////////////////////////////////////////////////

// TODO: Reuse from https://github.com/Stebalien/tool-rs?
pub(crate) trait IsEmpty {
    fn is_empty(&self) -> bool;
}

/// Trivial implementation of `IsEmpty` for the unit type `()`
impl IsEmpty for () {
    fn is_empty(&self) -> bool {
        true
    }
}

/// Trivial implementation of `IsEmpty` for quantities
impl IsEmpty for usize {
    fn is_empty(&self) -> bool {
        *self == 0
    }
}

///////////////////////////////////////////////////////////////////////////////
/// Mergeable
///////////////////////////////////////////////////////////////////////////////

/// A monoid for collecting or accumulating items
pub(crate) trait Mergeable {
    type Item;

    /// Create an empty instance
    ///
    /// The optional `capacity_hint` parameter allows to provide a best guess
    /// for the initial capacity, depending on how many items are expected
    /// to be accumulated.
    fn empty<H>(capacity_hint: H) -> Self
    where
        H: Into<Option<usize>>;

    /// Consuming combine operation that merges this with another instance
    fn merge(self, other: Self) -> Self;

    /// Consuming combine operation that merges this instance with the
    /// items generated by an iterator
    ///
    /// The `reserve_hint` parameter allows to provide a hint how much additional
    /// capacity should be reserved for the generated items, e.g. it could
    /// be a preliminary guess or context knowledge how many items the iterator
    /// is supposed to generate.
    fn merge_iter<H, I>(self, reserve_hint: H, iter: I) -> Self
    where
        H: Into<Option<usize>>,
        I: Iterator<Item = Self::Item>;
}

/// Trivial implementation of `Mergeable` for the unit type `()`
impl Mergeable for () {
    type Item = ();

    fn empty<H>(_: H) -> Self {}

    fn merge(self, _: Self) -> Self {}

    fn merge_iter<I, H>(self, _: I, _: H) -> Self {}
}

/// Trivial implementation of `Mergeable` for quantities
impl Mergeable for usize {
    type Item = usize;

    fn empty<H>(_: H) -> Self {
        0
    }

    fn merge(self, other: Self) -> Self {
        self + other
    }

    fn merge_iter<H, I>(self, _: H, iter: I) -> Self
    where
        I: Iterator<Item = Self::Item>,
    {
        iter.fold(self, |sum, item| sum + item)
    }
}

///////////////////////////////////////////////////////////////////////////////
/// UnitResult
///////////////////////////////////////////////////////////////////////////////

/// A result with only an error and the unit type `()` on success
pub(crate) type UnitResult<E> = core::result::Result<(), E>;

impl<E> Mergeable for UnitResult<E>
where
    E: Mergeable + IsEmpty,
{
    type Item = E::Item;

    fn empty<H>(_: H) -> Self {
        Ok(())
    }

    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(err)) => Err(err),
            (Err(err), Ok(())) => Err(err),
            (Err(e1), Err(e2)) => Err(e1.merge(e2)),
        }
    }

    fn merge_iter<H, I>(self, reserve_hint: H, iter: I) -> Self
    where
        H: Into<Option<usize>>,
        I: Iterator<Item = Self::Item>,
    {
        let reserve_hint = reserve_hint.into();
        match self {
            Ok(()) => Self::empty(reserve_hint).merge_iter(reserve_hint, iter),
            Err(e) => Err(e.merge_iter(reserve_hint, iter)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_is_empty() {
        assert!(().is_empty())
    }

    #[test]
    fn unit_mergeable() {
        assert_eq!((), <() as Mergeable>::empty(5));
        assert_eq!((), ().merge(()));
        assert_eq!((), ().merge_iter(3, core::iter::repeat(()).take(3)));
    }

    #[test]
    fn unit_result_quantities() {
        assert_eq!(Ok(()) as UnitResult<usize>, Ok(()).merge(Ok(())));
        assert_eq!(Err(1usize), Err(1).merge(Ok(())));
        assert_eq!(Err(2usize), Ok(()).merge(Err(2)));
        assert_eq!(Err(3usize), Err(1).merge(Err(2)));
    }

    #[cfg(feature = "std")]
    impl<T> IsEmpty for Vec<T> {
        fn is_empty(&self) -> bool {
            self.is_empty()
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn vec_is_empty() {
        assert!(<Vec<()> as IsEmpty>::is_empty(&vec![]));
    }

    #[cfg(feature = "std")]
    impl<T> Mergeable for Vec<T> {
        type Item = T;

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
            let (mut that, mut this) = if self.capacity() < other.capacity() {
                (self, other)
            } else {
                (other, self)
            };
            this.append(&mut that);
            this
        }

        fn merge_iter<H, I>(mut self, reserve_hint: H, iter: I) -> Self
        where
            H: Into<Option<usize>>,
            I: Iterator<Item = Self::Item>,
        {
            self.reserve(reserve_hint.into().unwrap_or(0));
            iter.fold(self, |mut this, item| {
                this.push(item);
                this
            })
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn vec_mergeable() {
        assert!(<Vec<()> as Mergeable>::empty(None).is_empty());
        assert!(<Vec<()> as Mergeable>::empty(5).is_empty());
        assert_eq!(vec![1, 2], vec![].merge(vec![1, 2]));
        assert_eq!(vec![1, 2], vec![1, 2].merge(vec![]));
        assert_eq!(vec![2, 1], vec![2].merge(vec![1]));
        assert_eq!(vec![2, 3, 1, 4], vec![2, 3].merge(vec![1, 4]));
        assert_eq!(vec![0, 2, 3, 1, 4], vec![0].merge_iter(4, vec![2, 3, 1, 4].into_iter()));
    }
}
