//! Symbol representation for expressions.
//!
//! This module provides the [`Symbol`] struct representing function symbols
//! or operators with children in expression trees.

use super::{
    Language,
    expression::{AnyExpression, LangExpression},
};
use serde::{Deserialize, Serialize};

pub type SymbolId = usize;

/// A symbol with `id` as its ID and children of type `E`.
///
/// Represents a function symbol or operator in an expression tree.
/// The symbol is identified by its ID, and has a list of child expressions.
///
/// # Type Parameters
///
/// * `E` - The type of the child expressions
#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Symbol<E> {
    /// The ID of the symbol in the language to which this `Symbol` belongs
    pub id: SymbolId,
    /// The child expressions
    pub children: Vec<E>,
}

impl<E> Symbol<E> {
    /// Checks if `self` and `other` have the same shape,
    /// i.e. the same symbol id and the same number of children.
    pub fn same_shape_as<EO>(&self, other: &Symbol<EO>) -> bool {
        self.id == other.id && self.children.len() == other.children.len()
    }

    /// Maps the children of this symbol using the provided function.
    ///
    /// # Arguments
    ///
    /// * `f` - A function to apply to each child
    ///
    /// # Returns
    ///
    /// Returns a new `Symbol` with transformed children
    pub fn map_children<T>(&self, f: impl FnMut(&E) -> T) -> Symbol<T> {
        Symbol::<T> {
            id: self.id,
            children: self.children.iter().map(f).collect(),
        }
    }
}

impl<'e, 'l, E: AnyExpression> Symbol<E>
where
    LangExpression<'e, 'l, E>: std::fmt::Display,
{
    pub fn fmt(
        &'e self,
        f: &mut std::fmt::Formatter<'_>,
        language: &'l Language,
    ) -> std::fmt::Result {
        write!(f, "({}", language.get_symbol(self.id))?;
        for child in &self.children {
            write!(f, " {}", child.with_language(language))?;
        }
        write!(f, ")")
    }
}
