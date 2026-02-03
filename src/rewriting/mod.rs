//! Term rewriting system modules.
//!
//! This module contains the core components of the term rewriting system:
//! - E-graphs for efficient equality representation
//! - Matching algorithms for pattern matching
//! - Rewrite rules and their application
//! - Unification algorithms
//! - Reachability analysis

pub mod egraph;
pub mod matching;
pub mod reachability;
pub mod rule;
pub mod system;
pub mod unification;
