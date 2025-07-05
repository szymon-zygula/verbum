use super::{
    Language,
    expression::{AnyExpression, LangExpression},
};

pub type SymbolId = usize;

/// A symbol with `id` as its ID and children of type `E`
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Symbol<E: AnyExpression> {
    pub id: SymbolId,
    pub children: Vec<E>,
}

impl<'e, 'l, E: AnyExpression> Symbol<E>
where
    LangExpression<'e, 'l, E>: std::fmt::Display,
{
    fn fmt(&'e self, f: &mut std::fmt::Formatter<'_>, language: &'l Language) -> std::fmt::Result {
        write!(f, "({}", language.get_symbol(self.id))?;
        for child in &self.children {
            write!(f, " {}", child.with_language(language))?;
        }
        write!(f, ")")
    }
}
