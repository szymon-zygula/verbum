use std::marker::PhantomData;

use crate::rewriting::egraph::EGraph;
use crate::rewriting::egraph::class::local_cost::LocalCost;
use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::rule::Rule;

use super::Scheduler;

/// Computes the local-cost delta of a rule for a given `LocalCost` analysis.
pub fn rule_cost<LC: LocalCost>(rule: &Rule) -> LC {
    LC::expression_cost(rule.to()) - LC::expression_cost(rule.from())
}

/// Cost-directed scheduler that orders rules by `rule_cost` (ascending) and
/// applies the first rule that makes progress in each step.
pub struct CostDirectedScheduler<LC: LocalCost> {
    rules: Vec<Rule>,
    _phantom: PhantomData<LC>,
}

impl<LC: LocalCost> CostDirectedScheduler<LC> {
    pub fn new(mut rules: Vec<Rule>) -> Self {
        rules.sort_by_key(|a| rule_cost::<LC>(a));
        Self {
            rules,
            _phantom: PhantomData,
        }
    }
}

impl<LC: LocalCost> Scheduler<LC> for CostDirectedScheduler<LC> {
    fn apply_next(&mut self, egraph: &mut EGraph<LC>, matcher: &dyn Matcher) -> usize {
        for rule in self.rules.iter() {
            let applied = rule.apply(egraph, matcher);
            if applied > 0 {
                return applied;
            }
        }

        0
    }
}

#[cfg(test)]
mod tests {
    use crate::language::Language;
    use crate::macros::rules;
    use crate::rewriting::egraph::class::simple_math_local_cost::SimpleMathLocalCost;
    use crate::rewriting::egraph::matching::top_down::TopDownMatcher;
    use crate::rewriting::egraph::saturation::scheduler::Scheduler;
    use crate::rewriting::egraph::{DynEGraph, EGraph};

    use super::CostDirectedScheduler;

    #[test]
    fn cost_directed_prefers_lower_cost_delta() {
        let lang = Language::simple_math();
        // Start from an expression matching both rules
        let mut egraph =
            EGraph::<SimpleMathLocalCost>::from_expression(lang.parse_no_vars("(+ 3 0)").unwrap());

        // Put the more expensive rule first to ensure ordering is by cost, not input order.
        // Costs with SimpleMathLocalCost:
        //   from: (+ $0 0) => cost = 1 ("+") + 0 (var) + 1 (literal) = 2
        //   to1:  (* $0 1) => 4 ("*") + 0 (var) + 1 (literal) = 5   => delta = +3
        //   to2:  $0        => 0                                   => delta = -2
        let rules = rules![
            &lang;
            "(+ $0 0)" => "(* $0 1)", // expensive
            "(+ $0 0)" => "$0",        // cheap
        ];

        let mut sched = CostDirectedScheduler::<SimpleMathLocalCost>::new(rules);
        let applied = sched.apply_next(&mut egraph, &TopDownMatcher);
        assert_eq!(applied, 1, "scheduler should make progress on first step");

        // The first step should choose the cheaper rewrite, so no '*' nodes should be introduced yet.
        let star_nodes = egraph.find_symbols(lang.get_id("*"));
        assert!(
            star_nodes.is_empty(),
            "expensive '*' rewrite should not be chosen first"
        );
    }
}
