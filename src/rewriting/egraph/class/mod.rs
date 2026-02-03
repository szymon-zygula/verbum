//! Equivalence class representation and analysis.
//!
//! This module provides the [`Class`] type representing an equivalence class
//! in an e-graph, along with analysis computation mechanisms.

pub mod analysis;
pub mod literal_count;
pub mod local_cost;
pub mod simple_math_local_cost;

use std::collections::HashSet;
use std::collections::hash_set;

use super::{Analysis, EGraph, NodeId};

/// An equivalence class in an e-graph.
///
/// A class contains a set of nodes that are known to be equivalent,
/// along with associated analysis data computed from those nodes.
///
/// # Type Parameters
///
/// * `A` - The analysis type for computing metadata about the class
#[derive(Clone, Debug, Default)]
pub struct Class<A: Analysis> {
    nodes_ids: HashSet<NodeId>,
    parents_ids: HashSet<NodeId>,
    analysis: A,
}

impl<A: Analysis> Class<A> {
    /// Creates a new class containing a single node.
    ///
    /// # Arguments
    ///
    /// * `egraph` - The e-graph containing the node
    /// * `node_id` - The ID of the node to create a class for
    pub fn from_node(egraph: &EGraph<A>, node_id: NodeId) -> Self {
        Self {
            nodes_ids: HashSet::from([node_id]),
            parents_ids: HashSet::new(),
            analysis: A::make(egraph, node_id),
        }
    }

    /// Merges another class into this one.
    ///
    /// # Arguments
    ///
    /// * `other` - The class to merge into this one
    pub fn merge(&mut self, other: Self) {
        self.nodes_ids.extend(other.nodes_ids);
        self.parents_ids.extend(other.parents_ids);
        self.analysis = A::merge(self.analysis.clone(), other.analysis);
    }

    /// Returns a reference to the analysis data for this class.
    pub fn analysis(&self) -> &A {
        &self.analysis
    }
}

/// Trait for dynamically accessing class data.
pub trait DynClass {
    /// Returns an iterator over the node IDs in this class.
    fn iter_nodes(&self) -> hash_set::Iter<'_, usize>;

    /// Returns a reference to the set of node IDs.
    fn nodes_ids(&self) -> &HashSet<NodeId>;

    /// Returns a mutable reference to the set of node IDs.
    fn nodes_ids_mut(&mut self) -> &mut HashSet<NodeId>;

    /// Returns a reference to the set of parent node IDs.
    fn parents_ids(&self) -> &HashSet<NodeId>;

    /// Returns a mutable reference to the set of parent node IDs.
    fn parents_ids_mut(&mut self) -> &mut HashSet<NodeId>;
}

impl<A: Analysis> DynClass for Class<A> {
    fn iter_nodes(&self) -> hash_set::Iter<'_, usize> {
        self.nodes_ids.iter()
    }

    fn nodes_ids(&self) -> &HashSet<NodeId> {
        &self.nodes_ids
    }

    fn nodes_ids_mut(&mut self) -> &mut HashSet<NodeId> {
        &mut self.nodes_ids
    }

    fn parents_ids(&self) -> &HashSet<NodeId> {
        &self.parents_ids
    }

    fn parents_ids_mut(&mut self) -> &mut HashSet<NodeId> {
        &mut self.parents_ids
    }
}
