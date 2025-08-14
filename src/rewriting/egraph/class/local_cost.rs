use crate::{
    language::{expression::Literal, symbol::SymbolId},
    rewriting::egraph::Node,
};

use super::{Analysis, EGraph, NodeId};

pub trait LocalCost: Default + Clone {
    type Cost: Ord;

    fn symbol_cost(symbol_id: SymbolId) -> Self::Cost;
    fn literal_cost(literal: &Literal) -> Self::Cost;
    fn wrap_cost(cost: Self::Cost) -> Self;
    fn cost(&self) -> Self::Cost;
}

impl<LC> Analysis for LC
where
    LC: LocalCost,
{
    fn make(egraph: &EGraph<Self>, node_id: NodeId) -> Self {
        match egraph.node(node_id) {
            Node::Literal(literal) => Self::wrap_cost(Self::literal_cost(literal)),
            Node::Symbol(symbol) => Self::wrap_cost(Self::symbol_cost(symbol.id)),
        }
    }

    fn merge(a: Self, b: Self) -> Self {
        Self::wrap_cost(std::cmp::min(a.cost(), b.cost()))
    }
}
