use std::collections::HashMap;

use crate::did::Did;
use crate::equation::Equation;
use crate::language::expression::{Expression, VariableId};

/// This module implements unification algorithms, that is matching but in the case that both sides of an expression
/// include variables

#[derive(Clone, Default, Debug)]
pub struct Substitution(HashMap<VariableId, Expression>);

impl Substitution {
    pub fn get(&self, variable: VariableId) -> Option<&Expression> {
        self.0.get(&variable)
    }

    pub fn set(&mut self, variable: VariableId, expression: Expression) {
        self.0.insert(variable, expression);
    }

    /// Returns two substitutions which compose to `self`.
    /// The first one has all the variables up to `variable` (excluding),
    /// the second one has all the variables starting with `variable`.
    pub fn split_at(mut self, variable: VariableId) -> (Self, Self) {
        let mut first_substitution = Substitution::default();
        for id in 0..variable {
            if let Some(expression) = self.0.remove(&id) {
                first_substitution.set(id, expression);
            }
        }

        (first_substitution, self)
    }

    pub fn map_variables(&mut self, mut f: impl FnMut(VariableId) -> VariableId) {
        self.0 = std::mem::take(&mut self.0)
            .into_iter()
            .map(|(variable, expression)| (f(variable), expression))
            .collect();
    }

    /// Renames all variables by adding `shift` to their IDs
    pub fn shift_positive(&mut self, shift: usize) {
        self.map_variables(|variable| variable + shift);
    }

    /// Renames all variables by subtracting `shift` from their IDs
    pub fn shift_negative(&mut self, shift: usize) {
        self.map_variables(|variable| variable - shift);
    }
}

#[derive(Clone, Default, Debug)]
struct IndependentVarUnifier {
    left_substitution: Substitution,
    right_substitution: Substitution,
}

impl IndependentVarUnifier {
    /// Solves a unification problem for a given equation, treating variables in both of them as separate.
    /// I.e. when both expressions contain `$0`, then it is treated as two variables.
    /// Returns `None` if the problem has no solution.`
    /// If one of the variables on the left will have a variable on the right assigned to it in the solution,
    /// the name of the variable from the right will be shifted.
    pub fn unify(mut equation: Equation) -> Option<Self> {
        let shift = equation.left.max_variable_id().map(|x| x + 1).unwrap_or(0);
        equation.right.shift_variables(shift);

        let (left_substitution, mut right_substitution) =
            UnificationProblem::from_equation(equation)
                .solve()?
                .split_at(shift);

        right_substitution.shift_negative(shift);

        Some(Self {
            left_substitution,
            right_substitution,
        })
    }

    pub fn left_substitution(&self) -> &Substitution {
        &self.left_substitution
    }

    pub fn right_substitution(&self) -> &Substitution {
        &self.right_substitution
    }
}

/// An arbitratry unification problem. Variables can be shared between expressions.
#[derive(Clone, Debug, Default)]
pub struct UnificationProblem {
    equations: Vec<Equation>,
}

impl UnificationProblem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_equation(equation: Equation) -> Self {
        Self {
            equations: vec![equation],
        }
    }

    pub fn from_equations(equations: Vec<Equation>) -> Self {
        Self { equations }
    }

    pub fn add_equation(&mut self, equation: Equation) {
        self.equations.push(equation);
    }

    pub fn with_equation(mut self, equation: Equation) -> Self {
        self.add_equation(equation);
        self
    }

    /// Solves the unification problem.
    /// The algorithm used is taken directly from
    /// Baader, Nipkow, "Term Rewriting and All That", Cambridge University Press 1998,
    /// pages 74-75.
    pub fn solve(mut self) -> Option<Substitution> {
        let mut substitution = Substitution::default();

        'outer: loop {
            for i in 0..self.equations.len() {
                if self.delete(i).did_something()
                    || self.decompose(i).did_something()
                    || self.eliminate(i, &mut substitution).did_something()
                    || self.orient(i).did_something()
                {
                    continue 'outer;
                }
            }

            break;
        }

        if self.equations.is_empty() {
            Some(substitution)
        } else {
            None
        }
    }

    /// If equation number `equation_idx` is trivial, removes it from equations.
    pub fn delete(&mut self, equation_idx: usize) -> Did {
        if self.equations[equation_idx].is_trivial() {
            self.equations.remove(equation_idx);
            Did::Something
        } else {
            Did::Nothing
        }
    }

    /// If equation number `equation_idx` has the same symbol at root on both sides, it is
    /// replaced with equations on children.
    pub fn decompose(&mut self, equation_idx: usize) -> Did {
        match self.equations[equation_idx].decompose_if_matching_symbol() {
            Some(child_equations) => {
                self.equations.remove(equation_idx);
                self.equations.extend(child_equations);
            }
            None => return Did::Nothing,
        }

        Did::Something
    }

    /// Checks if the equation number `equation_idx` is of the form `t = $i` and replaces it with `$i = t`.
    pub fn orient(&mut self, equation_idx: usize) -> Did {
        if matches!(self.equations[equation_idx].right, Expression::Variable(_)) {
            self.equations[equation_idx].reorient();
            Did::Something
        } else {
            Did::Nothing
        }
    }

    /// Checks if the equation number `equation_idx` is of the form `$i = t` and `$i` is not in `t`.
    /// If it is, replaces all occurences of `$i` in other equations with `t`,
    /// removes `equation_idx` from the set of equations and adds `$i -> t` mapping to `substitution`.
    pub fn eliminate(&mut self, equation_idx: usize, substitution: &mut Substitution) -> Did {
        let Expression::Variable(variable_id) = self.equations[equation_idx].left else {
            return Did::Nothing;
        };

        // In case of `$i = $i` the `delete` function should get rid of the equation earlier.
        if self.equations[equation_idx]
            .right
            .contains_variable(variable_id)
        {
            return Did::Nothing;
        }

        let substitute = self.equations.remove(equation_idx).right;

        for equation in &mut self.equations {
            equation.left.substitute(variable_id, &substitute);
            equation.right.substitute(variable_id, &substitute);
        }

        substitution.set(variable_id, substitute);

        Did::Something
    }
}

#[cfg(test)]
mod tests {
    use crate::language::Language;

    use super::*;

    fn solve_unification_problem(expression_1: &str, expression_2: &str) -> Option<Substitution> {
        let lang = Language::simple_math();
        let expression_1 = lang.parse(expression_1).unwrap();
        let expression_2 = lang.parse(expression_2).unwrap();
        UnificationProblem::from_equation(Equation::new(expression_1, expression_2)).solve()
    }

    #[test]
    fn test_unify_trivial() {
        let substitution = solve_unification_problem("1", "1");
        assert!(substitution.is_some());
    }

    #[test]
    fn test_unify_simple() {
        let lang = Language::simple_math();
        let substitution = solve_unification_problem("(* $0 2)", "(* 3 $1)").unwrap();
        assert_eq!(*substitution.get(0).unwrap(), lang.parse("3").unwrap());
        assert_eq!(*substitution.get(1).unwrap(), lang.parse("2").unwrap());
    }

    #[test]
    fn test_unify_nested() {
        let lang = Language::simple_math();
        let substitution = solve_unification_problem("(+ (* $0 2) $2)", "(+ $1 5)").unwrap();
        assert_eq!(*substitution.get(2).unwrap(), lang.parse("5").unwrap());
        assert_eq!(
            *substitution.get(1).unwrap(),
            lang.parse("(* $0 2)").unwrap()
        );
    }

    #[test]
    fn test_unify_no_unification() {
        assert!(solve_unification_problem("(* 2 $0)", "(+ 3 $1)").is_none());
    }

    #[test]
    fn test_unify_occur_check() {
        assert!(solve_unification_problem("$0", "(* $0 2)").is_none());
    }

    #[test]
    fn test_unify_variable_conflict() {
        assert!(
            solve_unification_problem("(+ $1 (+ $1 (+ 1 2)))", "(+ $2 (+ 1 (+ $1 $2)))").is_none()
        )
    }

    fn unify_independent_vars(
        expression_1: &str,
        expression_2: &str,
    ) -> Option<IndependentVarUnifier> {
        let lang = Language::simple_math();
        let expression_1 = lang.parse(expression_1).unwrap();
        let expression_2 = lang.parse(expression_2).unwrap();
        IndependentVarUnifier::unify(Equation::new(expression_1, expression_2))
    }

    #[test]
    fn test_unify_independent_vars_trivial() {
        let unifier = unify_independent_vars("1", "1");
        assert!(unifier.is_some());
    }

    #[test]
    fn test_unify_independent_vars_simple() {
        let lang = Language::simple_math();
        let unifier = unify_independent_vars("(* $0 2)", "(* 3 $0)").unwrap();
        let left_sub = unifier.left_substitution();
        let right_sub = unifier.right_substitution();
        assert_eq!(*left_sub.get(0).unwrap(), lang.parse("3").unwrap());
        assert_eq!(*right_sub.get(0).unwrap(), lang.parse("2").unwrap());
    }

    #[test]
    fn test_unify_independent_vars_occur_check() {
        let lang = Language::simple_math();
        let unifier = unify_independent_vars("$0", "(* $0 2)").unwrap();
        let left_sub = unifier.left_substitution();
        let right_sub = unifier.right_substitution();
        assert_eq!(*left_sub.get(0).unwrap(), lang.parse("(* $1 2)").unwrap());
        assert!(right_sub.get(0).is_none());
    }
}
