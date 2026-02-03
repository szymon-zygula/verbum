//! Analysis computation for equivalence classes.
//!
//! This module provides the [`Analysis`] trait for computing and maintaining
//! metadata about equivalence classes in an e-graph.

use super::{EGraph, NodeId};

/// Trait for computing analysis data on e-graph classes.
///
/// Analysis allows associating metadata with each equivalence class,
/// which is automatically maintained as classes are created and merged.
pub trait Analysis: Sized + Clone + Default {
    /// Creates analysis data for a new class that includes only one node.
    ///
    /// # Arguments
    ///
    /// * `egraph` - The e-graph containing the node
    /// * `node_id` - The ID of the node
    fn make(egraph: &EGraph<Self>, node_id: NodeId) -> Self;

    /// Creates analysis data by merging data from two other classes.
    ///
    /// # Arguments
    ///
    /// * `a` - Analysis data from the first class
    /// * `b` - Analysis data from the second class
    ///
    /// # Returns
    ///
    /// Returns the merged analysis data
    fn merge(a: Self, b: Self) -> Self;

    /// Converts the analysis data to a string for display purposes.
    ///
    /// # Returns
    ///
    /// Returns `Some(string)` if the analysis can be displayed, `None` otherwise
    fn to_string(&self) -> Option<String> {
        None
    }
}

/// Unit analysis - no metadata is computed.
impl Analysis for () {
    fn make(_egraph: &EGraph<Self>, _node_id: NodeId) -> Self {}
    fn merge(_a: Self, _b: Self) -> Self {}
}
