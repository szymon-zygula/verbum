use serde::{Deserialize, Serialize};
use crate::language::{Language, expression::Expression};

use super::egraph::{Analysis, DynEGraph, EGraph, matching::Matcher};

#[derive(Serialize, Deserialize)]
pub struct Rule {
    from: Expression,
    to: Expression,
}

impl Rule {
    pub fn from_strings(from: &str, to: &str, language: &Language) -> Self {
        let from = language.parse(from).unwrap();
        let to = language.parse(to).unwrap();

        Self { from, to }
    }

    pub fn from(&self) -> &Expression {
        &self.from
    }

    pub fn to(&self) -> &Expression {
        &self.to
    }

    /// Returns `true` if anything changed on application, `false` if fixed-point reached.
    pub fn apply<A: Analysis>(&self, egraph: &mut EGraph<A>, matcher: &(impl Matcher + ?Sized)) -> bool {
        matcher
            .try_match(egraph, &self.from)
            .into_iter()
            .any(|matching| {
                let to_add = self.to.clone().mixed_expression(&matching);
                let added = egraph.add_mixed_expression(to_add);
                egraph
                    .merge_classes(matching.root(), *added.as_ref().any())
                    .new()
                    .is_some()
                    || added.new().is_some()
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        language::{Language, expression::Literal},
        rewriting::egraph::{
            DynEGraph, EGraph, Node,
            matching::{Matcher, top_down::TopDownMatcher},
        },
    };

    use super::Rule;

    #[test]
    fn simple_rule_application() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("1").unwrap());
        let rule = Rule::from_strings("1", "2", &lang);

        rule.apply(&mut egraph, &TopDownMatcher);

        assert_eq!(egraph.class_count(), 1);
        assert_eq!(egraph.actual_node_count(), 2);
        assert!(egraph.node_id(&Node::Literal(Literal::Int(1))).is_some());
        assert!(egraph.node_id(&Node::Literal(Literal::Int(2))).is_some());
    }

    #[test]
    fn addition_commutative() {
        let lang = Language::simple_math();
        let mut egraph =
            EGraph::<()>::from_expression(lang.parse_no_vars("(+ 2 (sin 5))").unwrap());
        let rule = Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang);

        rule.apply(&mut egraph, &TopDownMatcher);

        assert_eq!(egraph.class_count(), 4);
        assert_eq!(egraph.actual_node_count(), 5);

        let expected = lang.parse("(+ (sin 5) 2)").unwrap();

                assert_eq!(TopDownMatcher.try_match(&egraph, &expected).len(), 1);
    }

    #[test]
    fn test_rule_serialization() {
        let lang = Language::simple_math();
        let rule = Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang);
        let serialized = serde_json::to_string(&rule).unwrap();
        let deserialized: Rule = serde_json::from_str(&serialized).unwrap();
        assert_eq!(rule.from(), deserialized.from());
        assert_eq!(rule.to(), deserialized.to());
    }
}