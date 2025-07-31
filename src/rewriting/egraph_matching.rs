use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    index_selector::IndexSelector,
    language::{
        expression::{Expression, VariableId},
        symbol::Symbol,
    },
};

use super::egraph::{ClassId, EGraph, NodeId};

#[derive(Clone, Debug)]
pub struct EGraphMatch {
    root: ClassId,
    substitutions: HashMap<VariableId, ClassId>,
}

impl EGraphMatch {
    fn empty(root: ClassId) -> Self {
        EGraphMatch {
            root,
            substitutions: HashMap::new(),
        }
    }

    /// Merges two matches if substitutions don't assign different classes to identical variables
    fn merge(mut self, root: ClassId, other: Self) -> Option<EGraphMatch> {
        self.root = root;

        for (var_id, other_class_id) in other.substitutions {
            if let Some(my_class_id) = self.substitutions.get(&var_id)
                && *my_class_id != other_class_id
            {
                return None;
            }

            self.substitutions.insert(var_id, other_class_id);
        }

        return Some(self);
    }

    /// Same as `EgraphMatch::merge` but generalized to more than two matches
    fn merge_multiple(root: ClassId, matches: Vec<EGraphMatch>) -> Option<EGraphMatch> {
        let mut total_match = EGraphMatch::empty(root);

        for child_match in matches {
            total_match = total_match.merge(root, child_match.clone())?;
        }

        Some(total_match)
    }

    /// Returns the ID of the class which was matched against the root of the pattern
    pub fn root(&self) -> ClassId {
        self.root
    }

    /// Returns the ID of the class which was matched against the variable with ID `variable_id`
    pub fn class_variable(&self, variable_id: VariableId) -> ClassId {
        self.substitutions[&variable_id]
    }
}

pub trait Matcher {
    fn try_match(&self, egraph: &EGraph, expression: &Expression) -> Vec<EGraphMatch>;
}

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

        println!("Matching {:?} at {:?}", symbol, node);

        let matches = egraph
            .node(node_id)
            .iter_children()
            .zip_eq(symbol.children.iter())
            .map(|(&child_class_id, child_expression)| {
                self.try_match_at_class(egraph, child_class_id, child_expression)
            })
            .collect_vec();

        println!("Matched: {:?}", matches);

        let index_matches = IndexSelector::new(
            matches
                .iter()
                .map(|child_matches| child_matches.len())
                .collect_vec(),
        );

        let mut all_matches = Vec::new();

        println!("IndexSelector: {:?}", index_matches);

        for indices in index_matches {
            println!("Indices: {:?}", indices);
            let children_matches = indices
                .iter()
                .enumerate()
                .map(|(idx, inner_idx)| &matches[idx][*inner_idx])
                .collect_vec();

            println!("Children matches: {:?}", children_matches);

            let root = egraph.containing_class(node_id);

            if let Some(matching) = EGraphMatch::merge_multiple(
                root,
                children_matches.iter().map(|x| (*x).clone()).collect_vec(),
            ) {
                all_matches.push(matching);
            }
        }

        println!("All matches: {:?}", all_matches);

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
                .map(|&node_id| self.try_match_symbol_at_node(egraph, node_id, symbol))
                .flatten()
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
            .map(|(&class_id, _)| self.try_match_at_class(egraph, class_id, expression))
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        language::{Language, expression::Literal},
        rewriting::egraph::{EGraph, Node},
    };

    use super::{EGraphMatch, Matcher, TopDownMatcher};

    #[test]
    fn merge_matches() {
        let mut match_1 = EGraphMatch::empty(4);
        match_1.substitutions = HashMap::from([(1, 2), (2, 3), (5, 4)]);

        let mut match_2 = EGraphMatch::empty(4);
        match_2.substitutions = HashMap::from([(2, 3), (4, 5)]);

        assert_eq!(match_1.merge(4, match_2).unwrap().substitutions.len(), 4);
    }

    #[test]
    fn not_merge_matches() {
        let mut match_1 = EGraphMatch::empty(4);
        match_1.substitutions = HashMap::from([(1, 2), (2, 3), (5, 4)]);

        let mut match_2 = EGraphMatch::empty(4);
        match_2.substitutions = HashMap::from([(2, 5), (4, 5)]);

        assert!(match_1.merge(4, match_2).is_none());
    }

    #[test]
    fn find_literal() {
        let lang = Language::math();
        let five = lang.parse("5").unwrap();
        let expr = lang.parse_no_vars("(* (+ 5 (sin (+ 5 3))))").unwrap();
        let egraph = EGraph::from_expression(expr);

        let matcher = TopDownMatcher;
        let matches = matcher.try_match(&egraph, &five);

        assert_eq!(matches.len(), 1);

        let five_id = egraph.node_id(&Node::Literal(Literal::Int(5))).unwrap();
        let five_class_id = egraph.containing_class(five_id);

        assert_eq!(matches[0].root, five_class_id);
        assert!(matches[0].substitutions.is_empty());
    }

    #[test]
    fn find_symbol() {
        let lang = Language::math();
        let sin = lang.parse("(sin (+ 5 3))").unwrap();
        let expr = lang.parse_no_vars("(* (+ 5 (sin (+ 5 3))))").unwrap();
        let egraph = EGraph::from_expression(expr);

        let matcher = TopDownMatcher;
        let matches = matcher.try_match(&egraph, &sin);

        assert_eq!(matches.len(), 1);

        let sin_id = egraph.find_symbols(lang.try_get_id("sin").unwrap())[0];
        let sin_class_id = egraph.containing_class(sin_id);

        assert_eq!(matches[0].root, sin_class_id);
        assert!(matches[0].substitutions.is_empty());
    }

    #[test]
    fn not_find_symbol() {
        let lang = Language::math();
        let five = lang.parse("(sin (+ 5 3))").unwrap();
        let expr = lang.parse_no_vars("(* (+ 5 (sin (+ 5 4))))").unwrap();
        let egraph = EGraph::from_expression(expr);

        let matcher = TopDownMatcher;
        let matches = matcher.try_match(&egraph, &five);

        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn match_with_variables() {
        let lang = Language::math();
        let five = lang.parse("(+ 5 x0)").unwrap();
        let expr = lang.parse_no_vars("(* (+ 5 (sin (+ 5 3))))").unwrap();
        let egraph = EGraph::from_expression(expr);

        let matcher = TopDownMatcher;
        let matches = matcher.try_match(&egraph, &five);

        let pluses = egraph.find_symbols(lang.try_get_id("+").unwrap());
        let mut plus_0 = egraph.containing_class(pluses[0]);
        let mut plus_1 = egraph.containing_class(pluses[1]);

        // Which match is going to be plus_0 and plus_1 is non-deterministic at runtime, without this
        // the test sometimes fails and sometimes passes
        if matches[0].root != plus_0 {
            std::mem::swap(&mut plus_0, &mut plus_1);
        }

        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].root, plus_0);
        assert_eq!(matches[1].root, plus_1);

        let sin_id = egraph.find_symbols(lang.try_get_id("sin").unwrap())[0];
        let sin_class_id = egraph.containing_class(sin_id);

        let three_id = egraph.node_id(&Node::Literal(Literal::Int(3))).unwrap();
        let three_class_id = egraph.containing_class(three_id);

        assert_eq!(matches[0].substitutions.len(), 1);
        assert_eq!(matches[1].substitutions.len(), 1);

        if matches[0].root == 2 {
            assert_eq!(matches[0].substitutions[&0], three_class_id);
            assert_eq!(matches[1].substitutions[&0], sin_class_id);
        } else {
            assert_eq!(matches[0].substitutions[&0], sin_class_id);
            assert_eq!(matches[1].substitutions[&0], three_class_id);
        }
    }

    #[test]
    fn match_with_repeated_variables() {
        let lang = Language::math();
        let five = lang.parse("(* (+ x0 x0) x1)").unwrap();
        let expr = lang.parse_no_vars("(* (+ (sin 5) (sin 5)) 3)").unwrap();
        let egraph = EGraph::from_expression(expr);

        let matcher = TopDownMatcher;
        let matches = matcher.try_match(&egraph, &five);

        let mul_id = egraph.find_symbols(lang.try_get_id("*").unwrap())[0];

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].root, mul_id);

        assert_eq!(matches[0].substitutions.len(), 2);

        let sin_id = egraph.find_symbols(lang.try_get_id("sin").unwrap())[0];
        let three_id =
            egraph.containing_class(egraph.node_id(&Node::Literal(Literal::Int(3))).unwrap());

        assert_eq!(matches[0].substitutions[&0], sin_id);
        assert_eq!(matches[0].substitutions[&1], three_id);
    }

    #[test]
    fn match_with_repeated_variables_fail() {
        let lang = Language::math();
        let five = lang.parse("(* (+ x0 x0) x1)").unwrap();
        let expr = lang.parse_no_vars("(* (+ 8 5) 3)").unwrap();
        let egraph = EGraph::from_expression(expr);

        let matcher = TopDownMatcher;
        let matches = matcher.try_match(&egraph, &five);

        assert_eq!(matches.len(), 0);
    }
}
