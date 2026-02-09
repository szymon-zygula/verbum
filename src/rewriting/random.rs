//! Random rewriter for destructive term rewriting.
//!
//! This module is deprecated. Use `rewriting::direct::var_free` instead.

#[deprecated(since = "0.1.0", note = "Use `rewriting::direct::var_free::rewrite` instead")]
pub use super::direct::var_free::rewrite;

