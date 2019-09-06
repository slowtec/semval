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
