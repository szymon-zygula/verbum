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

    /// Applies a transformation function at a specific path in the expression tree.
    ///
    /// Uses `subexpression` to find the target, avoiding manual recursion.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the subexpression to transform
    /// * `f` - The transformation function to apply
    ///
    /// # Returns
    ///
    /// Returns a new expression with the transformation applied at the specified path.
    pub fn apply_at_path<F>(mut self, path: &super::OwnedPath, f: F) -> Self
    where
        F: FnOnce(&Self) -> Self,
    {
        if path.0.is_empty() {
            // Apply transformation at root
            f(&self)
        } else {
            // Navigate to parent and replace the child using iterative approach
            Self::apply_at_path_helper(&mut self, &path.0, f);
            self
        }
    }

    /// Helper to apply transformation at a path.
    fn apply_at_path_helper<F>(expression: &mut Self, path: &[usize], f: F)
    where
        F: FnOnce(&Self) -> Self,
    {
        if path.is_empty() {
            return;
        }

        match expression {
            VarFreeExpression::Symbol(symbol) => {
                let child_index = path[0];
                let remaining_path = &path[1..];

                if remaining_path.is_empty() {
                    // Apply transformation at this child
                    symbol.children[child_index] = f(&symbol.children[child_index]);
                } else {
                    // Continue to the child
                    Self::apply_at_path_helper(
                        &mut symbol.children[child_index],
                        remaining_path,
                        f,
                    );
                }
            }
            _ => {
                // Shouldn't happen with a valid path
            }
        }
    }

    /// Instantiates an expression pattern with matched variables.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The expression pattern that may contain variables
    /// * `matching` - The match result containing variable substitutions
    ///
    /// # Returns
    ///
    /// Returns a variable-free expression with all variables replaced by their matched values.
    pub fn instantiate_from_pattern(
        pattern: &Expression,
        matching: &crate::rewriting::matching::Match,
    ) -> Self {
        match pattern {
            Expression::Literal(lit) => VarFreeExpression::Literal(lit.clone()),
            Expression::Variable(var_id) => {
                // Look up the matched value for this variable
                matching
                    .at(*var_id)
                    .expect("Variable should be in matching")
                    .clone()
            }
            Expression::Symbol(symbol) => {
                let children = symbol
                    .children
                    .iter()
                    .map(|child| Self::instantiate_from_pattern(child, matching))
                    .collect();

                VarFreeExpression::Symbol(Symbol {
                    id: symbol.id,
                    children,
                })
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
