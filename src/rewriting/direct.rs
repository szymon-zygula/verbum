//! Direct term rewriting without e-graphs.
//!
//! This module provides functionality for applying rewrites directly to expressions
//! without using e-graphs. It's a destructive approach that directly modifies
//! expressions by selecting applicable rules and positions.
//!
//! Supports both variable-free expressions and expressions with variables.

pub mod var_free;
pub mod with_vars;
