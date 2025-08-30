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

mod tests {}
