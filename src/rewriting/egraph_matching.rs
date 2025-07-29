use std::collections::HashMap;

use crate::language::expression::{Expression, VariableId};

use super::egraph::{ClassId, EGraph};

pub struct EgraphMatch {
    root: ClassId,
    substitutions: HashMap<VariableId, ClassId>,
}

trait Matcher {
    fn try_match(&self, egraph: &EGraph, expression: &Expression) -> Vec<EgraphMatch>;
}

struct BottomUpMatcher;

impl Matcher for BottomUpMatcher {
    fn try_match(&self, egraph: &EGraph, expression: &Expression) -> Vec<EgraphMatch> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        language::{Language, expression::Literal},
        rewriting::egraph::{self, EGraph, Node},
    };

    use super::{BottomUpMatcher, Matcher};

    #[test]
    fn find_literal() {
        let lang = Language::math();
        let five = lang.parse("5").unwrap();
        let expr = lang.parse_no_vars("(* (+ 5 (sin (+ 5 3))))").unwrap();

        let egraph = EGraph::from_expression(expr);

        let matcher = BottomUpMatcher;
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
        let five = lang.parse("(sin (+ 5 3))").unwrap();
        let expr = lang.parse_no_vars("(* (+ 5 (sin (+ 5 3))))").unwrap();

        let egraph = EGraph::from_expression(expr);

        let matcher = BottomUpMatcher;
        let matches = matcher.try_match(&egraph, &five);

        assert_eq!(matches.len(), 1);

        let sin_id = egraph.find_symbols(lang.try_get_id("sin").unwrap())[0];
        let sin_class_id = egraph.containing_class(sin_id);

        assert_eq!(matches[0].root, sin_class_id);
        assert!(matches[0].substitutions.is_empty());
    }

    #[test]
    fn match_with_variables() {}
}
