use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    index_selector::IndexSelector,
    language::{expression::Expression, symbol::Symbol},
    rewriting::egraph::{ClassId, EGraph, NodeId},
};

use super::{EGraphMatch, Matcher};

pub struct TopDownMatcher;

impl TopDownMatcher {
    fn try_match_symbol_at_node(
        &self,
        egraph: &EGraph,
        node_id: NodeId,
        symbol: &Symbol<Expression>,
    ) -> Vec<EGraphMatch> {
        let node = egraph.node(node_id);
        let Some(node_symbol) = node.try_as_symbol() else {
            return Vec::new();
        };

        if !node_symbol.same_shape_as(symbol) {
            return Vec::new();
        }

        let matches = egraph
            .node(node_id)
            .iter_children()
            .zip_eq(symbol.children.iter())
            .map(|(&child_class_id, child_expression)| {
                self.try_match_at_class(egraph, child_class_id, child_expression)
            })
            .collect_vec();

        let index_matches = IndexSelector::new(
            matches
                .iter()
                .map(|child_matches| child_matches.len())
                .collect_vec(),
        );

        let mut all_matches = Vec::new();

        for indices in index_matches {
            let children_matches = indices
                .iter()
                .enumerate()
                .map(|(idx, inner_idx)| &matches[idx][*inner_idx])
                .collect_vec();

            let root = egraph.containing_class(node_id);

            if let Some(matching) = EGraphMatch::merge_multiple(
                root,
                children_matches.iter().map(|x| (*x).clone()).collect_vec(),
            ) {
                all_matches.push(matching);
            }
        }

        all_matches
    }

    fn try_match_at_class(
        &self,
        egraph: &EGraph,
        class_id: ClassId,
        expression: &Expression,
    ) -> Vec<EGraphMatch> {
        match expression {
            Expression::Literal(literal) => {
                if egraph.class_contains_literal(class_id, literal) {
                    vec![EGraphMatch {
                        root: class_id,
                        substitutions: HashMap::new(),
                    }]
                } else {
                    Vec::new()
                }
            }
            Expression::Symbol(symbol) => egraph
                .class(class_id)
                .iter_nodes()
                .flat_map(|&node_id| self.try_match_symbol_at_node(egraph, node_id, symbol))
                .collect(),
            Expression::Variable(variable_id) => vec![EGraphMatch {
                root: class_id,
                substitutions: HashMap::from([(*variable_id, class_id)]),
            }],
        }
    }
}

impl Matcher for TopDownMatcher {
    fn try_match(&self, egraph: &EGraph, expression: &Expression) -> Vec<EGraphMatch> {
        egraph
            .iter_classes()
            .flat_map(|(&class_id, _)| self.try_match_at_class(egraph, class_id, expression))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::TopDownMatcher;

    #[test]
    fn find_literal() {
        super::super::tests::find_literal(TopDownMatcher);
    }

    #[test]
    fn find_symbol() {
        super::super::tests::find_symbol(TopDownMatcher);
    }

    #[test]
    fn not_find_symbol() {
        super::super::tests::not_find_symbol(TopDownMatcher);
    }

    #[test]
    fn match_with_variables() {
        super::super::tests::match_with_variables(TopDownMatcher);
    }

    #[test]
    fn match_with_repeated_variables() {
        super::super::tests::match_with_repeated_variables(TopDownMatcher);
    }

    #[test]
    fn match_with_repeated_variables_fail() {
        super::super::tests::match_with_repeated_variables_fail(TopDownMatcher);
    }
}
