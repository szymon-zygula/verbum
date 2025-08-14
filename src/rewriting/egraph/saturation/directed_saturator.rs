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

#[cfg(test)]
mod tests {
    use super::DirectedSaturator;
    use crate::language::Language;
    use crate::macros::rules;
    use crate::rewriting::egraph::EGraph;
    use crate::rewriting::egraph::class::simple_math_local_cost::SimpleMathLocalCost;
    use crate::rewriting::egraph::matching::bottom_up::BottomUpMatcher;
    use crate::rewriting::egraph::saturation::{SaturationConfig, SaturationStopReason, Saturator};
    use crate::rewriting::rule::Rule;
    use std::time::Duration;

    fn simple_math_rules(lang: &Language) -> Vec<Rule> {
        rules!(lang;
            "(* $0 2)" => "(<< $0 1)",
            "(* $0 1)" => "$0",
            "(/ (* $0 $1) $2)" => "(* $0 (/ $1 $2))",
            "(/ $0 $0)" => "1",
        )
    }

    #[test]
    fn test_directed_saturator_simple_math() {
        let lang = Language::simple_math();
        let rules = simple_math_rules(&lang);
        let config = SaturationConfig::default();

        // Test case 1: (/ (* (sin 5) 2) 2) should become (sin 5)
        let mut egraph_1 = EGraph::<SimpleMathLocalCost>::from_expression(
            lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap(),
        );
        let initial_expr_1_root_id =
            egraph_1.add_expression(lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap());

        let saturator_1 = DirectedSaturator::new(BottomUpMatcher);
        let reason_1 = saturator_1.saturate(&mut egraph_1, &rules, &config);
        assert_eq!(reason_1, SaturationStopReason::Saturated);

        let expected_expr_1_root_id =
            egraph_1.add_expression(lang.parse_no_vars("(sin 5)").unwrap());
        assert_eq!(
            egraph_1.union_find.find(initial_expr_1_root_id),
            egraph_1.union_find.find(expected_expr_1_root_id)
        );
    }

    #[test]
    fn test_directed_saturator_no_rules() {
        let lang = Language::simple_math();
        let rules = vec![];
        let config = SaturationConfig::default();

        let mut egraph =
            EGraph::<SimpleMathLocalCost>::from_expression(lang.parse_no_vars("(+ 1 2)").unwrap());
        let initial_id = egraph.add_expression(lang.parse_no_vars("(+ 1 2)").unwrap());

        let saturator = DirectedSaturator::new(BottomUpMatcher);
        let reason = saturator.saturate(&mut egraph, &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        // Ensure no changes occurred
        let expected_id = egraph.add_expression(lang.parse_no_vars("(+ 1 2)").unwrap());
        assert_eq!(
            egraph.union_find.find(initial_id),
            egraph.union_find.find(expected_id)
        );
    }

    #[test]
    fn test_directed_saturator_no_changes() {
        let lang = Language::simple_math();
        let rules = simple_math_rules(&lang);
        let config = SaturationConfig::default();

        // E-graph is already in a saturated state for these rules
        let mut egraph =
            EGraph::<SimpleMathLocalCost>::from_expression(lang.parse_no_vars("(sin 5)").unwrap());
        let initial_id = egraph.add_expression(lang.parse_no_vars("(sin 5)").unwrap());

        let saturator = DirectedSaturator::new(BottomUpMatcher);
        let reason = saturator.saturate(&mut egraph, &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        // Ensure no changes occurred
        let expected_id = egraph.add_expression(lang.parse_no_vars("(sin 5)").unwrap());
        assert_eq!(
            egraph.union_find.find(initial_id),
            egraph.union_find.find(expected_id)
        );
    }

    #[test]
    fn test_directed_saturator_multiple_applications_same_rule() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(+ $0 $0)" => "(* $0 2)", // x + x => x * 2
        );
        let config = SaturationConfig::default();

        let mut egraph = EGraph::<SimpleMathLocalCost>::from_expression(
            lang.parse_no_vars("(+ (+ 1 1) (+ 1 1))").unwrap(),
        );
        let initial_id = egraph.add_expression(lang.parse_no_vars("(+ (+ 1 1) (+ 1 1))").unwrap());

        let saturator = DirectedSaturator::new(BottomUpMatcher);
        let reason = saturator.saturate(&mut egraph, &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        // (+ (+ 1 1) (+ 1 1)) should become (* (* 1 2) 2)
        let expected_expr = lang.parse_no_vars("(* (* 1 2) 2)").unwrap();
        let expected_id = egraph.add_expression(expected_expr.clone());
        assert_eq!(
            egraph.union_find.find(initial_id),
            egraph.union_find.find(expected_id)
        );
    }

    #[test]
    fn test_directed_saturator_rules_enable_other_rules() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(+ $0 $0)" => "(* $0 2)", // x + x => x * 2
            "(* $0 2)" => "(<< $0 1)", // x * 2 => x << 1
        );
        let config = SaturationConfig::default();

        let mut egraph =
            EGraph::<SimpleMathLocalCost>::from_expression(lang.parse_no_vars("(+ 1 1)").unwrap());
        let initial_id = egraph.add_expression(lang.parse_no_vars("(+ 1 1)").unwrap());

        let saturator = DirectedSaturator::new(BottomUpMatcher);
        let reason = saturator.saturate(&mut egraph, &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        // (+ 1 1) should become (1 << 1)
        let expected_expr = lang.parse_no_vars("(<< 1 1)").unwrap();
        let expected_id = egraph.add_expression(expected_expr.clone());
        assert_eq!(
            egraph.union_find.find(initial_id),
            egraph.union_find.find(expected_id)
        );
    }

    #[test]
    fn test_directed_saturator_max_applications_limit() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(+ $0 $0)" => "(* $0 2)", // x + x => x * 2
            "(* $0 2)" => "(<< $0 1)", // x * 2 => x << 1
        );
        let config = SaturationConfig {
            max_applications: Some(1),
            ..Default::default()
        };

        let mut egraph =
            EGraph::<SimpleMathLocalCost>::from_expression(lang.parse_no_vars("(+ 1 1)").unwrap());
        let initial_id = egraph.add_expression(lang.parse_no_vars("(+ 1 1)").unwrap());

        let saturator = DirectedSaturator::new(BottomUpMatcher);
        let reason = saturator.saturate(&mut egraph, &rules, &config);
        assert_eq!(reason, SaturationStopReason::MaxApplications);

        // After 1 application, (+ 1 1) should become (* 1 2)
        let expected_expr = lang.parse_no_vars("(* 1 2)").unwrap();
        let expected_id = egraph.add_expression(expected_expr.clone());
        assert_eq!(
            egraph.canonical_class(initial_id),
            egraph.canonical_class(expected_id)
        );
    }

    #[test]
    fn test_directed_saturator_time_limit() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(+ $0 $0)" => "(* $0 2)", // x + x => x * 2
            "(* $0 2)" => "(<< $0 1)", // x * 2 => x << 1
        );
        let config = SaturationConfig {
            time_limit: Some(Duration::from_nanos(0)), // Very short time limit
            ..Default::default()
        };

        let mut egraph =
            EGraph::<SimpleMathLocalCost>::from_expression(lang.parse_no_vars("(+ 1 1)").unwrap());

        let saturator = DirectedSaturator::new(BottomUpMatcher);
        let reason = saturator.saturate(&mut egraph, &rules, &config);
        assert_eq!(reason, SaturationStopReason::Timeout);

        // The expression might not be fully simplified due to time limit
        // We just assert that it stopped due to timeout.
    }
}
