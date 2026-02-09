//! Pattern matching for expressions.
//!
//! This module provides pattern matching capabilities for both variable-free
//! expressions and expressions with variables, allowing patterns with variables
//! to be matched against concrete expressions.

use std::collections::HashMap;

use itertools::Itertools;

use crate::language::{
    expression::{Expression, VarFreeExpression, VariableId},
    symbol::Symbol,
};

/// A pattern match result for variable-free expressions.
///
/// Represents a successful match of a pattern against a variable-free expression,
/// storing the substitutions for all variables in the pattern.
#[derive(Clone, Debug, Default)]
pub struct Match<'e> {
    substitutions: HashMap<VariableId, &'e VarFreeExpression>,
}

/// A pattern match result for expressions with variables.
///
/// Represents a successful match of a pattern against an expression that may contain variables,
/// storing the substitutions for all pattern variables.
#[derive(Clone, Debug, Default)]
pub struct ExpressionMatch {
    substitutions: HashMap<VariableId, Expression>,
}

impl<'e> Match<'e> {
    /// Tries to merge two matches, returning `None` if they conflict.
    pub fn try_merge(&self, other: &Self) -> Option<Self> {
        let new_match = self.clone();
        new_match.try_add_match(other)
    }

    /// Tries to add substitutions from another match to this one.
    fn try_add_match(mut self, other: &Match<'e>) -> Option<Self> {
        for (key, value) in &self.substitutions {
            if let Some(other_value) = other.substitutions.get(key)
                && other_value != value
            {
                return None;
            }
        }

        self.substitutions.extend(other.substitutions.iter());
        Some(self)
    }

    /// Returns the substitutions in this match.
    pub fn substitutions(&self) -> &HashMap<VariableId, &'e VarFreeExpression> {
        &self.substitutions
    }

    /// Gets the expression substituted for a variable.
    ///
    /// # Arguments
    ///
    /// * `variable` - The variable ID to look up
    ///
    /// # Returns
    ///
    /// Returns `Some(expression)` if the variable was matched, `None` otherwise
    pub fn at(&self, variable: VariableId) -> Option<&'e VarFreeExpression> {
        self.substitutions.get(&variable).copied()
    }

    pub fn set(&mut self, variable: VariableId, expression: &'e VarFreeExpression) {
        self.substitutions.insert(variable, expression);
    }
}

impl ExpressionMatch {
    /// Gets the expression substituted for a pattern variable.
    pub fn at(&self, variable: VariableId) -> Option<&Expression> {
        self.substitutions.get(&variable)
    }

    /// Sets the expression for a pattern variable.
    fn set(&mut self, variable: VariableId, expression: Expression) {
        self.substitutions.insert(variable, expression);
    }

    /// Tries to merge two matches, returning `None` if they conflict.
    pub fn try_merge(&self, other: &Self) -> Option<Self> {
        let mut new_match = self.clone();
        for (key, value) in &other.substitutions {
            if let Some(existing) = new_match.substitutions.get(key) {
                if existing != value {
                    return None;
                }
            } else {
                new_match.substitutions.insert(*key, value.clone());
            }
        }
        Some(new_match)
    }
}

impl Expression {
    /// `self` is treated as a pattern which may match `expression`
    pub fn try_match<'e>(&self, expression: &'e VarFreeExpression) -> Option<Match<'e>> {
        match (self, expression) {
            (
                Expression::Symbol(Symbol {
                    id: id_1,
                    children: children_1,
                }),
                VarFreeExpression::Symbol(Symbol {
                    id: id_2,
                    children: children_2,
                }),
            ) => {
                if id_1 == id_2 && children_1.len() == children_2.len() {
                    let mut entire_match = Match::default();

                    for (child_1, child_2) in children_1.iter().zip_eq(children_2.iter()) {
                        let child_match = child_1.try_match(child_2)?;
                        entire_match = entire_match.try_add_match(&child_match)?;
                    }

                    Some(entire_match)
                } else {
                    None
                }
            }
            (Expression::Variable(variable), expression) => Some(Match::<'e> {
                substitutions: HashMap::from([(*variable, expression)]),
            }),
            (Expression::Literal(lit_1), VarFreeExpression::Literal(lit_2)) => {
                (lit_1 == lit_2).then_some(Match::<'e>::default())
            }
            _ => None,
        }
    }

    /// Tries to match a pattern against an expression with variables.
    ///
    /// Pattern variables (in the pattern) can match any subexpression, including
    /// expression variables. Expression variables are treated as atomic terms.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern to match (may contain pattern variables)
    /// * `expression` - The expression to match against (may contain expression variables)
    ///
    /// # Returns
    ///
    /// Returns `Some(match)` if the pattern matches, `None` otherwise.
    pub fn try_match_expression(pattern: &Expression, expression: &Expression) -> Option<ExpressionMatch> {
        match (pattern, expression) {
            // Pattern variable matches anything
            (Expression::Variable(pattern_var), expr) => {
                let mut m = ExpressionMatch::default();
                m.set(*pattern_var, expr.clone());
                Some(m)
            }
            // Symbols must have same ID and matching children
            (
                Expression::Symbol(Symbol {
                    id: id1,
                    children: children1,
                }),
                Expression::Symbol(Symbol {
                    id: id2,
                    children: children2,
                }),
            ) => {
                if id1 != id2 || children1.len() != children2.len() {
                    return None;
                }

                let mut combined_match = ExpressionMatch::default();
                for (child1, child2) in children1.iter().zip(children2.iter()) {
                    let child_match = Expression::try_match_expression(child1, child2)?;
                    combined_match = combined_match.try_merge(&child_match)?;
                }
                Some(combined_match)
            }
            // Literals must be equal
            (Expression::Literal(lit1), Expression::Literal(lit2)) => {
                if lit1 == lit2 {
                    Some(ExpressionMatch::default())
                } else {
                    None
                }
            }
            // Other combinations don't match
            _ => None,
        }
    }

    /// Instantiates a pattern with matched expression variables to produce an expression.
    ///
    /// # Arguments
    ///
    /// * `pattern` - The pattern to instantiate (may contain pattern variables)
    /// * `matching` - The match result containing substitutions
    ///
    /// # Returns
    ///
    /// Returns an expression with all pattern variables replaced by their matched values.
    pub fn instantiate_expression(pattern: &Expression, matching: &ExpressionMatch) -> Expression {
        match pattern {
            Expression::Literal(lit) => Expression::Literal(lit.clone()),
            Expression::Variable(var_id) => {
                // Look up the matched value for this pattern variable
                matching
                    .at(*var_id)
                    .cloned()
                    .unwrap_or_else(|| Expression::Variable(*var_id))
            }
            Expression::Symbol(symbol) => {
                let children = symbol
                    .children
                    .iter()
                    .map(|child| Expression::instantiate_expression(child, matching))
                    .collect();

                Expression::Symbol(Symbol {
                    id: symbol.id,
                    children,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Match;
    use crate::language::{
        Language,
        expression::{Literal, VarFreeExpression},
    };

    #[test]
    fn match_merge() {
        let mut match_1 = Match::default();
        let mut match_2 = Match::default();

        let expr_1 = VarFreeExpression::Literal(Literal::Int(0));
        let expr_2 = VarFreeExpression::Literal(Literal::Int(1));

        match_1.set(0, &expr_1);
        match_1.set(1, &expr_2);

        match_2.set(0, &expr_1);
        match_2.set(2, &expr_2);

        let merged = match_1.try_merge(&match_2).unwrap();

        assert_eq!(merged.substitutions().len(), 3);
        assert_eq!(merged.at(0).unwrap(), &expr_1);
        assert_eq!(merged.at(1).unwrap(), &expr_2);
        assert_eq!(merged.at(2).unwrap(), &expr_2);

        match_1.set(3, &expr_1);
        match_2.set(3, &expr_2);
        assert!(match_1.try_merge(&match_2).is_none());
    }

    #[test]
    fn simple_match() {
        let lang = Language::simple_math();
        let pattern = lang.parse("(+ 3 7u)").unwrap();

        let expr = lang.parse_no_vars("(+ 3 7u)").unwrap();
        assert_eq!(pattern.try_match(&expr).unwrap().substitutions().len(), 0);

        let expr = lang.parse_no_vars("(+ 3 7)").unwrap();
        assert!(pattern.try_match(&expr).is_none());

        let expr = lang.parse_no_vars("(+ 3 3u)").unwrap();
        assert!(pattern.try_match(&expr).is_none());

        let expr = lang.parse_no_vars("(- 3 7u)").unwrap();
        assert!(pattern.try_match(&expr).is_none());
    }

    #[test]
    fn nested_match() {
        let lang = Language::simple_math();
        let string = "(+ (sin 1) (cos 3) (- 8 12u))";
        let pattern = lang.parse(string).unwrap();

        let expr = lang.parse_no_vars(string).unwrap();
        assert_eq!(pattern.try_match(&expr).unwrap().substitutions().len(), 0);

        let expr = lang.parse_no_vars("(+ (cos 1) (sin 3) (- 8 12u))").unwrap();
        assert!(pattern.try_match(&expr).is_none());
    }

    #[test]
    fn singular_variables_match() {
        let lang = Language::simple_math();
        let pattern = lang.parse("(* $0 (sin $1))").unwrap();

        let expr = lang.parse_no_vars("(* 12 (sin (cos 5u)))").unwrap();
        let matches = pattern.try_match(&expr).unwrap();
        assert_eq!(matches.substitutions().len(), 2);

        let mul_args = expr.expect_symbol("*", &lang);
        assert!(std::ptr::eq(&mul_args[0], matches.substitutions()[&0]));

        let cos = &mul_args[1].expect_symbol("sin", &lang)[0];
        assert!(std::ptr::eq(cos, matches.substitutions()[&1]));
    }

    #[test]
    fn repeated_variable_match() {
        let lang = Language::simple_math();
        let pattern = lang.parse("(+ (* $0 $1) (* $0 $2))").unwrap();

        let expr = lang
            .parse_no_vars("(+ (* (sin 1) 5) (* (sin 1) 3u))")
            .unwrap();

        let match_ = pattern.try_match(&expr).unwrap();
        assert_eq!(match_.substitutions().len(), 3);

        let add_args = expr.expect_symbol("+", &lang);
        let mul_1_args = add_args[0].expect_symbol("*", &lang);
        let mul_2_args = add_args[1].expect_symbol("*", &lang);

        // Any of these two is semantically correct
        assert!(
            std::ptr::eq(match_.at(0).unwrap(), &mul_1_args[0])
                || std::ptr::eq(match_.at(0).unwrap(), &mul_2_args[0])
        );
        assert!(std::ptr::eq(match_.at(1).unwrap(), &mul_1_args[1]));
        assert!(std::ptr::eq(match_.at(2).unwrap(), &mul_2_args[1]));

        let expr = lang
            .parse_no_vars("(+ (* (sin 1) 5) (* (sin 2) 3u))")
            .unwrap();
        assert!(pattern.try_match(&expr).is_none());
    }
}
