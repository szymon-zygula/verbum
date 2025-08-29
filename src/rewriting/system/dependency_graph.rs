use itertools::Itertools;

use crate::{graph::DataGraph, rewriting::rule::Rule};

use super::TermRewritingSystem;

impl Rule {
    /// Checks if `self` depends on `other`, i.e. if application of `other` may cause `self`
    /// to be applicable at new positions. There are two cases where this may happen.
    /// Let `L -> R` be `self` and `L' -> R'` be `other`.
    ///
    /// Case 1: (`L` contained in `R'`)
    /// There is a substitution `σ` and position `ρ` such that `L[σ] = R'[σ]_p`,
    /// but there is no position `λ` such that `L[σ] = L'[σ]_p`, i.e.
    /// `L` matches a subexpression of `R` with some variable assignment, but no
    /// subexpression of `L'` with the same variable assignment.
    ///
    /// Case 2: (`R'` contained in `L`)
    /// There is a substitution `σ` and position `ρ` such that `R'[σ] = L[σ]_p`,
    /// but `L'[σ] != L[σ]_p`, i.e. `R'` matches a subexpression of `L`,
    /// but `L'` does not match the same subexpression.
    ///
    /// Case 3: (Parial matching)
    /// not possible when expressions are trees. But this case is possible in
    /// general monoidal categories (e-hypergraphs or other things like that).
    fn depends_on(&self, other: &Rule) -> bool {
        true
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
