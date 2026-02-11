//! Compact value types representing either finite or infinite values.
//!
//! This module provides the `SinglyCompact` enum which can represent either
//! a finite value of type T or positive infinity.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```
//! use verbum::compact::SinglyCompact;
//!
//! let finite = SinglyCompact::Finite(42);
//! let infinite: SinglyCompact<i32> = SinglyCompact::Infinite;
//!
//! assert!(finite.is_finite());
//! assert!(infinite.is_infinite());
//! ```
//!
//! Arithmetic operations:
//!
//! ```
//! use verbum::compact::SinglyCompact;
//!
//! let a = SinglyCompact::Finite(10);
//! let b = SinglyCompact::Finite(20);
//! let inf: SinglyCompact<i32> = SinglyCompact::Infinite;
//!
//! assert_eq!(a + b, SinglyCompact::Finite(30));
//! assert_eq!(a + inf, SinglyCompact::Infinite);
//! ```
//!
//! Ordering and comparison:
//!
//! ```
//! use verbum::compact::SinglyCompact;
//!
//! let a = SinglyCompact::Finite(1);
//! let b = SinglyCompact::Finite(2);
//! let inf: SinglyCompact<i32> = SinglyCompact::Infinite;
//!
//! assert!(a < b);
//! assert!(b < inf);
//! assert_eq!(inf, inf);
//! ```

use std::cmp::{Ordering, PartialOrd};
use std::fmt;
use std::ops::{Add, Mul};

/// Represents a value that can be either finite or positive infinity.
///
/// This is useful for representing results of computations that may not have
/// a finite solution, such as ILP problems with no feasible solution or
/// distance metrics in disconnected spaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SinglyCompact<T> {
    /// A finite value
    Finite(T),
    /// Positive infinity
    Infinite,
}

impl<T> SinglyCompact<T> {
    /// Returns `true` if the value is finite.
    pub fn is_finite(&self) -> bool {
        matches!(self, SinglyCompact::Finite(_))
    }

    /// Returns `true` if the value is infinite.
    pub fn is_infinite(&self) -> bool {
        matches!(self, SinglyCompact::Infinite)
    }

    /// Converts `SinglyCompact<T>` to `Option<T>`.
    ///
    /// Returns `Some(value)` for `Finite(value)`, and `None` for `Infinite`.
    pub fn to_option(self) -> Option<T> {
        match self {
            SinglyCompact::Finite(v) => Some(v),
            SinglyCompact::Infinite => None,
        }
    }

    /// Maps a `SinglyCompact<T>` to `SinglyCompact<U>` by applying a function to the finite value.
    ///
    /// # Arguments
    ///
    /// * `f` - A function to apply to the finite value
    ///
    /// # Returns
    ///
    /// Returns `Finite(f(value))` if `self` is `Finite(value)`, otherwise returns `Infinite`.
    pub fn map<U, F>(self, f: F) -> SinglyCompact<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            SinglyCompact::Finite(v) => SinglyCompact::Finite(f(v)),
            SinglyCompact::Infinite => SinglyCompact::Infinite,
        }
    }

    /// Unwraps the finite value or panics.
    ///
    /// # Panics
    ///
    /// Panics if the value is `Infinite`.
    pub fn unwrap(self) -> T {
        match self {
            SinglyCompact::Finite(v) => v,
            SinglyCompact::Infinite => panic!("called `SinglyCompact::unwrap()` on an `Infinite` value"),
        }
    }

    /// Returns the finite value or a default.
    ///
    /// # Arguments
    ///
    /// * `default` - The default value to return if `self` is `Infinite`
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            SinglyCompact::Finite(v) => v,
            SinglyCompact::Infinite => default,
        }
    }
}

impl<T: PartialOrd> PartialOrd for SinglyCompact<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (SinglyCompact::Finite(a), SinglyCompact::Finite(b)) => a.partial_cmp(b),
            (SinglyCompact::Infinite, SinglyCompact::Infinite) => Some(Ordering::Equal),
            (SinglyCompact::Finite(_), SinglyCompact::Infinite) => Some(Ordering::Less),
            (SinglyCompact::Infinite, SinglyCompact::Finite(_)) => Some(Ordering::Greater),
        }
    }
}

impl<T: Ord> Ord for SinglyCompact<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (SinglyCompact::Finite(a), SinglyCompact::Finite(b)) => a.cmp(b),
            (SinglyCompact::Infinite, SinglyCompact::Infinite) => Ordering::Equal,
            (SinglyCompact::Finite(_), SinglyCompact::Infinite) => Ordering::Less,
            (SinglyCompact::Infinite, SinglyCompact::Finite(_)) => Ordering::Greater,
        }
    }
}

impl<T: Add<Output = T>> Add for SinglyCompact<T> {
    type Output = SinglyCompact<T>;

    fn add(self, other: Self) -> Self::Output {
        match (self, other) {
            (SinglyCompact::Finite(a), SinglyCompact::Finite(b)) => SinglyCompact::Finite(a + b),
            _ => SinglyCompact::Infinite,
        }
    }
}

impl<T: Mul<Output = T>> Mul for SinglyCompact<T> {
    type Output = SinglyCompact<T>;

    fn mul(self, other: Self) -> Self::Output {
        match (self, other) {
            (SinglyCompact::Finite(a), SinglyCompact::Finite(b)) => SinglyCompact::Finite(a * b),
            _ => SinglyCompact::Infinite,
        }
    }
}

impl<T: fmt::Display> fmt::Display for SinglyCompact<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SinglyCompact::Finite(v) => write!(f, "{}", v),
            SinglyCompact::Infinite => write!(f, "∞"),
        }
    }
}

impl<T> From<Option<T>> for SinglyCompact<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => SinglyCompact::Finite(v),
            None => SinglyCompact::Infinite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_finite() {
        let finite: SinglyCompact<i32> = SinglyCompact::Finite(42);
        let infinite: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert!(finite.is_finite());
        assert!(!infinite.is_finite());
    }

    #[test]
    fn test_is_infinite() {
        let finite: SinglyCompact<i32> = SinglyCompact::Finite(42);
        let infinite: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert!(!finite.is_infinite());
        assert!(infinite.is_infinite());
    }

    #[test]
    fn test_to_option() {
        let finite: SinglyCompact<i32> = SinglyCompact::Finite(42);
        let infinite: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert_eq!(finite.to_option(), Some(42));
        assert_eq!(infinite.to_option(), None);
    }

    #[test]
    fn test_map() {
        let finite: SinglyCompact<i32> = SinglyCompact::Finite(21);
        let infinite: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert_eq!(finite.map(|x| x * 2), SinglyCompact::Finite(42));
        assert_eq!(infinite.map(|x| x * 2), SinglyCompact::Infinite);
    }

    #[test]
    fn test_unwrap() {
        let finite: SinglyCompact<i32> = SinglyCompact::Finite(42);
        assert_eq!(finite.unwrap(), 42);
    }

    #[test]
    #[should_panic(expected = "called `SinglyCompact::unwrap()` on an `Infinite` value")]
    fn test_unwrap_panic() {
        let infinite: SinglyCompact<i32> = SinglyCompact::Infinite;
        infinite.unwrap();
    }

    #[test]
    fn test_unwrap_or() {
        let finite: SinglyCompact<i32> = SinglyCompact::Finite(42);
        let infinite: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert_eq!(finite.unwrap_or(0), 42);
        assert_eq!(infinite.unwrap_or(0), 0);
    }

    #[test]
    fn test_ordering() {
        let a: SinglyCompact<i32> = SinglyCompact::Finite(1);
        let b: SinglyCompact<i32> = SinglyCompact::Finite(2);
        let inf: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert!(a < b);
        assert!(b < inf);
        assert!(a < inf);
        assert!(inf == inf);
    }

    #[test]
    fn test_add() {
        let a: SinglyCompact<i32> = SinglyCompact::Finite(1);
        let b: SinglyCompact<i32> = SinglyCompact::Finite(2);
        let inf: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert_eq!(a + b, SinglyCompact::Finite(3));
        assert_eq!(a + inf, SinglyCompact::Infinite);
        assert_eq!(inf + b, SinglyCompact::Infinite);
        assert_eq!(inf + inf, SinglyCompact::Infinite);
    }

    #[test]
    fn test_mul() {
        let a: SinglyCompact<i32> = SinglyCompact::Finite(2);
        let b: SinglyCompact<i32> = SinglyCompact::Finite(3);
        let inf: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert_eq!(a * b, SinglyCompact::Finite(6));
        assert_eq!(a * inf, SinglyCompact::Infinite);
        assert_eq!(inf * b, SinglyCompact::Infinite);
        assert_eq!(inf * inf, SinglyCompact::Infinite);
    }

    #[test]
    fn test_display() {
        let finite: SinglyCompact<i32> = SinglyCompact::Finite(42);
        let infinite: SinglyCompact<i32> = SinglyCompact::Infinite;

        assert_eq!(format!("{}", finite), "42");
        assert_eq!(format!("{}", infinite), "∞");
    }

    #[test]
    fn test_from_option() {
        let some_val: SinglyCompact<i32> = Some(42).into();
        let none_val: SinglyCompact<i32> = None.into();

        assert_eq!(some_val, SinglyCompact::Finite(42));
        assert_eq!(none_val, SinglyCompact::Infinite);
    }
}
