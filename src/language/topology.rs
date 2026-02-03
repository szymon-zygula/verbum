//! Expression topology and distance metrics.
//!
//! This module provides utilities for computing the size and structural distance
//! between expressions, useful for cost analysis and heuristics.

use itertools::Itertools;

use super::expression::{AnyExpression, Literal};
use crate::language::expression::Expression;
use crate::language::symbol::Symbol;

type ExpressionMagnitude = u32;

const IDENTICAL_DISTANCE: ExpressionMagnitude = 0;
const BASE_DISTANCE: ExpressionMagnitude = 1;

/// Computes the size (node count) of an expression.
///
/// # Arguments
///
/// * `expr` - The expression to measure
///
/// # Returns
///
/// Returns the total number of nodes in the expression tree
pub fn expression_size(expr: &Expression) -> ExpressionMagnitude {
    match expr {
        Expression::Literal(literal) => literal_size(literal),
        Expression::Symbol(symbol) => symbol_size(symbol),
        Expression::Variable(_) => BASE_DISTANCE,
    }
}

/// Returns the size of a literal (always 1).
pub fn literal_size(_: &Literal) -> ExpressionMagnitude {
    BASE_DISTANCE
}

/// Computes the size of a symbol and its children.
pub fn symbol_size(symbol: &Symbol<Expression>) -> ExpressionMagnitude {
    BASE_DISTANCE
        + symbol
            .children
            .iter()
            .map(expression_size)
            .sum::<ExpressionMagnitude>()
}

/// Computes the structural distance between two expressions.
///
/// # Arguments
///
/// * `expr_1` - The first expression
/// * `expr_2` - The second expression
///
/// # Returns
///
/// Returns a measure of how different the two expressions are structurally
pub fn distance<'a>(mut expr_1: &'a Expression, mut expr_2: &'a Expression) -> ExpressionMagnitude {
    let mut total_distance = distance_structural(Some(expr_1), Some(expr_2));

    let mut vars_1 = expr_1.find_all_variables();
    let mut vars_2 = expr_2.find_all_variables();

    if !vars_1.is_empty() && !vars_2.is_empty() {
        todo!(
            "How to compare two expressions with variables?
            Maybe that's not important for our use case anyway?"
        )
    }

    if !vars_1.is_empty() && vars_2.is_empty() {
        std::mem::swap(&mut vars_1, &mut vars_2);
        std::mem::swap(&mut expr_1, &mut expr_2);
    }

    if vars_1.is_empty() && !vars_2.is_empty() {
        for (_, paths_2) in vars_2 {
            if paths_2.is_empty() {
                continue;
            }

            // Maps each position of a given variable either to
            // 1. The matching subexpression of the expression 1
            // 2. `None` if the variable is structurally impossible to match,
            // e.g. when matching `(+ (sin $1) $1)` with `(+ 4 1)`.
            let subexpressions: Vec<_> = paths_2
                .iter()
                .map(|path_2| expr_1.subexpression(path_2.as_ref()))
                .collect();

            // At this point the structural penalty has been applied already,
            // So $1 in the top example the matching `1` is compared with the empty expression.
            for subexpressions in subexpressions.iter().combinations(2) {
                total_distance += distance_structural(*subexpressions[0], *subexpressions[1]);
            }
        }
    }

    total_distance
}

/// Calculates distance between `expr_1` and `expr_2`, but does not take repeating variables
/// into consideration.
/// E.g. `x0 - x0` will have distance 0 from `1 - 2`, even though the first expression
/// does not match the second one. `distance` function adds on to this function by
/// implementing distance for expressions with repeating variables.)
fn distance_structural(
    expr_1: Option<&Expression>,
    expr_2: Option<&Expression>,
) -> ExpressionMagnitude {
    match (expr_1, expr_2) {
        (None, None) => IDENTICAL_DISTANCE,
        (None, Some(expr)) | (Some(expr), None) => expression_size(expr),
        (Some(expr_1), Some(expr_2)) => distance_structural_defined(expr_1, expr_2),
    }
}

fn distance_structural_defined(expr_1: &Expression, expr_2: &Expression) -> ExpressionMagnitude {
    match (expr_1, expr_2) {
        (Expression::Literal(literal_1), Expression::Literal(literal_2)) => {
            literal_distance(literal_1, literal_2)
        }
        (Expression::Literal(_), Expression::Symbol(_))
        | (Expression::Symbol(_), Expression::Literal(_)) => {
            expression_size(expr_1) + expression_size(expr_2)
        }
        (Expression::Symbol(symbol_1), Expression::Symbol(symbol_2)) => {
            symbol_distance(symbol_1, symbol_2)
        }
        (Expression::Variable(_), _) | (_, Expression::Variable(_)) => IDENTICAL_DISTANCE,
    }
}

pub fn literal_distance(literal_1: &Literal, literal_2: &Literal) -> ExpressionMagnitude {
    if literal_1 == literal_2 {
        IDENTICAL_DISTANCE
    } else {
        literal_size(literal_1) + literal_size(literal_2)
    }
}

pub fn symbol_distance(
    symbol_1: &Symbol<Expression>,
    symbol_2: &Symbol<Expression>,
) -> ExpressionMagnitude {
    if symbol_1.id == symbol_2.id {
        IDENTICAL_DISTANCE
            + symbol_1
                .children
                .iter()
                .zip_longest(symbol_2.children.iter())
                .map(|children| {
                    distance_structural(children.as_ref().left().copied(), children.right())
                })
                .sum::<ExpressionMagnitude>()
    } else {
        symbol_size(symbol_1) + symbol_size(symbol_2)
    }
}

#[cfg(test)]
mod tests {
    use crate::language::Language;
    use crate::language::topology::distance;

    #[test]
    fn variable_distance() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(+ (- 30 21) 5)").unwrap();
        let expr_2 = lang.parse("(+ $0 5)").unwrap();

        assert_eq!(distance(&expr_1, &expr_2), 0);
    }

    #[test]
    fn self_distance() {
        let lang = Language::simple_math();
        let expr = lang
            .parse("(+ (- (sin (cos (* 1 2 3u 4 5u 6 7u) 21) 5)))")
            .unwrap();

        assert_eq!(distance(&expr, &expr), 0);
    }

    #[test]
    fn other_distance() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(+ (- 30 21) (sin (+ (* 1 2) 3 5)))").unwrap();
        let expr_2 = lang.parse("(+ $0        (sin (+ (+ 1 2) 3 4)))").unwrap();

        assert_eq!(distance(&expr_1, &expr_2), 8);
    }

    #[test]
    fn repeated_variable_distance() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(- $0 $0)").unwrap();
        let expr_2 = lang.parse("(- (+ 1 5) (* 5 3))").unwrap();

        assert_eq!(distance(&expr_1, &expr_2), 6);
    }

    #[test]
    fn repeated_variable_distance_2() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(+ $0 $0 $0)").unwrap();
        let expr_2 = lang.parse("(+ 5 (sin 3) (* 3 5))").unwrap();

        assert_eq!(distance(&expr_1, &expr_2), 12);
    }

    #[test]
    fn nested_repeated_variable_distance() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(+ $0 (- $0 $0))").unwrap();
        let expr_2 = lang.parse("(+ 1 (- 2 3))").unwrap();

        // dist(1, 2) + dist(1, 3) + dist(2, 3)
        // dist(1, 2) = size(1) + size(2) = 1 + 1 = 2
        // dist(1, 3) = size(1) + size(3) = 1 + 1 = 2
        // dist(2, 3) = size(2) + size(3) = 1 + 1 = 2
        // Total = 2 + 2 + 2 = 6
        assert_eq!(distance(&expr_1, &expr_2), 6);
    }

    #[test]
    fn different_variables_repeated_distance() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(+ $0 $0 $1 $1)").unwrap();
        let expr_2 = lang.parse("(+ 1 2 3 4)").unwrap();

        // For $0: dist(1, 2) = 2
        // For $1: dist(3, 4) = 2
        // Total = 2 + 2 = 4
        assert_eq!(distance(&expr_1, &expr_2), 4);
    }

    #[test]
    fn variables_in_different_positions_distance() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(- $0 $1 $0)").unwrap();
        let expr_2 = lang.parse("(- 1 2 3)").unwrap();

        // For $0: dist(1, 3) = 2
        // For $1: 0 (single occurrence)
        // Total = 2
        assert_eq!(distance(&expr_1, &expr_2), 2);
    }

    #[test]
    fn variables_and_non_variables_mixed_distance() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(+ $0 1 $0)").unwrap();
        let expr_2 = lang.parse("(+ 2 1 3)").unwrap();

        // Initial distance: dist(1, 1) = 0
        // For $0: dist(2, 3) = 2
        // Total = 0 + 2 = 2
        assert_eq!(distance(&expr_1, &expr_2), 2);
    }

    #[test]
    fn variables_non_matching_structure() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(+ (sin $0) $0 $0)").unwrap();
        let expr_2 = lang.parse("(+ 4 3 5)").unwrap();

        assert_eq!(distance(&expr_1, &expr_2), 7);
    }

    #[test]
    fn variables_non_matching_structure_2() {
        let lang = Language::simple_math();
        let expr_1 = lang.parse("(+ $0 $1 $1)").unwrap();
        let expr_2 = lang.parse("(+ 4 3)").unwrap();

        // 1 for different structure of `+`, one for repeated variable not matching the same thing
        assert_eq!(distance(&expr_1, &expr_2), 2);
    }
}
