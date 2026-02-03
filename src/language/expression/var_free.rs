//! Variable-free expression representation.
//!
//! This module provides the [`VarFreeExpression`] type for expressions that
//! contain no variables - only literals and symbols with concrete children.

use super::{AnyExpression, Expression, Literal};
use crate::language::Language;
use crate::language::symbol::Symbol;
use serde::{Deserialize, Serialize};

/// An expression which does not contain variables.
///
/// Unlike [`Expression`], this type guarantees the absence of variables.
/// These are ground terms consisting only of literals and symbols with concrete children.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VarFreeExpression {
    /// A literal constant value
    Literal(Literal),
    /// A function symbol with variable-free child expressions
    Symbol(Symbol<VarFreeExpression>),
}

impl VarFreeExpression {
    /// Panics if `self` is not a symbol represented by `name` in `lang`. Returns its children otherwise.
    /// Just for doing tests.
    pub fn expect_symbol<'a>(&'a self, name: &str, lang: &Language) -> &'a Vec<Self> {
        let Self::Symbol(Symbol { id, children }) = self else {
            panic!("Expected symbol but did not find a symbol")
        };

        assert_eq!(lang.get_id(name), *id);

        children
    }

    /// Converts this variable-free expression to a general expression.
    pub fn to_expression(&self) -> Expression {
        match self {
            VarFreeExpression::Literal(literal) => Expression::Literal(literal.clone()),
            VarFreeExpression::Symbol(symbol) => {
                Expression::Symbol(symbol.map_children(|child| child.to_expression()))
            }
        }
    }
}

impl AnyExpression for VarFreeExpression {
    fn children(&self) -> Option<Vec<&Self>> {
        match self {
            VarFreeExpression::Symbol(symbol) => Some(symbol.children.iter().collect()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VarFreeExpression;
    use serde_json;

    #[test]
    fn test_var_free_expression_serialization() {
        let lang = crate::language::Language::simple_math();
        let expr = lang.parse_no_vars("(* 2 (+ 3 4))").unwrap();
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: VarFreeExpression = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }
}
