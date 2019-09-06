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

///////////////////////////////////////////////////////////////////////////////
/// Mergeable
///////////////////////////////////////////////////////////////////////////////

pub(crate) trait Mergeable {
    type Item;

    fn merge(self, other: Self) -> Self;

    fn from_iter<I, M>(reserve: usize, from_iter: I, map_from: M) -> Self
    where
        I: Iterator,
        M: Fn(<I as Iterator>::Item) -> Self::Item;

    fn merge_from_iter<I, M>(self, reserve: usize, from_iter: I, map_from: M) -> Self
    where
        I: Iterator,
        M: Fn(<I as Iterator>::Item) -> Self::Item;
}

/// Trivial implementation of `Mergeable` for the unit type `()`
impl Mergeable for () {
    type Item = ();

    fn merge(self, _: Self) -> Self {}

    fn from_iter<I, M>(_: usize, _: I, _: M) -> Self {}

    fn merge_from_iter<I, M>(self, _: usize, _: I, _: M) -> Self {}
}

///////////////////////////////////////////////////////////////////////////////
/// UnitResult
///////////////////////////////////////////////////////////////////////////////

/// A result with only an error and the unit type `()` on success
pub(crate) type UnitResult<E> = core::result::Result<(), E>;

///
impl<E> Mergeable for UnitResult<E>
where
    E: Mergeable + IsEmpty,
{
    type Item = E::Item;

    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Ok(()), Ok(())) => Ok(()),
            (Ok(()), Err(err)) => Err(err),
            (Err(err), Ok(())) => Err(err),
            (Err(e1), Err(e2)) => Err(e1.merge(e2)),
        }
    }

    fn from_iter<I, M>(reserve: usize, from_iter: I, map_from: M) -> Self
    where
        I: Iterator,
        M: Fn(<I as Iterator>::Item) -> Self::Item,
    {
        let e = E::from_iter(reserve, from_iter, map_from);
        if e.is_empty() {
            // No errors
            Ok(())
        } else {
            // One or more errors
            Err(e)
        }
    }

    fn merge_from_iter<I, M>(self, reserve: usize, from_iter: I, map_from: M) -> Self
    where
        I: Iterator,
        M: Fn(<I as Iterator>::Item) -> Self::Item,
    {
        match self {
            Ok(()) => Self::from_iter(reserve, from_iter, map_from),
            Err(e) => Err(e.merge_from_iter(reserve, from_iter, map_from)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl IsEmpty for usize {
        fn is_empty(&self) -> bool {
            *self == 0
        }
    }

    impl Mergeable for usize {
        type Item = usize;

        fn merge(self, other: Self) -> Self {
            self + other
        }

        fn from_iter<I, M>(_: usize, from_iter: I, map_from: M) -> Self
        where
            I: Iterator,
            M: Fn(<I as Iterator>::Item) -> Self::Item,
        {
            from_iter.fold(0, |sum, item| sum + map_from(item))
        }

        fn merge_from_iter<I, M>(self, _: usize, from_iter: I, map_from: M) -> Self
        where
            I: Iterator,
            M: Fn(<I as Iterator>::Item) -> Self::Item,
        {
            from_iter.fold(self, |sum, item| sum + map_from(item))
        }
    }

    #[test]
    fn unit_is_empty() {
        assert!(().is_empty())
    }

    #[test]
    fn unit_mergeable() {
        assert_eq!((), ().merge(()));
        assert_eq!((), <() as Mergeable>::from_iter(3, core::iter::repeat(()).take(3), core::convert::identity));
        assert_eq!((), ().merge_from_iter(3, core::iter::repeat(()).take(3), core::convert::identity));
    }

    #[test]
    fn unit_result() {
        assert_eq!(Ok(()) as UnitResult<usize>, Ok(()).merge(Ok(())));
        assert_eq!(Err(1usize), Err(1).merge(Ok(())));
        assert_eq!(Err(2usize), Ok(()).merge(Err(2)));
        assert_eq!(Err(3usize), Err(1).merge(Err(2)));
    }
}
