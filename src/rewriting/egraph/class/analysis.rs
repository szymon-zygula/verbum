use super::{EGraph, NodeId};

pub trait Analysis: Sized + Clone + Default {
    /// Creates analysis data for a new class that includes only one node`.
    fn make(egraph: &EGraph<Self>, node_id: NodeId) -> Self;

    // Creates analysis data by merging data from two other classes.
    fn merge(a: Self, b: Self) -> Self;
}

impl Analysis for () {
    fn make(_egraph: &EGraph<Self>, _node_id: NodeId) -> Self {}
    fn merge(_a: Self, _b: Self) -> Self {}
}
