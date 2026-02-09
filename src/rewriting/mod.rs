//! Term rewriting system modules.
//!
//! This module contains the core components of the term rewriting system.

pub mod direct;
pub mod egraph;
pub mod matching;
#[deprecated(since = "0.1.0", note = "Use `direct::var_free` instead")]
pub mod random;
pub mod reachability;
pub mod rule;
pub mod strings;
pub mod system;
pub mod unification;
