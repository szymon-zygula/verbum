//! Direct term rewriting.
//!
//! This module provides functionality for applying rewrites directly to expressions.
//! Unlike the `random` module which applies rewrites randomly,
//! this module provides deterministic rewriting strategies.
//!
//! Variables in expressions are treated as distinct from pattern variables in rules,
//! allowing rules to be applied to expressions containing variables.

use crate::language::expression::{Expression, VarFreeExpression};
use crate::rewriting::rule::Rule;

// Re-export ExpressionMatch for convenience
pub use crate::rewriting::matching::ExpressionMatch;

/// Applies a single rewrite rule at the first matching position in the expression.
///
/// Finds the first position where the rule can be applied (in depth-first order)
/// and applies the rewrite at that position.
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
    let rules = vec![rule.clone()];
    let positions = find_all_rewrite_positions_expr(&expression, &rules);
    
    if let Some(first_pos) = positions.first() {
        Some(apply_rewrite_at_position_expr(expression, &rules, first_pos))
    } else {
        None
    }
}

/// Applies rewrite rules exhaustively to an expression until no more rules can be applied.
///
/// Repeatedly applies rules until a fixed point is reached or the maximum number of
/// iterations is reached. Rules are tried in order, and for each rule, the first
/// matching position is rewritten. Stops if a previously seen expression is encountered
/// to prevent looping.
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
/// Returns the fully rewritten expression.
pub fn rewrite(mut expression: Expression, rules: &[Rule], max_iterations: usize) -> Expression {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    seen.insert(expression.clone());
    
    for _ in 0..max_iterations {
        let mut changed = false;
        
        for rule in rules {
            if let Some(new_expr) = rewrite_once(expression.clone(), rule) {
                // Check if we've seen this expression before (looping detection)
                if seen.contains(&new_expr) {
                    // Already seen this state, stop to prevent looping
                    return expression;
                }
                
                expression = new_expr;
                seen.insert(expression.clone());
                changed = true;
                break;  // Apply only first matching rule per iteration
            }
        }
        
        // Stop if no rule could be applied in this iteration
        if !changed {
            break;
        }
    }
    
    expression
}

/// Applies a single rewrite rule at the first matching position in a variable-free expression.
///
/// Converts to Expression, applies the rewrite, and converts back.
///
/// # Arguments
///
/// * `expression` - The variable-free expression to rewrite
/// * `rule` - The rewrite rule to apply
///
/// # Returns
///
/// Returns `Some(rewritten_expression)` if the rule was applied and the result
/// is still variable-free, `None` otherwise.
pub fn rewrite_once_var_free(
    expression: VarFreeExpression,
    rule: &Rule,
) -> Option<VarFreeExpression> {
    let expr = expression.to_expression();
    let rewritten = rewrite_once(expr, rule)?;
    rewritten.without_variables()
}

/// Applies rewrite rules to a variable-free expression.
///
/// Converts the variable-free expression to a regular expression, applies rewriting,
/// and converts back.
///
/// # Arguments
///
/// * `expression` - The variable-free expression to rewrite
/// * `rules` - The rewrite rules to apply
/// * `max_iterations` - Maximum number of rewrite steps
///
/// # Returns
///
/// Returns the fully rewritten variable-free expression.
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

/// Represents a position in an expression tree where a rewrite can be applied.
///
/// Used for finding all positions where rules can be applied, which is useful
/// for implementing random rewriting strategies.
#[derive(Debug, Clone)]
pub struct RewritePosition {
    /// Path to the subexpression
    pub path: crate::language::expression::OwnedPath,
    /// Index of the applicable rule
    pub rule_index: usize,
}

/// Finds all positions in an expression where any rule can be applied.
///
/// Scans the entire expression tree and identifies all locations
/// where at least one rule matches.
///
/// # Arguments
///
/// * `expression` - The expression to analyze
/// * `rules` - The rewrite rules to check
///
/// # Returns
///
/// Returns a vector of all positions where rules can be applied.
pub fn find_all_rewrite_positions_expr(
    expression: &Expression,
    rules: &[Rule],
) -> Vec<RewritePosition> {
    use crate::language::expression::AnyExpression;
    
    expression
        .iter_paths()
        .flat_map(|path| {
            let subexpr = expression.subexpression(path.as_path())?;
            Some(
                rules
                    .iter()
                    .enumerate()
                    .filter_map(move |(rule_index, rule)| {
                        Expression::try_match_expression(rule.from(), subexpr)
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

/// Applies a rewrite at a specific position in an expression.
///
/// Given a position identified by `find_all_rewrite_positions_expr`, applies
/// the corresponding rule at that location in the expression tree.
///
/// # Arguments
///
/// * `expression` - The expression to rewrite
/// * `rules` - The rewrite rules (must include the rule at `position.rule_index`)
/// * `position` - The position where to apply the rewrite
///
/// # Returns
///
/// Returns the expression with the rewrite applied at the specified position.
pub fn apply_rewrite_at_position_expr(
    expression: Expression,
    rules: &[Rule],
    position: &RewritePosition,
) -> Expression {
    expression.apply_at_path(&position.path, |subexpr| {
        let rule = &rules[position.rule_index];

        // Try to match the rule at this position
        if let Some(matching) = Expression::try_match_expression(rule.from(), subexpr) {
            // Instantiate the right-hand side with the matched variables
            Expression::instantiate_expression(rule.to(), &matching)
        } else {
            // This shouldn't happen if find_all_rewrite_positions_expr is correct
            subexpr.clone()
        }
    })
}

/// Finds all positions in a variable-free expression where any rule can be applied.
///
/// Converts to Expression, finds positions, and returns them.
///
/// # Arguments
///
/// * `expression` - The variable-free expression to analyze
/// * `rules` - The rewrite rules to check
///
/// # Returns
///
/// Returns a vector of all positions where rules can be applied.
pub fn find_all_rewrite_positions(
    expression: &VarFreeExpression,
    rules: &[Rule],
) -> Vec<RewritePosition> {
    let expr = expression.to_expression();
    find_all_rewrite_positions_expr(&expr, rules)
}

/// Applies a rewrite at a specific position in a variable-free expression.
///
/// Converts to Expression, applies the rewrite, and converts back.
///
/// # Arguments
///
/// * `expression` - The expression to rewrite
/// * `rules` - The rewrite rules (must include the rule at `position.rule_index`)
/// * `position` - The position where to apply the rewrite
///
/// # Returns
///
/// Returns the expression with the rewrite applied at the specified position.
pub fn apply_rewrite_at_position(
    expression: VarFreeExpression,
    rules: &[Rule],
    position: &RewritePosition,
) -> VarFreeExpression {
    let expr = expression.to_expression();
    let rewritten = apply_rewrite_at_position_expr(expr, rules, position);
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

        let matching = Expression::try_match_expression(&pattern, &expr).unwrap();

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

        let matching = Expression::try_match_expression(&pattern, &expr).unwrap();
        let matched = matching.at(0).unwrap();
        assert_eq!(*matched, lang.parse("$5").unwrap());

        // Pattern (+ $0 $0) should match (+ $5 $5) with $0 bound to $5
        let pattern = lang.parse("(+ $0 $0)").unwrap();
        let expr = lang.parse("(+ $5 $5)").unwrap();
        let matching = Expression::try_match_expression(&pattern, &expr).unwrap();
        let matched = matching.at(0).unwrap();
        assert_eq!(*matched, lang.parse("$5").unwrap());

        // But (+ $5 $6) should not match with pattern (+ $0 $0) because
        // $0 would need to match both $5 and $6 simultaneously
        let expr = lang.parse("(+ $5 $6)").unwrap();
        assert!(Expression::try_match_expression(&pattern, &expr).is_none());
    }

    #[test]
    fn test_instantiate_with_expression_variables() {
        let lang = Language::simple_math();

        let pattern = lang.parse("(+ $1 $0)").unwrap();
        let mut matching = ExpressionMatch::default();
        matching.at(0);  // Just to satisfy the method access
        
        // Build matching manually since set is private
        let expr0 = lang.parse("$5").unwrap();
        let expr1 = lang.parse("2").unwrap();
        let pattern0 = lang.parse("$0").unwrap();
        let pattern1 = lang.parse("$1").unwrap();
        
        let m0 = Expression::try_match_expression(&pattern0, &expr0).unwrap();
        let m1 = Expression::try_match_expression(&pattern1, &expr1).unwrap();
        let matching = m0.try_merge(&m1).unwrap();

        let result = Expression::instantiate_expression(&pattern, &matching);
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
    
    #[test]
    fn test_commutative_rule_does_not_oscillate() {
        let lang = Language::simple_math();

        // Expression: (+ $5 2)
        let expr = lang.parse("(+ $5 2)").unwrap();
        let rules = vec![Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang)];

        // Should apply once and then stop (not oscillate)
        let result = rewrite(expr.clone(), &rules, 10);
        let expected = lang.parse("(+ 2 $5)").unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_find_all_rewrite_positions() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("(+ 0 (+ 0 5))").unwrap();
        let rules = vec![Rule::from_strings("(+ 0 $0)", "$0", &lang)];

        let positions = find_all_rewrite_positions(&expr, &rules);

        // Should find 2 positions: at root and at nested (+)
        assert_eq!(positions.len(), 2);
    }
}
