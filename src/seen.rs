//! Tracking whether a value is newly discovered or previously seen.
//!
//! This module provides the [`Seen`] enum for distinguishing between values
//! that are encountered for the first time versus those that have been seen before.

/// An enum to track whether a value is newly discovered or previously seen.
///
/// This is commonly used in graph traversal and caching scenarios where it's
/// important to distinguish between new discoveries and revisited values.
#[derive(Debug, Clone)]
pub enum Seen<T> {
    /// A value that is being encountered for the first time
    New(T),
    /// A value that has been seen before
    Old(T),
}

impl<T> Seen<T> {
    /// Extracts the inner value regardless of whether it's new or old.
    pub fn any(self) -> T {
        match self {
            Seen::New(x) => x,
            Seen::Old(x) => x,
        }
    }

    /// Returns `Some(T)` if the value is new, `None` if it's old.
    #[allow(clippy::new_ret_no_self, clippy::wrong_self_convention)]
    pub fn new(self) -> Option<T> {
        match self {
            Seen::New(x) => Some(x),
            Seen::Old(_) => None,
        }
    }

    /// Returns `Some(T)` if the value is old, `None` if it's new.
    pub fn old(self) -> Option<T> {
        match self {
            Seen::New(_) => None,
            Seen::Old(x) => Some(x),
        }
    }

    /// Maps the inner value using the provided function.
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Seen<U> {
        match self {
            Self::New(x) => Seen::<U>::New(f(x)),
            Self::Old(x) => Seen::<U>::Old(f(x)),
        }
    }

    /// Converts from `Seen<T>` to `Seen<&T>`.
    pub fn as_ref(&self) -> Seen<&T> {
        match self {
            Self::New(x) => Seen::New(x),
            Self::Old(x) => Seen::Old(x),
        }
    }
}
