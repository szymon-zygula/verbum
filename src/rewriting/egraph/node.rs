use crate::language::{expression::Literal, symbol::Symbol};

use super::{ClassId, EGraph, Analysis};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Literal(Literal),
    Symbol(Symbol<ClassId>),
}

impl Node {
    pub fn iter_mut_children(&mut self) -> std::slice::IterMut<ClassId> {
        match self {
            Node::Literal(_) => std::slice::IterMut::default(),
            Node::Symbol(symbol) => symbol.children.iter_mut(),
        }
    }

    pub fn iter_children(&self) -> std::slice::Iter<ClassId> {
        match self {
            Node::Literal(_) => std::slice::Iter::default(),
            Node::Symbol(symbol) => symbol.children.iter(),
        }
    }

    pub fn canonical<A: Analysis + Default>(&self, graph: &EGraph<A>) -> Self {
        let mut cloned = self.clone();
        cloned.make_canonical(graph);
        cloned
    }

    pub fn make_canonical<A: Analysis + Default>(&mut self, graph: &EGraph<A>) {
        for child_id in self.iter_mut_children() {
            *child_id = graph.canonical_class(*child_id);
        }
    }

    pub fn as_symbol(&self) -> &Symbol<ClassId> {
        match self {
            Node::Literal(literal) => panic!("Expected node to be a symbol, is {literal:?}"),
            Node::Symbol(symbol) => symbol,
        }
    }

    pub fn try_as_symbol(&self) -> Option<&Symbol<ClassId>> {
        match self {
            Node::Literal(_) => None,
            Node::Symbol(symbol) => Some(symbol),
        }
    }
}