//! Direct term rewriting without e-graphs.
//!
//! This module provides functionality for applying rewrites directly to expressions
//! without using e-graphs. Unlike the `random` module which applies rewrites randomly,
//! this module provides deterministic rewriting strategies.
//!
//! Variables in expressions are treated as distinct from pattern variables in rules,
//! allowing rules to be applied to expressions containing variables.

use crate::language::expression::{Expression, VarFreeExpression, VariableId};
use crate::language::symbol::Symbol;
use crate::rewriting::rule::Rule;
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
pub fn try_match(pattern: &Expression, expression: &Expression) -> Option<ExpressionMatch> {
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
                let child_match = try_match(child1, child2)?;
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
pub fn instantiate(pattern: &Expression, matching: &ExpressionMatch) -> Expression {
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
                .map(|child| instantiate(child, matching))
                .collect();

            Expression::Symbol(Symbol {
                id: symbol.id,
                children,
            })
        }
    }
}

/// Applies a single rewrite rule at the first matching position in the expression.
///
/// This function performs a single deterministic rewrite by:
/// 1. Finding the first position where the rule can be applied (in depth-first order)
/// 2. Applying the rewrite at that position
///
/// Variables in the expression are treated as symbolic constants during matching.
///
/// # Arguments
///
/// * `expression` - The expression to rewrite
/// * `rule` - The rewrite rule to apply
///
/// # Returns
///
/// Returns `Some(rewritten_expression)` if the rule was applied, `None` if no match was found.
pub fn rewrite_once(expression: Expression, rule: &Rule) -> Option<Expression> {
    // Try to match at the root
    if let Some(matching) = try_match(rule.from(), &expression) {
        return Some(instantiate(rule.to(), &matching));
    }

    // Try to match in children
    match expression {
        Expression::Symbol(symbol) => {
            for (i, child) in symbol.children.iter().enumerate() {
                if let Some(rewritten_child) = rewrite_once(child.clone(), rule) {
                    let mut new_children = symbol.children.clone();
                    new_children[i] = rewritten_child;
                    return Some(Expression::Symbol(Symbol {
                        id: symbol.id,
                        children: new_children,
                    }));
                }
            }
            None
        }
        _ => None,
    }
}

/// Applies rewrite rules exhaustively to an expression until no more rules can be applied.
///
/// This function performs deterministic rewriting by repeatedly applying rules
/// until a fixed point is reached. Rules are tried in order, and for each rule,
/// the first matching position is rewritten.
///
/// Variables in the expression are treated as symbolic constants during matching.
///
/// # Arguments
///
/// * `expression` - The expression to rewrite
/// * `rules` - The rewrite rules to apply
/// * `max_iterations` - Maximum number of rewrite steps to prevent infinite loops
///
/// # Returns
///
/// Returns the rewritten expression after no more rules can be applied or
/// the maximum number of iterations is reached.
pub fn rewrite(mut expression: Expression, rules: &[Rule], max_iterations: usize) -> Expression {
    for _ in 0..max_iterations {
        let mut rewritten = false;
        
        for rule in rules {
            if let Some(new_expr) = rewrite_once(expression.clone(), rule) {
                expression = new_expr;
                rewritten = true;
                break;
            }
        }
        
        if !rewritten {
            break;
        }
    }
    
    expression
}

/// Applies rewrite rules to a variable-free expression.
///
/// This is a convenience function that converts the variable-free expression
/// to a regular expression, applies rewriting, and converts back.
///
/// # Arguments
///
/// * `expression` - The variable-free expression to rewrite
/// * `rules` - The rewrite rules to apply
/// * `max_iterations` - Maximum number of rewrite steps
///
/// # Returns
///
/// Returns the rewritten variable-free expression, or the original if the result
/// contains variables (which shouldn't happen with proper rules).
pub fn rewrite_var_free(
    expression: VarFreeExpression,
    rules: &[Rule],
    max_iterations: usize,
) -> VarFreeExpression {
    let expr = expression.to_expression();
    let rewritten = rewrite(expr, rules, max_iterations);
    rewritten
        .without_variables()
        .unwrap_or(expression)
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

        let matching = try_match(&pattern, &expr).unwrap();

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

        let matching = try_match(&pattern, &expr).unwrap();
        let matched = matching.at(0).unwrap();
        assert_eq!(*matched, lang.parse("$5").unwrap());

        // Pattern (+ $0 $0) should match (+ $5 $5) with $0 bound to $5
        let pattern = lang.parse("(+ $0 $0)").unwrap();
        let expr = lang.parse("(+ $5 $5)").unwrap();
        let matching = try_match(&pattern, &expr).unwrap();
        let matched = matching.at(0).unwrap();
        assert_eq!(*matched, lang.parse("$5").unwrap());

        // But (+ $5 $6) should not match with pattern (+ $0 $0) because
        // $0 would need to match both $5 and $6 simultaneously
        let expr = lang.parse("(+ $5 $6)").unwrap();
        assert!(try_match(&pattern, &expr).is_none());
    }

    #[test]
    fn test_instantiate_with_expression_variables() {
        let lang = Language::simple_math();

        let pattern = lang.parse("(+ $1 $0)").unwrap();
        let mut matching = ExpressionMatch::default();
        matching.set(0, lang.parse("$5").unwrap());
        matching.set(1, lang.parse("2").unwrap());

        let result = instantiate(&pattern, &matching);
        let expected = lang.parse("(+ 2 $5)").unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rewrite_once_expression_with_variables() {
        let lang = Language::simple_math();

        // Expression with variable: (+ $5 2)
        let expr = lang.parse("(+ $5 2)").unwrap();

        // Rule: commutativity (+ $0 $1) -> (+ $1 $0)
        let rule = Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang);

        let result = rewrite_once(expr, &rule).unwrap();
        let expected = lang.parse("(+ 2 $5)").unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rewrite_expression_with_variables() {
        let lang = Language::simple_math();

        // Expression: (+ 0 (+ $x 0))
        let expr = lang.parse("(+ 0 (+ $5 0))").unwrap();
        let rules = vec![
            Rule::from_strings("(+ 0 $0)", "$0", &lang),
            Rule::from_strings("(+ $0 0)", "$0", &lang),
        ];

        // Should simplify to $5
        let result = rewrite(expr, &rules, 10);
        let expected = lang.parse("$5").unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rewrite_var_free() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("(+ 0 5)").unwrap();
        let rules = vec![Rule::from_strings("(+ 0 $0)", "$0", &lang)];

        let result = rewrite_var_free(expr, &rules, 10);
        let expected = lang.parse_no_vars("5").unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_rewrite_no_match() {
        let lang = Language::simple_math();

        let expr = lang.parse("$7").unwrap();
        let rules = vec![Rule::from_strings("(+ 0 $0)", "$0", &lang)];

        let result = rewrite(expr.clone(), &rules, 5);
        assert_eq!(result, expr);
    }

    #[test]
    fn test_rewrite_nested_expression() {
        let lang = Language::simple_math();

        // Expression: (* $0 (+ $1 0))
        let expr = lang.parse("(* $0 (+ $1 0))").unwrap();
        let rules = vec![Rule::from_strings("(+ $0 0)", "$0", &lang)];

        // Should simplify to (* $0 $1)
        let result = rewrite(expr, &rules, 10);
        let expected = lang.parse("(* $0 $1)").unwrap();

        assert_eq!(result, expected);
    }
}
