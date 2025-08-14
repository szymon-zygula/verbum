use std::time::Instant;

use itertools::Itertools;

use crate::rewriting::{
    egraph::{class::local_cost::LocalCost, matching::Matcher},
    rule::Rule,
};

use super::{SaturationStopReason, Saturator, check_limits};

pub fn rule_cost<LC: LocalCost>(rule: &Rule) -> LC {
    LC::expression_cost(rule.to()) - LC::expression_cost(rule.from())
}

pub struct DirectedSaturator<M: Matcher> {
    matcher: M,
}

impl<M: Matcher> DirectedSaturator<M> {
    pub fn new(matcher: M) -> Self {
        Self { matcher }
    }
}

impl<LC: LocalCost, M: Matcher> Saturator<LC> for DirectedSaturator<M> {
    fn saturate(
        &self,
        egraph: &mut crate::rewriting::egraph::EGraph<LC>,
        rules: &[crate::rewriting::rule::Rule],
        config: &super::SaturationConfig,
    ) -> SaturationStopReason {
        let sorted_rules = rules
            .iter()
            .map(|rule| (rule, rule_cost::<LC>(rule)))
            .sorted_by(|(_, cost_1), (_, cost_2)| cost_1.cmp(cost_2))
            .map(|(rule, _)| rule)
            .collect_vec();

        let start = Instant::now();
        let mut applications: usize = 0;

        // Early check in case the initial graph already violates limits
        if let Some(reason) = check_limits(egraph, applications, start, config) {
            return reason;
        }

        loop {
            let mut any_applied = false;

            for rule in sorted_rules.iter() {
                // Check time on each rule iteration
                if let Some(SaturationStopReason::Timeout) =
                    check_limits(egraph, applications, start, config)
                {
                    return SaturationStopReason::Timeout;
                }

                if rule.apply(egraph, &self.matcher) {
                    applications += 1;
                    any_applied = true;

                    if let Some(reason) = check_limits(egraph, applications, start, config) {
                        return reason;
                    }

                    // Go back to the top of the list of rules to start with the best rules again
                    break;
                }
            }

            if !any_applied {
                return SaturationStopReason::Saturated;
            }
        }
    }
}
