use itertools::Itertools;

use crate::{
    equation::Equation,
    graph::DataGraph,
    language::expression::{AnyExpression, Expression},
    rewriting::{rule::Rule, unification::IndependentVarUnifier},
};

use super::TermRewritingSystem;

impl Rule {
    /// Checks if `self` depends on `other`, i.e. if application of `other` may cause `self`
    /// to be applicable at new positions. There are two cases where this may happen.
    /// Let `L -> R` be `self` and `L_2 -> R_2` be `other`.
    ///
    /// Case 1: (`L` contained in `R_2`)
    /// See `Self::depends_on_case_1`
    ///
    /// Case 2: (`R_2` contained in `L`)
    /// See `Self::depends_on_case_2`
    ///
    /// Case 3: (Parial matching)
    /// not possible when expressions are trees. But this case is possible in
    /// general monoidal categories (e-hypergraphs or other things like that).
    pub fn depends_on(&self, other: &Rule) -> bool {
        self.depends_on_case_1(other) || self.depends_on_case_2(other)
    }

    /// There is a substitution `σ` and position `ρ` such that `L[σ] = R_2[σ]_ρ`,
    /// but there is no position `λ` such that `L[σ] = L_2[σ]_λ`, i.e.
    /// `L` matches a subexpression of `R` with some variable assignment, but no
    /// subexpression of `L_2` with the same variable assignment.
    // If there are more identitcal substitutions for L in R_2 than in L_2,
    // this would allow for new applications if we didn't have e-graphs.
    // But we do, so this case can be ignored. We only really care about substitutions.
    // ^^^
    // I'm not sure if this is strict enough. Is it possible for new subexpressions
    // to match after `other` has been applied, but these new subexpressions to sit in the same e-classes
    // and thus not add anything?
    fn depends_on_case_1(&self, other: &Rule) -> bool {
        let matches_in_r_2 = other
            .to()
            .iter_subexpressions()
            .filter_map(|subexpression| {
                IndependentVarUnifier::unify(Equation::new(self.from().clone(), subexpression))
            });

        let matches_in_l_2 = other
            .from()
            .iter_subexpressions()
            .filter_map(|subexpression| {
                IndependentVarUnifier::unify(Equation::new(self.from().clone(), subexpression))
            });

        'outer: for r_2_match in matches_in_r_2 {
            for l_2_match in matches_in_l_2.clone() {
                if r_2_match == l_2_match {
                    continue 'outer;
                }
            }

            return true;
        }

        false
    }

    /// There is a substitution `σ` and position `ρ` such that `R_2[σ] = L[σ]_ρ`,
    /// but `L_2[σ] != L[σ]_p`, i.e. `R_2` matches a subexpression of `L`,
    /// but `L_2` does not match the same subexpression.
    /// We don't have to compare the unifiers here! `self` matches
    /// using the same e-classes if there is a match for both `R_2` and `L_2`,
    /// regardless what the unifiers are.
    fn depends_on_case_2(&self, other: &Rule) -> bool {
        for subexpression in self.from().iter_subexpressions() {
            if let Expression::Variable(_) = subexpression {
                continue;
            }

            if IndependentVarUnifier::unify(Equation::new(
                other.to().clone(),
                subexpression.clone(),
            ))
            .is_none()
            {
                // Right-hand side does not match L => try next subexpression
                continue;
            };

            if IndependentVarUnifier::unify(Equation::new(other.from().clone(), subexpression))
                .is_none()
            {
                // Right-hand side of the rule matches L but left-hand doesn't -> ok
                return true;
            }
        }

        false
    }
}

impl TermRewritingSystem {
    pub fn dependency_graph(&self) -> DataGraph<Rule> {
        let mut graph = DataGraph::new();

        for rule in self.rules() {
            graph.add_vertex(rule.clone());
        }

        for rules in self.rules().iter().enumerate().permutations(2) {
            let (idx_1, rule_1) = rules[0];
            let (idx_2, rule_2) = rules[1];
            if rule_1.depends_on(rule_2) {
                graph.add_edge(idx_1, idx_2);
            }
        }

        graph
    }
}

#[cfg(test)]
mod tests {
    use crate::{language::Language, rewriting::rule::Rule};

    fn rule(lang: &Language, from: &str, to: &str) -> Rule {
        Rule::from_strings(from, to, lang)
    }

    #[test]
    fn test_depends_on_case_1() {
        let lang = Language::default()
            .add_symbol("a")
            .add_symbol("b")
            .add_symbol("f")
            .add_symbol("g");
        let rule1 = rule(&lang, "(f $0)", "(g $0)");
        let rule2 = rule(&lang, "(a)", "(f (b))");
        assert!(rule1.depends_on(&rule2));
    }

    #[test]
    fn test_depends_on_case_2() {
        let lang = Language::default()
            .add_symbol("a")
            .add_symbol("b")
            .add_symbol("c")
            .add_symbol("f")
            .add_symbol("g");
        let rule1 = rule(&lang, "(f $0)", "(g $0)");
        let rule2 = rule(&lang, "(f (b))", "(c)");
        assert!(!rule1.depends_on(&rule2));
    }

    #[test]
    fn test_depends_on_case_2_with_variable_lhs() {
        let lang = Language::default()
            .add_symbol("a")
            .add_symbol("b")
            .add_symbol("f")
            .add_symbol("g");
        let rule1 = rule(&lang, "$0", "(g $0)");
        let rule2 = rule(&lang, "(a)", "(f (b))");
        assert!(rule1.depends_on(&rule2));
    }

    #[test]
    fn test_depends_on_case_1_direct() {
        let lang = Language::default()
            .add_symbol("a")
            .add_symbol("b")
            .add_symbol("f")
            .add_symbol("g");
        let rule1 = rule(&lang, "(f $0)", "(g $0)");
        let rule2 = rule(&lang, "(a)", "(f (b))");
        assert!(rule1.depends_on_case_1(&rule2));
    }

    #[test]
    fn test_depends_on_fail() {
        let lang = Language::default()
            .add_symbol("a")
            .add_symbol("b")
            .add_symbol("c")
            .add_symbol("f")
            .add_symbol("g");

        let rule1 = rule(&lang, "(f $0)", "(g $0)");
        let rule2 = rule(&lang, "(f (b))", "(c)");
        assert!(!rule1.depends_on(&rule2));
        assert!(!rule2.depends_on(&rule1));
    }

    #[test]
    fn test_dependency_graph() {
        use crate::graph::DataGraph;
        use crate::rewriting::system::TermRewritingSystem;

        let lang = Language::default()
            .add_symbol("a")
            .add_symbol("b")
            .add_symbol("c")
            .add_symbol("f")
            .add_symbol("g");

        let rule1 = rule(&lang, "(f $0)", "(g $0)"); // f(x) -> g(x)
        let rule2 = rule(&lang, "(a)", "(f (b))"); // a -> f(b) (rule1 depends on rule2 - case 1)
        let rule3 = rule(&lang, "(g (c))", "(a)"); // g(c) -> a
        let rule4 = rule(&lang, "(f (b))", "(c)"); // f(b) -> c (rule1 depends on rule4 - case 2)

        let rules = vec![rule1.clone(), rule2.clone(), rule3.clone(), rule4.clone()];
        let trs = TermRewritingSystem::new(lang, rules);

        let graph = trs.dependency_graph();

        let mut expected_graph = DataGraph::new();
        let idx1 = expected_graph.add_vertex(rule1);
        let idx2 = expected_graph.add_vertex(rule2);
        let idx3 = expected_graph.add_vertex(rule3);
        let idx4 = expected_graph.add_vertex(rule4);

        // rule1 depends on rule2 (case 1)
        expected_graph.add_edge(idx1, idx2);
        // rule2 depends on rule3
        expected_graph.add_edge(idx2, idx3);
        // rule3 depends on rule1
        expected_graph.add_edge(idx3, idx1);
        // rule3 depends on rule4
        expected_graph.add_edge(idx3, idx4);
        // rule4 depends on rule2
        expected_graph.add_edge(idx4, idx2);

        assert_eq!(graph, expected_graph);
    }
}
