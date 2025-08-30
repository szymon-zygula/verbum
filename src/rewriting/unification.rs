use std::collections::HashMap;

use crate::language::expression::{Expression, VariableId};

/// This module implements unification algorithms, that is matching but in the case that both sides of an expression
/// include variables

#[derive(Clone, Default, Debug)]
struct Substitution<'e>(HashMap<VariableId, &'e Expression>);

impl<'e> Substitution<'e> {
    pub fn at(&self, variable: VariableId) -> Option<&'e Expression> {
        self.0.get(&variable).copied()
    }

    pub fn set(&mut self, variable: VariableId, expression: &'e Expression) {
        self.0.insert(variable, expression);
    }
}

#[derive(Clone, Default, Debug)]
struct Unifier<'e> {
    left_substitution: Substitution<'e>,
    right_substitution: Substitution<'e>,
}

impl<'e> Unifier<'e> {
    pub fn unify_robinson(expression_1: &'e Expression, expression_2: &'e Expression) -> Self {
        todo!()
    }

    pub fn unify_de_champeaux(expression_1: &'e Expression, expression_2: &'e Expression) -> Self {
        todo!()
    }

    pub fn left_substitution(&self) -> &Substitution<'e> {
        &self.left_substitution
    }

    pub fn right_substitution(&self) -> &Substitution<'e> {
        &self.left_substitution
    }
}

#[cfg(test)]
mod tests {
    use crate::language::Language;

    use super::*;

    #[test]
    fn test_unify_simple() {
        let lang = Language::simple_math();
        let expression_1 = lang.parse("(* $0 2)").unwrap();
        let expression_2 = lang.parse("(* 3 $1)").unwrap();
        let unifier = Unifier::unify_robinson(&expression_1, &expression_2).unwrap();
        let left_sub = unifier.left_substitution();
        let right_sub = unifier.right_substitution();
        assert_eq!(*left_sub.0.get(&0).unwrap(), &lang.parse("3").unwrap());
        assert_eq!(*right_sub.0.get(&1).unwrap(), &lang.parse("2").unwrap());
    }

    #[test]
    fn test_unify_nested() {
        let lang = Language::simple_math();
        let expression_1 = lang.parse("(+ (* $0 2) $2)").unwrap();
        let expression_2 = lang.parse("(+ $1 5)").unwrap();
        let unifier = Unifier::unify_robinson(&expression_1, &expression_2).unwrap();
        let left_sub = unifier.left_substitution();
        let right_sub = unifier.right_substitution();
        assert_eq!(*left_sub.0.get(&2).unwrap(), &lang.parse("5").unwrap());
        assert_eq!(
            *right_sub.0.get(&1).unwrap(),
            &lang.parse("(* $0 2)").unwrap()
        );
    }

    #[test]
    fn test_unify_no_unification() {
        let lang = Language::simple_math();
        let expression_1 = lang.parse("(* $0 2)").unwrap();
        let expression_2 = lang.parse("(+ 3 $1)").unwrap();
        assert!(Unifier::unify_robinson(&expression_1, &expression_2).is_none());
    }

    #[test]
    fn test_unify_occur_check() {
        let lang = Language::simple_math();
        let expression_1 = lang.parse("$0").unwrap();
        let expression_2 = lang.parse("(* $0 2)").unwrap();
        assert!(Unifier::unify_robinson(&expression_1, &expression_2).is_some());
    }

    #[test]
    fn test_unify_variable_conflict() {
        let lang = Language::simple_math();
        let expression_1 = lang.parse("(+ $1 (+ $1 (+ 1 2)))").unwrap();
        let expression_2 = lang.parse("(+ $2 (+ 1 (+ $1 $2)))").unwrap();
        assert!(Unifier::unify_robinson(&expression_1, &expression_2).is_none())
    }
}
