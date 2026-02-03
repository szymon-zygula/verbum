//! E-graph node representation.
//!
//! This module provides the [`Node`] type representing individual nodes in an e-graph,
//! which can be either literals or symbols with child class references.

use crate::language::{expression::Literal, symbol::Symbol};

use super::{Analysis, ClassId, DynEGraph, EGraph};

/// A node in an e-graph.
///
/// Nodes represent the actual expressions in the e-graph. Each node is either
/// a literal value or a symbol with references to child equivalence classes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Node {
    /// A literal constant value
    Literal(Literal),
    /// A function symbol with child class IDs
    Symbol(Symbol<ClassId>),
}

impl Node {
    /// Returns a mutable iterator over the child class IDs.
    pub fn iter_mut_children(&mut self) -> std::slice::IterMut<'_, ClassId> {
        match self {
            Node::Literal(_) => std::slice::IterMut::default(),
            Node::Symbol(symbol) => symbol.children.iter_mut(),
        }
    }

    /// Returns an iterator over the child class IDs.
    pub fn iter_children(&self) -> std::slice::Iter<'_, ClassId> {
        match self {
            Node::Literal(_) => std::slice::Iter::default(),
            Node::Symbol(symbol) => symbol.children.iter(),
        }
    }

    /// Returns a canonical version of this node with all child IDs canonicalized.
    pub fn canonical<A: Analysis>(&self, graph: &EGraph<A>) -> Self {
        let mut cloned = self.clone();
        cloned.make_canonical(graph);
        cloned
    }

    /// Makes this node canonical by updating all child IDs to their canonical representatives.
    pub fn make_canonical<A: Analysis>(&mut self, graph: &EGraph<A>) {
        for child_id in self.iter_mut_children() {
            *child_id = graph.canonical_class(*child_id);
        }
    }

    /// Returns the symbol, panicking if this node is a literal.
    pub fn as_symbol(&self) -> &Symbol<ClassId> {
        match self {
            Node::Literal(literal) => panic!("Expected node to be a symbol, is {literal:?}"),
            Node::Symbol(symbol) => symbol,
        }
    }

    /// Returns the symbol if this node is a symbol, `None` if it's a literal.
    pub fn try_as_symbol(&self) -> Option<&Symbol<ClassId>> {
        match self {
            Node::Literal(_) => None,
            Node::Symbol(symbol) => Some(symbol),
        }
    }
}
