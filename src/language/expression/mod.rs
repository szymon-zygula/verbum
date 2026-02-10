//! Expression types and representations.
//!
//! This module provides various expression types used throughout the system.

pub mod any;
pub mod literal;
pub mod mixed;
pub mod multi;
pub mod path;
pub mod var_free;

pub use any::{AnyExpression, LangExpression};
pub use literal::Literal;
pub use mixed::MixedExpression;
pub use path::{OwnedPath, Path};
pub use var_free::VarFreeExpression;

use crate::language::Language;
use crate::language::symbol::Symbol;
use crate::rewriting::egraph::matching::EGraphMatch;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap, HashSet};

pub type VariableId = usize;

/// An expression with variables.
///
/// Represents expressions that may contain variables (e.g., `$0`, `$1`),
/// literals, or symbols with children. Used in pattern matching and rule definitions.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum Expression {
    /// A literal value
    Literal(Literal),
    /// A function symbol with child expressions
    Symbol(Symbol<Expression>),
    /// A variable identified by its ID
    Variable(VariableId),
}

impl Expression {
    /// Generates a variable name from its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The variable identifier
    ///
    /// # Returns
    ///
    /// Returns a string representation like `"$0"`, `"$1"`, etc.
    pub fn variable_name(id: VariableId) -> String {
        format!("${id}")
    }

    /// Returns `None` if `self` contains variables, or a corresponding `VarFreeExpression` if it does not
    pub fn without_variables(&self) -> Option<VarFreeExpression> {
        match self {
            Expression::Variable(_) => None,
            Expression::Symbol(Symbol { id, children }) => {
                let mut children_no_vars = Vec::with_capacity(children.len());

                for child in children {
                    children_no_vars.push(child.without_variables()?);
                }

                Some(VarFreeExpression::Symbol(Symbol {
                    id: *id,
                    children: children_no_vars,
                }))
            }
            Expression::Literal(literal) => Some(VarFreeExpression::Literal(literal.clone())),
        }
    }

    /// Panics if `self` is not a variable, returns its ID otherwise. Just for doing tests.
    pub fn expect_variable(&self) -> VariableId {
        let Self::Variable(id) = self else {
            panic!("Expected variable but did not find it")
        };

        *id
    }

    /// Panics if `self` is not a symbol represented by `name` in `lang`. Returns its children otherwise.
    /// Just for doing tests.
    pub fn expect_symbol<'a>(&'a self, name: &str, lang: &Language) -> &'a Vec<Self> {
        let Self::Symbol(Symbol { id, children }) = self else {
            panic!("Expected symbol but did not find a symbol")
        };

        assert_eq!(lang.get_id(name), *id);

        children
    }

    /// Returns a `HashSet` of all variables occuriring in `self`
    pub fn variables(&self) -> HashSet<VariableId> {
        match self {
            Expression::Literal(_) => HashSet::new(),
            Expression::Symbol(symbol) => symbol
                .children
                .iter()
                .flat_map(|child| child.variables())
                .collect(),
            Expression::Variable(id) => HashSet::from([*id]),
        }
    }

    /// Checks if `self` contains `variable_id` as a descendant
    pub fn contains_variable(&self, variable_id: VariableId) -> bool {
        match self {
            Expression::Literal(_) => false,
            Expression::Symbol(symbol) => symbol
                .children
                .iter()
                .any(|child| child.contains_variable(variable_id)),
            Expression::Variable(x) => *x == variable_id,
        }
    }

    /// Returns a vector of all variables occurring in `self`
    pub fn variables_vec(&self) -> Vec<VariableId> {
        self.variables().into_iter().collect_vec()
    }

    pub fn common_variables(&self, other: &Expression) -> HashSet<VariableId> {
        self.variables()
            .intersection(&other.variables())
            .copied()
            .collect()
    }

    pub fn mixed_expression(self, matching: &EGraphMatch) -> MixedExpression {
        match self {
            Expression::Literal(literal) => MixedExpression::Literal(literal),
            Expression::Symbol(symbol) => MixedExpression::Symbol(Symbol {
                id: symbol.id,
                children: symbol
                    .children
                    .into_iter()
                    .map(|child| child.mixed_expression(matching))
                    .collect(),
            }),
            Expression::Variable(variable_id) => {
                MixedExpression::Class(matching.class_variable(variable_id))
            }
        }
    }

    /// Finds all variables in the expression together with their paths
    pub fn find_all_variables(&self) -> HashMap<VariableId, Vec<OwnedPath>> {
        let mut vars = HashMap::new();
        self.find_all_variables_impl(&mut OwnedPath::default(), &mut vars);
        vars
    }

    fn find_all_variables_impl(
        &self,
        current_path: &mut OwnedPath,
        vars: &mut HashMap<VariableId, Vec<OwnedPath>>,
    ) {
        match self {
            Expression::Variable(variable_id) => {
                vars.entry(*variable_id)
                    .or_default()
                    .push(current_path.clone());
            }
            Expression::Symbol(symbol) => {
                for (i, child) in symbol.children.iter().enumerate() {
                    current_path.push(i);
                    child.find_all_variables_impl(current_path, vars);
                    current_path.pop();
                }
            }
            Expression::Literal(_) => {}
        }
    }

    /// Returns the maximal variable ID found in the expression
    pub fn max_variable_id(&self) -> Option<VariableId> {
        self.variables().into_iter().max()
    }

    /// Renames all variables by adding `shift` to their IDs
    pub fn shift_variables(&mut self, shift: usize) {
        match self {
            Expression::Variable(id) => *id += shift,
            Expression::Symbol(symbol) => {
                for child in &mut symbol.children {
                    child.shift_variables(shift);
                }
            }
            _ => {}
        }
    }

    /// Replaces all occurences of `variable_id` with `expression`
    pub fn substitute(&mut self, variable_id: VariableId, expression: &Expression) {
        match self {
            Expression::Literal(_) => {}
            Expression::Symbol(symbol) => symbol
                .children
                .iter_mut()
                .for_each(|child| child.substitute(variable_id, expression)),
            Expression::Variable(v) => {
                if *v == variable_id {
                    *self = expression.clone()
                }
            }
        }
    }

    /// Applies a transformation function to a specified subexpression.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the subexpression to transform
    /// * `f` - The transformation function to apply
    ///
    /// # Returns
    ///
    /// Returns a new expression with the transformation applied at the specified path.
    pub fn apply_at_path<F>(mut self, path: &OwnedPath, f: F) -> Self
    where
        F: FnOnce(&Self) -> Self,
    {
        if path.0.is_empty() {
            // Apply transformation at root
            return f(&self);
        }

        // Navigate to parent and replace the child
        if let Expression::Symbol(ref mut symbol) = self {
            let child_index = path.0[0];
            let remaining_path = OwnedPath(path.0[1..].to_vec());
            
            symbol.children[child_index] = symbol.children[child_index]
                .clone()
                .apply_at_path(&remaining_path, f);
        }
        
        self
    }
}

impl AnyExpression for Expression {
    fn children(&self) -> Option<Vec<&Self>> {
        match self {
            Expression::Symbol(symbol) => Some(symbol.children.iter().collect()),
            _ => None,
        }
    }
}

/// Helper struct for loading expressions from JSON
#[derive(Deserialize)]
struct ExpressionsFile {
    expressions: Vec<String>,
}

/// Helper function to parse expressions from strings
fn parse_expressions(
    expr_strings: Vec<String>,
    language: &Language,
) -> Result<Vec<VarFreeExpression>, Box<dyn std::error::Error>> {
    expr_strings
        .iter()
        .map(|expr_str| {
            language
                .parse_no_vars(expr_str)
                .map_err(|e| format!("Failed to parse expression '{}': {}", expr_str, e).into())
        })
        .collect()
}

/// Load variable-free expressions from a JSON string
pub fn load_expressions_from_json(
    json_str: &str,
    language: &Language,
) -> Result<Vec<VarFreeExpression>, Box<dyn std::error::Error>> {
    let file: ExpressionsFile = serde_json::from_str(json_str)?;
    parse_expressions(file.expressions, language)
}

/// Load variable-free expressions from a JSON file
pub fn load_expressions_from_file<P: AsRef<std::path::Path>>(
    path: P,
    language: &Language,
) -> Result<Vec<VarFreeExpression>, Box<dyn std::error::Error>> {
    let file: ExpressionsFile = crate::utils::json::load_json(path)?;
    parse_expressions(file.expressions, language)
}

#[cfg(test)]
mod tests {
    use super::Expression;
    use crate::language::Language;
    use serde_json;

    #[test]
    fn test_expression_serialization() {
        let lang = crate::language::Language::simple_math();
        let expr = lang.parse("(* $0 (+ $1 4))").unwrap();
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn contains_variable() {
        let lang = Language::simple_math();

        assert!(!lang.parse("4").unwrap().contains_variable(0));
        assert!(lang.parse("$4").unwrap().contains_variable(4));
        assert!(
            lang.parse("(+ 4 (* (/ 3 $0)) 3 8)")
                .unwrap()
                .contains_variable(0)
        );
        assert!(
            !lang
                .parse("(+ 4 (* (/ 3 $5)) 3 8)")
                .unwrap()
                .contains_variable(0)
        );
    }
}
