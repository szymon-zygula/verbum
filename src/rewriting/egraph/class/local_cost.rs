use std::{
    iter::Sum,
    ops::{Add, Sub},
};

use crate::{
    language::{
        expression::{Expression, Literal},
        symbol::SymbolId,
    },
    rewriting::egraph::Node,
};

use super::{Analysis, EGraph, NodeId};

pub trait LocalCost: Default + Clone + Ord + Sum + Add<Output = Self> + Sub<Output = Self> {
    fn symbol_cost(symbol_id: SymbolId) -> Self;
    fn literal_cost(literal: &Literal) -> Self;

    fn expression_cost(expression: &Expression) -> Self {
        match expression {
            Expression::Literal(literal) => Self::literal_cost(literal),
            Expression::Symbol(symbol) => {
                symbol
                    .children
                    .iter()
                    .map(|child| Self::expression_cost(child))
                    .sum::<Self>()
                    + Self::symbol_cost(symbol.id)
            }
            Expression::Variable(_) => Self::default(),
        }
    }
}

impl<LC> Analysis for LC
where
    LC: LocalCost,
{
    fn make(egraph: &EGraph<Self>, node_id: NodeId) -> Self {
        match egraph.node(node_id) {
            Node::Literal(literal) => Self::literal_cost(literal),
            Node::Symbol(symbol) => {
                Self::symbol_cost(symbol.id)
                    + egraph
                        .node(node_id)
                        .iter_children()
                        .map(|child_id| egraph.class(*child_id).analysis())
                        .cloned()
                        .sum::<Self>()
            }
        }
    }

    fn merge(a: Self, b: Self) -> Self {
        std::cmp::min(a, b)
    }
}
