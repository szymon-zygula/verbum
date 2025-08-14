use crate::{
    language::{expression::Literal, symbol::SymbolId},
    rewriting::egraph::Node,
};

use super::{Analysis, EGraph, NodeId};

pub trait LocalCost: Default + Clone + Ord {
    fn symbol_cost(symbol_id: SymbolId) -> Self;
    fn literal_cost(literal: &Literal) -> Self;
    fn add(&self, other: &Self) -> Self;
}

impl<LC> Analysis for LC
where
    LC: LocalCost,
{
    fn make(egraph: &EGraph<Self>, node_id: NodeId) -> Self {
        match egraph.node(node_id) {
            Node::Literal(literal) => Self::literal_cost(literal),
            Node::Symbol(symbol) => Self::symbol_cost(symbol.id).add(
                &egraph
                    .node(node_id)
                    .iter_children()
                    .map(|child_id| egraph.class(*child_id).analysis())
                    .fold(LC::default(), |acc, new| acc.add(&new)),
            ),
        }
    }

    fn merge(a: Self, b: Self) -> Self {
        std::cmp::min(a, b)
    }
}
