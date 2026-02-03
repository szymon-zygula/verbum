//! A descriptive boolean alternative for function return types.
//!
//! This module provides the [`Did`] enum as a more expressive alternative to
//! boolean return types for functions that either perform an action or don't.

use std::ops::{BitOr, BitOrAssign};

/// Descriptive enum to use instead of `bool` as return type for functions which either do something or not.
///
/// # Examples
///
/// ```no_run
/// # use verbum::did::Did;
/// fn try_update() -> Did {
///     if some_condition() {
///         Did::Something
///     } else {
///         Did::Nothing
///     }
/// }
/// # fn some_condition() -> bool { true }
/// ```
pub enum Did {
    /// Indicates that an action was performed
    Something,
    /// Indicates that no action was performed
    Nothing,
}

impl Did {
    /// Returns `true` if the action was performed.
    pub fn did_something(&self) -> bool {
        match self {
            Did::Something => true,
            Did::Nothing => false,
        }
    }

    /// Returns `true` if no action was performed.
    pub fn did_nothing(&self) -> bool {
        !self.did_something()
    }
}

impl BitOr for Did {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl BitOrAssign for Did {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Did::Nothing, Did::Nothing) => Did::Nothing,
            _ => Did::Something,
        };
    }
}
