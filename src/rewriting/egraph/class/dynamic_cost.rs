use super::{EGraph, NodeId, Analysis};

#[derive(Default, Clone)]
pub struct CostAnalysis;

impl Analysis for CostAnalysis {
    fn make(_egraph: &EGraph<Self>, _node_id: NodeId) -> Self {
        todo!()
    }

    fn merge(_a: Self, _b: Self) -> Self {
        todo!()
    }
}
