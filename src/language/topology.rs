use itertools::Itertools;

use super::expression::Literal;
use crate::language::expression::Expression;
use crate::language::symbol::Symbol;

type ExpressionMagnitude = u32;

const IDENTICAL_DISTANCE: ExpressionMagnitude = 0;
const BASE_DISTANCE: ExpressionMagnitude = 1;

pub fn expression_size(expr: &Expression) -> ExpressionMagnitude {
    match expr {
        Expression::Literal(_) => BASE_DISTANCE,
        Expression::Symbol(symbol) => symbol_size(symbol),
        Expression::Variable(_) => BASE_DISTANCE,
    }
}

pub fn symbol_size(symbol: &Symbol<Expression>) -> ExpressionMagnitude {
    BASE_DISTANCE
        + symbol
            .children
            .iter()
            .map(expression_size)
            .sum::<ExpressionMagnitude>()
}

pub fn distance(expr_1: &Expression, expr_2: &Expression) -> ExpressionMagnitude {
    distance_impl(Some(expr_1), Some(expr_2))
}

fn distance_impl(expr_1: Option<&Expression>, expr_2: Option<&Expression>) -> ExpressionMagnitude {
    match (expr_1, expr_2) {
        (None, None) => IDENTICAL_DISTANCE,
        (None, Some(expr)) | (Some(expr), None) => expression_size(expr),
        (Some(expr_1), Some(expr_2)) => distance_defined(expr_1, expr_2),
    }
}

fn distance_defined(expr_1: &Expression, expr_2: &Expression) -> ExpressionMagnitude {
    match (expr_1, expr_2) {
        (Expression::Literal(literal_1), Expression::Literal(literal_2)) => {
            literal_distance(literal_1, literal_2)
        }
        (Expression::Literal(_), Expression::Symbol(_))
        | (Expression::Symbol(_), Expression::Literal(_)) => BASE_DISTANCE,
        (Expression::Symbol(symbol_1), Expression::Symbol(symbol_2)) => {
            symbol_distance(symbol_1, symbol_2)
        }
        (Expression::Variable(_), _) | (_, Expression::Variable(_)) => 0,
    }
}

pub fn literal_distance(literal_1: &Literal, literal_2: &Literal) -> ExpressionMagnitude {
    if literal_1 == literal_2 {
        IDENTICAL_DISTANCE
    } else {
        BASE_DISTANCE
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
                .map(|children| distance_impl(children.as_ref().left().copied(), children.right()))
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

        assert_eq!(distance(&expr_1, &expr_2), 7);
    }
}
