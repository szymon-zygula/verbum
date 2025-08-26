use crate::rewriting::egraph::DynEGraph;

use super::{Analysis, EGraph};

/// A simple analysis class used only for testing
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LiteralCountAnalysis {
    count: usize,
}

impl LiteralCountAnalysis {
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Analysis for LiteralCountAnalysis {
    fn make(egraph: &EGraph<Self>, node_id: super::NodeId) -> Self {
        let node = egraph.node(node_id);
        Self {
            count: if node.try_as_symbol().is_none() { 1 } else { 0 },
        }
    }

    fn merge(a: Self, b: Self) -> Self {
        Self {
            count: a.count + b.count,
        }
    }
}
