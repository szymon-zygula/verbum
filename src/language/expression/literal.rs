//! Literal value representations.
//!
//! This module provides the [`Literal`] enum for representing constant values
//! in expressions, such as integers.

use serde::{Deserialize, Serialize};

/// A literal constant value in an expression.
///
/// Represents concrete values like integers that appear in expressions.
#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Literal {
    /// An unsigned 64-bit integer
    UInt(u64),
    /// A signed 64-bit integer
    Int(i64),
}
