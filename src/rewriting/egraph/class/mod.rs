pub mod analysis;
pub mod literal_count;
pub mod local_cost;
pub mod simple_math_local_cost;

use std::collections::HashSet;
use std::collections::hash_set;

use super::{Analysis, EGraph, NodeId};

#[derive(Clone, Debug, Default)]
pub struct Class<A: Analysis> {
    nodes_ids: HashSet<NodeId>,
    parents_ids: HashSet<NodeId>,
    analysis: A,
}

impl<A: Analysis> Class<A> {
    pub fn from_node(egraph: &EGraph<A>, node_id: NodeId) -> Self {
        Self {
            nodes_ids: HashSet::from([node_id]),
            parents_ids: HashSet::new(),
            analysis: A::make(egraph, node_id),
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.nodes_ids.extend(other.nodes_ids);
        self.parents_ids.extend(other.parents_ids);
        self.analysis = A::merge(self.analysis.clone(), other.analysis);
    }

    pub fn analysis(&self) -> &A {
        &self.analysis
    }
}

pub trait DynClass {
    fn iter_nodes(&self) -> hash_set::Iter<'_, usize>;

    fn nodes_ids(&self) -> &HashSet<NodeId>;

    fn nodes_ids_mut(&mut self) -> &mut HashSet<NodeId>;

    fn parents_ids(&self) -> &HashSet<NodeId>;

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
