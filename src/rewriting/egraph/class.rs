use std::collections::HashSet;
use std::collections::hash_set;

use super::NodeId;

#[derive(Clone, Debug, Default)]
pub struct Class {
    nodes_ids: HashSet<NodeId>,
    parents_ids: HashSet<NodeId>,
}

impl Class {
    pub fn from_node(node_id: NodeId) -> Self {
        Self {
            nodes_ids: HashSet::from([node_id]),
            parents_ids: HashSet::new(),
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.nodes_ids.extend(other.nodes_ids);
        self.parents_ids.extend(other.parents_ids)
    }

    pub fn iter_nodes(&self) -> hash_set::Iter<'_, usize> {
        self.nodes_ids.iter()
    }

    pub fn nodes_ids(&self) -> &HashSet<NodeId> {
        &self.nodes_ids
    }

    pub fn nodes_ids_mut(&mut self) -> &mut HashSet<NodeId> {
        &mut self.nodes_ids
    }

    pub fn parents_ids(&self) -> &HashSet<NodeId> {
        &self.parents_ids
    }

    pub fn parents_ids_mut(&mut self) -> &mut HashSet<NodeId> {
        &mut self.parents_ids
    }
}