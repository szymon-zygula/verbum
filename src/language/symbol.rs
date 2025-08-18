use serde::{Deserialize, Serialize};
use super::{
    Language,
    expression::{AnyExpression, LangExpression},
};

pub type SymbolId = usize;

/// A symbol with `id` as its ID and children of type `E`
#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Symbol<E> {
    pub id: SymbolId,
    pub children: Vec<E>,
}

impl<E> Symbol<E> {
    /// Checks if `self` and `other` have the same shape,
    /// i.e. the same symbol id and the same number of children
    pub fn same_shape_as<EO>(&self, other: &Symbol<EO>) -> bool {
        self.id == other.id && self.children.len() == other.children.len()
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
