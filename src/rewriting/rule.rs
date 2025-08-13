use crate::language::{Language, expression::Expression};

use super::egraph::{EGraph, matching::Matcher};

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

    /// Returns `true` if anything changed on application, `false` if fixed-point reached.
    pub fn apply(&self, egraph: &mut EGraph, matcher: &impl Matcher) -> bool {
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
            EGraph, Node,
            matching::{Matcher, top_down::TopDownMatcher},
        },
    };

    use super::Rule;

    #[test]
    fn simple_rule_application() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::from_expression(lang.parse_no_vars("1").unwrap());
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
        let mut egraph = EGraph::from_expression(lang.parse_no_vars("(+ 2 (sin 5))").unwrap());
        let rule = Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang);

        rule.apply(&mut egraph, &TopDownMatcher);

        assert_eq!(egraph.class_count(), 4);
        assert_eq!(egraph.actual_node_count(), 5);

        let expected = lang.parse("(+ (sin 5) 2)").unwrap();

        assert_eq!(TopDownMatcher.try_match(&egraph, &expected).len(), 1);
    }
}
