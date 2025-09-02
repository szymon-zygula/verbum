use itertools::Itertools;

use crate::language::expression::Expression;

#[derive(Clone, Debug)]
pub struct Equation {
    pub left: Expression,
    pub right: Expression,
}

impl Equation {
    pub fn new(left: Expression, right: Expression) -> Self {
        Self { left, right }
    }

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
