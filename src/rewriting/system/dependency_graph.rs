use itertools::Itertools;

use crate::{
    equation::Equation,
    graph::DataGraph,
    language::expression::AnyExpression,
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

    /// There is a substitution `σ` and position `ρ` such that `R_2[σ] = L[σ]_p`,
    /// but `L_2[σ] != L[σ]_p`, i.e. `R_2` matches a subexpression of `L`,
    /// but `L_2` does not match the same subexpression.
    fn depends_on_case_2(&self, other: &Rule) -> bool {
        for subexpression in self.from().iter_subexpressions() {
            let Some(right_unifier) = IndependentVarUnifier::unify(Equation::new(
                other.to().clone(),
                subexpression.clone(),
            )) else {
                // Right-hand side does not match L -> try next subexpression
                continue;
            };

            let Some(left_unifier) =
                IndependentVarUnifier::unify(Equation::new(other.from().clone(), subexpression))
            else {
                // Right-hand side of the rule matches L but left-hand doesn't -> ok
                return true;
            };

            if right_unifier != left_unifier {
                // Something matching before rule applied, but it allows for new substitutions
                // -> possibly new rewrites using `self` rule
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
