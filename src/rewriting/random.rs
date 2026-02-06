//! Random rewriter for destructive term rewriting.
//!
//! This module provides functionality for applying random rewrites to variable-free
//! expressions without using e-graphs. It's a destructive approach that directly
//! modifies expressions by randomly selecting applicable rules and positions.

use crate::language::expression::{AnyExpression, Expression, OwnedPath, VarFreeExpression};
use crate::rewriting::rule::Rule;
use rand::Rng;

/// Represents a position in an expression tree where a rewrite can be applied.
#[derive(Debug, Clone)]
struct RewritePosition {
    /// Path to the subexpression
    path: OwnedPath,
    /// Index of the applicable rule
    rule_index: usize,
}

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
pub fn rewrite(
    mut expression: VarFreeExpression,
    rules: &[Rule],
    n: usize,
) -> VarFreeExpression {
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
fn find_all_rewrite_positions(
    expression: &VarFreeExpression,
    rules: &[Rule],
) -> Vec<RewritePosition> {
    expression
        .iter_paths()
        .flat_map(|path| {
            let subexpr = expression.subexpression(path.as_path())?;
            Some(
                rules
                    .iter()
                    .enumerate()
                    .filter_map(move |(rule_index, rule)| {
                        rule.from().try_match(subexpr).map(|_| RewritePosition {
                            path: path.clone(),
                            rule_index,
                        })
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect()
}

/// Applies a rewrite at a specific position in the expression.
fn apply_rewrite_at_position(
    expression: VarFreeExpression,
    rules: &[Rule],
    position: &RewritePosition,
) -> VarFreeExpression {
    apply_at_path(expression, &position.path, |subexpr| {
        let rule = &rules[position.rule_index];
        
        // Try to match the rule at this position
        if let Some(matching) = rule.from().try_match(subexpr) {
            // Instantiate the right-hand side with the matched variables
            instantiate_expression(rule.to(), &matching)
        } else {
            // This shouldn't happen if find_all_rewrite_positions is correct
            subexpr.clone()
        }
    })
}

/// Applies a transformation function at a specific path in the expression tree.
fn apply_at_path<F>(
    mut expression: VarFreeExpression,
    path: &OwnedPath,
    f: F,
) -> VarFreeExpression
where
    F: FnOnce(&VarFreeExpression) -> VarFreeExpression,
{
    if path.0.is_empty() {
        // Apply transformation at root
        f(&expression)
    } else {
        // Navigate to parent and replace the child
        apply_at_path_recursive(&mut expression, &path.0, f);
        expression
    }
}

/// Helper function to recursively navigate and apply transformation.
fn apply_at_path_recursive<F>(
    expression: &mut VarFreeExpression,
    path: &[usize],
    f: F,
) where
    F: FnOnce(&VarFreeExpression) -> VarFreeExpression,
{
    match expression {
        VarFreeExpression::Symbol(symbol) if !path.is_empty() => {
            let child_index = path[0];
            let remaining_path = &path[1..];
            
            if remaining_path.is_empty() {
                // Apply transformation at this child
                symbol.children[child_index] = f(&symbol.children[child_index]);
            } else {
                // Continue recursion
                apply_at_path_recursive(&mut symbol.children[child_index], remaining_path, f);
            }
        }
        _ => {
            // Shouldn't happen with a valid path
        }
    }
}

/// Instantiates an expression pattern with matched variables.
fn instantiate_expression(
    pattern: &Expression,
    matching: &crate::rewriting::matching::Match,
) -> VarFreeExpression {
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
                .map(|child| instantiate_expression(child, matching))
                .collect();
            
            VarFreeExpression::Symbol(crate::language::symbol::Symbol {
                id: symbol.id,
                children,
            })
        }
    }
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
        assert!(matches!(result, VarFreeExpression::Symbol(_) | VarFreeExpression::Literal(_)));
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

    #[test]
    fn test_instantiate_expression() {
        let lang = Language::simple_math();
        let pattern = lang.parse("(* $0 $1)").unwrap();
        let expr = lang.parse_no_vars("(* 3 5)").unwrap();
        
        let matching = pattern.try_match(&expr).unwrap();
        let result_pattern = lang.parse("(+ $1 $0)").unwrap();
        
        let result = instantiate_expression(&result_pattern, &matching);
        let expected = lang.parse_no_vars("(+ 5 3)").unwrap();
        
        assert_eq!(result, expected);
    }
}
