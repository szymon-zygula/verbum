use crate::language::{Language, expression::VarFreeExpression};
use crate::rewriting::egraph::{EGraph, matching::bottom_up::BottomUpMatcher, saturation::{saturate, SaturationConfig}};
use crate::rewriting::rule::Rule;

mod calculus;

pub struct TermRewritingSystem {
    language: Language,
    rules: Vec<Rule>,
}

impl TermRewritingSystem {
    pub fn new(language: Language, rules: Vec<Rule>) -> Self {
        Self { language, rules }
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn rules(&self) -> &Vec<Rule> {
        &self.rules
    }

    /// Build an e-graph from the provided expression and saturate it using the system's rules.
    /// Returns the saturated e-graph.
    pub fn rewrite(&self, expression: VarFreeExpression) -> EGraph {
        let mut egraph = EGraph::from_expression(expression);
        let _ = saturate(&mut egraph, &self.rules, BottomUpMatcher, &SaturationConfig::default());
        egraph
    }
}

#[cfg(test)]
mod tests {
    use super::TermRewritingSystem;
    use crate::language::Language;

    #[test]
    fn trs_rewrite_classical() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(* x0 2)" => "(<< x0 1)",
            "(* x0 1)" => "x0",
            "(/ (* x0 x1) x2)" => "(* x0 (/ x1 x2))",
            "(/ x0 x0)" => "1",
        );

        let expr = lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap();
        let trs = TermRewritingSystem::new(lang, rules);

        let egraph = trs.rewrite(expr);

        assert_eq!(egraph.class_count(), 5);
        assert_eq!(egraph.actual_node_count(), 9);
    }
}