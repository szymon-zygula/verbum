use std::marker::PhantomData;

use itertools::Itertools;

use crate::rewriting::egraph::EGraph;
use crate::rewriting::egraph::class::local_cost::LocalCost;
use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::rule::Rule;

use super::Scheduler;

/// Computes the cost of a rule for a given `LocalCost` analysis.
pub fn rule_cost<LC: LocalCost>(rule: &Rule) -> LC {
    LC::expression_cost(rule.to()) - LC::expression_cost(rule.from())
}

/// Cost-directed scheduler that orders rules by `rule_cost` (ascending) and
/// applies the first rule that makes progress in each step.
#[derive(Default)]
pub struct CostDirectedScheduler<LC: LocalCost> {
    _phantom: PhantomData<LC>,
}

impl<LC: LocalCost> CostDirectedScheduler<LC> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<LC: LocalCost> Scheduler<LC> for CostDirectedScheduler<LC> {
    fn apply_next(
        &mut self,
        egraph: &mut EGraph<LC>,
        rules: &[Rule],
        matcher: &dyn Matcher,
    ) -> usize {
        // Sort by increasing cost delta; recompute each step to keep it simple for now.
        // TODO: this should be computed only once of course.
        for rule in rules
            .iter()
            .map(|r| (r, rule_cost::<LC>(r)))
            .sorted_by(|(_, c1), (_, c2)| c1.cmp(c2))
            .map(|(r, _)| r)
        {
            let applied = rule.apply(egraph, matcher);
            if applied > 0 {
                return applied;
            }
        }

        0
    }
}
