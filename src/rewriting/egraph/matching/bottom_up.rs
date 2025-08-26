use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    language::{
        expression::{Expression, VariableId},
        symbol::Symbol,
    },
    rewriting::egraph::{ClassId, DynEGraph},
};

use super::{EGraphMatch, Matcher};

pub struct BottomUpMatcher;

impl BottomUpMatcher {
    fn try_match_with_variable_assignment(
        &self,
        egraph: &dyn DynEGraph,
        expression: &Expression,
        assignment: &HashMap<VariableId, ClassId>,
    ) -> Option<ClassId> {
        match expression {
            Expression::Literal(literal) => egraph.find_literal(literal.clone()),
            Expression::Symbol(symbol) => self.try_match_symbol(egraph, symbol, assignment),
            Expression::Variable(variable_id) => Some(assignment[variable_id]),
        }
    }

    fn try_match_symbol(
        &self,
        egraph: &dyn DynEGraph,
        symbol: &Symbol<Expression>,
        assignment: &HashMap<VariableId, ClassId>,
    ) -> Option<ClassId> {
        let children = symbol
            .children
            .iter()
            .map(|child| self.try_match_with_variable_assignment(egraph, child, assignment))
            .collect::<Option<Vec<_>>>()?;

        egraph.find_symbol(Symbol {
            id: symbol.id,
            children,
        })
    }
}

impl Matcher for BottomUpMatcher {
    fn try_match(&self, egraph: &dyn DynEGraph, expression: &Expression) -> Vec<EGraphMatch> {
        expression
            .variables()
            .iter()
            .map(|variable_id| {
                egraph
                    .dyn_classes()
                    .into_iter()
                    .map(|(class_id, _)| (*variable_id, *class_id))
            })
            .multi_cartesian_product()
            .filter_map(|assignment| {
                let substitutions = HashMap::from_iter(assignment);
                self.try_match_with_variable_assignment(egraph, expression, &substitutions)
                    .map(|class_id| EGraphMatch {
                        root: class_id,
                        substitutions,
                    })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::BottomUpMatcher;

    #[test]
    fn find_literal() {
        super::super::tests::find_literal::<BottomUpMatcher, ()>(BottomUpMatcher);
    }

    #[test]
    fn find_symbol() {
        super::super::tests::find_symbol::<BottomUpMatcher, ()>(BottomUpMatcher);
    }

    #[test]
    fn not_find_symbol() {
        super::super::tests::not_find_symbol::<BottomUpMatcher, ()>(BottomUpMatcher);
    }

    #[test]
    fn match_with_variables() {
        super::super::tests::match_with_variables::<BottomUpMatcher, ()>(BottomUpMatcher);
    }

    #[test]
    fn match_with_repeated_variables() {
        super::super::tests::match_with_repeated_variables::<BottomUpMatcher, ()>(BottomUpMatcher);
    }

    #[test]
    fn match_with_repeated_variables_fail() {
        super::super::tests::match_with_repeated_variables_fail::<BottomUpMatcher, ()>(
            BottomUpMatcher,
        );
    }
}
