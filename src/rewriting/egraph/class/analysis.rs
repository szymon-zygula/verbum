use super::{EGraph, NodeId};

pub trait Analysis: Sized + Clone + Default {
    fn make(egraph: &EGraph<Self>, node_id: NodeId) -> Self;
    fn merge(a: Self, b: Self) -> Self;
}

impl Analysis for () {
    fn make(_egraph: &EGraph<Self>, _node_id: NodeId) -> Self {}
    fn merge(_a: Self, _b: Self) -> Self {}
}
