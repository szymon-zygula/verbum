//! Equation representation for term rewriting.
//!
//! This module provides the [`Equation`] struct that represents an equation
//! between two expressions, used in unification and term rewriting algorithms.

use itertools::Itertools;

use crate::language::expression::Expression;

/// An equation between two expressions.
///
/// An equation represents the equality `left = right` between two expressions.
/// Equations are used in unification algorithms to determine if two expressions
/// can be made equal through variable substitution.
#[derive(Clone, Debug)]
pub struct Equation {
    /// The left-hand side of the equation
    pub left: Expression,
    /// The right-hand side of the equation
    pub right: Expression,
}

impl Equation {
    /// Creates a new equation from two expressions.
    ///
    /// # Arguments
    ///
    /// * `left` - The left-hand side expression
    /// * `right` - The right-hand side expression
    pub fn new(left: Expression, right: Expression) -> Self {
        Self { left, right }
    }

    /// Checks if the equation is trivial (both sides are identical).
    ///
    /// # Returns
    ///
    /// Returns `true` if `left == right`, `false` otherwise
    pub fn is_trivial(&self) -> bool {
        self.left == self.right
    }

    /// Checks if the top level nodes are symbols of the same shape.
    /// If so, decomposes the equation into equations comparing the children of the symbol.
    pub fn decompose_if_matching_symbol(&self) -> Option<Vec<Equation>> {
        let Expression::Symbol(symbol_left) = &self.left else {
            return None;
        };

        let Expression::Symbol(symbol_right) = &self.right else {
            return None;
        };

        if symbol_left.same_shape_as(symbol_right) {
            Some(
                symbol_left
                    .children
                    .iter()
                    .zip_eq(symbol_right.children.iter())
                    .map(|(child_1, child_2)| Self::new(child_1.clone(), child_2.clone()))
                    .collect(),
            )
        } else {
            None
        }
    }

    /// Changes the equation from `u = t` to `t = u`
    pub fn reorient(&mut self) {
        std::mem::swap(&mut self.left, &mut self.right);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        equation::Equation,
        language::{Language, expression::Expression},
    };

    fn lang() -> Language {
        Language::simple_math()
    }

    fn expr(str: &str) -> Expression {
        lang().parse(str).unwrap()
    }

    #[test]
    fn new() {
        let eq = Equation::new(expr("1"), expr("2"));
        assert_eq!(eq.left, expr("1"));
        assert_eq!(eq.right, expr("2"));
    }

    #[test]
    fn is_trivial_simple() {
        let eq = Equation::new(expr("1"), expr("1"));
        assert!(eq.is_trivial());

        let eq = Equation::new(expr("1"), expr("2"));
        assert!(!eq.is_trivial());
    }

    #[test]
    fn is_trivial() {
        let eq = Equation::new(expr("(+ (* 3 4) 4 8)"), expr("(+ (* 3 4) 4 8)"));
        assert!(eq.is_trivial());

        let eq = Equation::new(expr("(+ (* 3 3) 4 8)"), expr("(+ (* 3 4) 4 8)"));
        assert!(!eq.is_trivial());
    }

    #[test]
    fn decompose_if_matching_symbol() {
        let eq = Equation::new(expr("(+ 1 2)"), expr("(+ 3 4)"));
        let decomposed = eq.decompose_if_matching_symbol().unwrap();
        assert_eq!(decomposed.len(), 2);
        assert_eq!(decomposed[0].left, expr("1"));
        assert_eq!(decomposed[0].right, expr("3"));
        assert_eq!(decomposed[1].left, expr("2"));
        assert_eq!(decomposed[1].right, expr("4"));
    }

    #[test]
    fn decompose_if_matching_symbol_different_symbols() {
        let eq = Equation::new(expr("(+ 1 2)"), expr("(- 3 4)"));
        assert!(eq.decompose_if_matching_symbol().is_none());
    }

    #[test]
    fn decompose_if_matching_symbol_different_arity() {
        let eq = Equation::new(expr("(+ 1 2)"), expr("(+ 3 4 5)"));
        assert!(eq.decompose_if_matching_symbol().is_none());
    }

    #[test]
    fn decompose_if_matching_symbol_not_symbols() {
        let eq = Equation::new(expr("1"), expr("2"));
        assert!(eq.decompose_if_matching_symbol().is_none());
    }

    #[test]
    fn reorient() {
        let mut eq = Equation::new(expr("1"), expr("2"));
        eq.reorient();
        assert_eq!(eq.left, expr("2"));
        assert_eq!(eq.right, expr("1"));
    }
}
