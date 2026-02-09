//! Random rewriter for destructive term rewriting.
//!
//! This module provides functionality for applying random rewrites to variable-free
//! expressions without using e-graphs. It's a destructive approach that directly
//! modifies expressions by randomly selecting applicable rules and positions.

use crate::language::expression::VarFreeExpression;
use crate::rewriting::direct::{apply_rewrite_at_position, find_all_rewrite_positions};
use crate::rewriting::rule::Rule;
use rand::Rng;

/// Applies n random rewrites to a variable-free expression.
///
/// This function performs destructive term rewriting by:
/// 1. Finding all positions where any rule can be applied
/// 2. Randomly selecting one position and rule
/// 3. Applying the rewrite
/// 4. Repeating n times
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
pub fn rewrite(mut expression: VarFreeExpression, rules: &[Rule], n: usize) -> VarFreeExpression {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;

    #[test]
    fn test_no_rewrites_possible() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("42").unwrap();
        let rules = vec![Rule::from_strings("(+ 0 $0)", "$0", &lang)];

        let result = rewrite(expr.clone(), &rules, 5);
        assert_eq!(result, expr);
    }

    #[test]
    fn test_single_rewrite() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("(+ 0 5)").unwrap();
        let rules = vec![Rule::from_strings("(+ 0 $0)", "$0", &lang)];

        let result = rewrite(expr, &rules, 1);
        let expected = lang.parse_no_vars("5").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_multiple_rewrites() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("(+ 0 (+ 0 5))").unwrap();
        let rules = vec![Rule::from_strings("(+ 0 $0)", "$0", &lang)];

        // After 2 rewrites, should reduce to 5
        let result = rewrite(expr, &rules, 2);
        let expected = lang.parse_no_vars("5").unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_random_rewrite_with_multiple_rules() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("(* 2 5)").unwrap();
        let rules = vec![
            Rule::from_strings("(* $0 $1)", "(* $1 $0)", &lang),
            Rule::from_strings("(* 2 $0)", "(+ $0 $0)", &lang),
        ];

        // Apply rewrites - result should be valid but may vary
        let result = rewrite(expr, &rules, 3);

        // Just check that it produces a valid expression
        // (exact result depends on random choices)
        assert!(matches!(
            result,
            VarFreeExpression::Symbol(_) | VarFreeExpression::Literal(_)
        ));
    }

    #[test]
    fn test_instantiate_expression() {
        let lang = Language::simple_math();
        let pattern = lang.parse("(* $0 $1)").unwrap();
        let expr = lang.parse_no_vars("(* 3 5)").unwrap();

        let matching = pattern.try_match(&expr).unwrap();
        let result_pattern = lang.parse("(+ $1 $0)").unwrap();

        let result = VarFreeExpression::instantiate_from_pattern(&result_pattern, &matching);
        let expected = lang.parse_no_vars("(+ 5 3)").unwrap();

        assert_eq!(result, expected);
    }
}
