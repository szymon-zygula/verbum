use crate::language::{Language, expression::Expression};

use super::egraph::{ClassId, EGraph, matching::Matcher};

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

    /// Returns a vector of created `ClassId`s.
    pub fn apply(&self, egraph: &mut EGraph, matcher: &impl Matcher) -> Vec<ClassId> {
        matcher
            .try_match(egraph, &self.from)
            .into_iter()
            .map(|matching| {
                let to_add = self.to.clone().mixed_expression(&matching);
                egraph.add_mixed_expression(to_add)
            })
            .collect()
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
        let lang = Language::math();
        let mut egraph = EGraph::from_expression(lang.parse_no_vars("1").unwrap());
        let rule = Rule::from_strings("1", "2", &lang);

        rule.apply(&mut egraph, &TopDownMatcher);

        assert_eq!(egraph.class_count(), 2);
        assert!(egraph.node_id(&Node::Literal(Literal::Int(1))).is_some());
        assert!(egraph.node_id(&Node::Literal(Literal::Int(2))).is_some());
    }

    #[test]
    fn addition_commutative() {
        let lang = Language::math();
        let mut egraph = EGraph::from_expression(lang.parse_no_vars("(+ 2 (sin 5))").unwrap());
        let rule = Rule::from_strings("(+ x0 x1)", "(+ x1 x0)", &lang);

        rule.apply(&mut egraph, &TopDownMatcher);

        assert_eq!(egraph.class_count(), 5);

        let expected = lang.parse("(+ (sin 5) 2)").unwrap();

        assert_eq!(TopDownMatcher.try_match(&egraph, &expected).len(), 1);
    }
}
