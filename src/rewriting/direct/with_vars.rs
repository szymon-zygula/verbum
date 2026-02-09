//! Random rewriter for expressions with variables.
//!
//! This module provides functionality for applying random rewrites to expressions
//! that may contain variables. Variables in expressions are treated as distinct from
//! pattern variables in rewrite rules - they act as symbolic constants during matching.
//!
//! For example, the rule `(+ $0 $1) -> (+ $1 $0)` can be applied to the expression
//! `(+ $5 2)` to produce `(+ 2 $5)`, where `$5` is a variable in the expression.

use crate::language::expression::{AnyExpression, Expression, OwnedPath, VariableId};
use crate::language::symbol::Symbol;
use crate::rewriting::rule::Rule;
use rand::Rng;
use std::collections::HashMap;

/// A pattern match result for expressions with variables.
///
/// Represents a successful match of a pattern against an expression with variables,
/// storing the substitutions for all pattern variables.
#[derive(Clone, Debug, Default)]
pub struct ExpressionMatch {
    substitutions: HashMap<VariableId, Expression>,
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
    fn try_merge(&self, other: &Self) -> Option<Self> {
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
fn try_match_expression(pattern: &Expression, expression: &Expression) -> Option<ExpressionMatch> {
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
                let child_match = try_match_expression(child1, child2)?;
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

/// Instantiates a pattern with matched variables to produce an expression.
///
/// # Arguments
///
/// * `pattern` - The pattern to instantiate (may contain pattern variables)
/// * `matching` - The match result containing substitutions
///
/// # Returns
///
/// Returns an expression with all pattern variables replaced by their matched values.
fn instantiate_pattern(pattern: &Expression, matching: &ExpressionMatch) -> Expression {
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
                .map(|child| instantiate_pattern(child, matching))
                .collect();

            Expression::Symbol(Symbol {
                id: symbol.id,
                children,
            })
        }
    }
}

/// Represents a position in an expression tree where a rewrite can be applied.
#[derive(Debug, Clone)]
struct RewritePosition {
    /// Path to the subexpression
    path: OwnedPath,
    /// Index of the applicable rule
    rule_index: usize,
}

/// Applies n random rewrites to an expression with variables.
///
/// This function performs destructive term rewriting by:
/// 1. Finding all positions where any rule can be applied
/// 2. Randomly selecting one position and rule
/// 3. Applying the rewrite
/// 4. Repeating n times
///
/// Variables in the expression are treated as symbolic constants during matching.
///
/// # Arguments
///
/// * `expression` - The expression to rewrite
/// * `rules` - The rewrite rules to apply
/// * `n` - The number of rewrites to perform
///
/// # Returns
///
/// Returns the rewritten expression after n random rewrites have been applied.
/// If no rewrites are possible at any step, returns the expression as-is.
pub fn rewrite(mut expression: Expression, rules: &[Rule], n: usize) -> Expression {
    let mut rng = rand::thread_rng();

    for _ in 0..n {
        let positions = find_all_rewrite_positions(&expression, rules);

        if positions.is_empty() {
            // No rewrites possible, return current expression
            return expression;
        }

        // Randomly select a position and apply the rewrite
        let idx = rng.gen_range(0..positions.len());
        let position = &positions[idx];

        expression = apply_rewrite_at_position(expression, rules, position);
    }

    expression
}

/// Finds all positions in an expression where any rule can be applied.
fn find_all_rewrite_positions(expression: &Expression, rules: &[Rule]) -> Vec<RewritePosition> {
    expression
        .iter_paths()
        .flat_map(|path| {
            let subexpr = expression.subexpression(path.as_path())?;
            Some(
                rules
                    .iter()
                    .enumerate()
                    .filter_map(move |(rule_index, rule)| {
                        try_match_expression(rule.from(), subexpr)
                            .map(|_| RewritePosition {
                                path: path.clone(),
                                rule_index,
                            })
                    }),
            )
        })
        .flatten()
        .collect()
}

/// Applies a rewrite at a specific position in the expression.
fn apply_rewrite_at_position(
    expression: Expression,
    rules: &[Rule],
    position: &RewritePosition,
) -> Expression {
    expression.apply_at_path(&position.path, |subexpr| {
        let rule = &rules[position.rule_index];

        // Try to match the rule at this position
        if let Some(matching) = try_match_expression(rule.from(), subexpr) {
            // Instantiate the right-hand side with the matched variables
            instantiate_pattern(rule.to(), &matching)
        } else {
            // This shouldn't happen if find_all_rewrite_positions is correct
            subexpr.clone()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;

    #[test]
    fn test_match_expression_with_variables() {
        let lang = Language::simple_math();

        // Pattern: (+ $0 $1) should match expression: (+ $5 2)
        let pattern = lang.parse("(+ $0 $1)").unwrap();
        let expr = lang.parse("(+ $5 2)").unwrap();

        let matching = try_match_expression(&pattern, &expr).unwrap();

        // $0 should match $5
        let matched_0 = matching.at(0).unwrap();
        assert_eq!(*matched_0, lang.parse("$5").unwrap());

        // $1 should match 2
        let matched_1 = matching.at(1).unwrap();
        assert_eq!(*matched_1, lang.parse("2").unwrap());
    }

    #[test]
    fn test_pattern_variables_match_expression_variables() {
        let lang = Language::simple_math();

        // Pattern: $0 should match expression variable $5
        let pattern = lang.parse("$0").unwrap();
        let expr = lang.parse("$5").unwrap();

        let matching = try_match_expression(&pattern, &expr).unwrap();
        let matched = matching.at(0).unwrap();
        assert_eq!(*matched, lang.parse("$5").unwrap());

        // Pattern (+ $0 $0) should match (+ $5 $5) with $0 bound to $5
        let pattern = lang.parse("(+ $0 $0)").unwrap();
        let expr = lang.parse("(+ $5 $5)").unwrap();
        let matching = try_match_expression(&pattern, &expr).unwrap();
        let matched = matching.at(0).unwrap();
        assert_eq!(*matched, lang.parse("$5").unwrap());

        // But (+ $5 $6) should not match with pattern (+ $0 $0) because
        // $0 would need to match both $5 and $6 simultaneously
        let expr = lang.parse("(+ $5 $6)").unwrap();
        assert!(try_match_expression(&pattern, &expr).is_none());
    }

    #[test]
    fn test_instantiate_with_expression_variables() {
        let lang = Language::simple_math();

        let pattern = lang.parse("(+ $1 $0)").unwrap();
        let mut matching = ExpressionMatch::default();
        matching.set(0, lang.parse("$5").unwrap());
        matching.set(1, lang.parse("2").unwrap());

        let result = instantiate_pattern(&pattern, &matching);
        let expected = lang.parse("(+ 2 $5)").unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rewrite_expression_with_variables() {
        let lang = Language::simple_math();

        // Expression with variable: (+ $5 2)
        let expr = lang.parse("(+ $5 2)").unwrap();

        // Rule: commutativity (+ $0 $1) -> (+ $1 $0)
        let rules = vec![Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang)];

        let result = rewrite(expr, &rules, 1);
        let expected = lang.parse("(+ 2 $5)").unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rewrite_nested_expression_with_variables() {
        let lang = Language::simple_math();

        // Expression: (* $x (+ $y 3))
        let expr = lang.parse("(* $0 (+ $1 3))").unwrap();

        // Rules: commutativity
        let rules = vec![
            Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang),
            Rule::from_strings("(* $0 $1)", "(* $1 $0)", &lang),
        ];

        // Apply rewrites - should be able to commute
        let result = rewrite(expr.clone(), &rules, 2);

        // After commuting both operations: (* (+ 3 $1) $0) or similar
        // Just verify it's different and valid
        assert_ne!(result, expr);
    }

    #[test]
    fn test_no_rewrites_with_expression_variables() {
        let lang = Language::simple_math();

        let expr = lang.parse("$5").unwrap();
        let rules = vec![Rule::from_strings("(+ 0 $0)", "$0", &lang)];

        let result = rewrite(expr.clone(), &rules, 5);
        assert_eq!(result, expr);
    }

    #[test]
    fn test_multiple_rewrites_with_expression_variables() {
        let lang = Language::simple_math();

        // Expression: (+ 0 (+ $x 0))
        let expr = lang.parse("(+ 0 (+ $5 0))").unwrap();
        let rules = vec![
            Rule::from_strings("(+ 0 $0)", "$0", &lang),
            Rule::from_strings("(+ $0 0)", "$0", &lang),
        ];

        // After 2 rewrites, should reduce to $5
        let result = rewrite(expr, &rules, 2);
        let expected = lang.parse("$5").unwrap();

        assert_eq!(result, expected);
    }
}
