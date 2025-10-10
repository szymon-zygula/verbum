use crate::rewriting::{
    egraph::{EGraph, class::local_cost::LocalCost, matching::Matcher},
    rule::Rule,
};

use super::{SaturationConfig, SaturationStopReason, Saturator};
use crate::rewriting::egraph::saturation::scheduled_saturator::ScheduledSaturator;
use crate::rewriting::egraph::saturation::scheduler::CostDirectedScheduler;

pub struct DirectedSaturator {
    matcher: Box<dyn Matcher>,
}

impl DirectedSaturator {
    pub fn new(matcher: Box<dyn Matcher>) -> Self {
        Self { matcher }
    }
}

impl<LC: LocalCost + 'static> Saturator<LC> for DirectedSaturator {
    fn saturate(
        &self,
        egraph: &mut EGraph<LC>,
        rules: &[Rule],
        config: &SaturationConfig,
    ) -> SaturationStopReason {
        let scheduler = Box::new(CostDirectedScheduler::<LC>::new());
        let mut saturator = ScheduledSaturator::new(scheduler);
        saturator.run(egraph, rules, config, &*self.matcher)
    }
}

#[cfg(test)]
mod tests {
    use super::DirectedSaturator;
    use crate::language::Language;
    use crate::macros::rules;
    use crate::rewriting::egraph::class::simple_math_local_cost::SimpleMathLocalCost;
    use crate::rewriting::egraph::matching::bottom_up::BottomUpMatcher;
    use crate::rewriting::egraph::saturation::{SaturationConfig, SaturationStopReason, Saturator};
    use crate::rewriting::egraph::{DynEGraph, EGraph};
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

    fn make_egraph(lang: &Language, expr: &str) -> EGraph<SimpleMathLocalCost> {
        EGraph::from_expression(lang.parse_no_vars(expr).unwrap())
    }

    fn saturate(
        egraph: &mut EGraph<SimpleMathLocalCost>,
        rules: &[Rule],
        config: &SaturationConfig,
    ) -> SaturationStopReason {
        let saturator = DirectedSaturator::new(Box::new(BottomUpMatcher));
        saturator.saturate(egraph, rules, config)
    }

    // Helpers to reduce repetition in assertions and runs
    fn class_of(egraph: &mut EGraph<SimpleMathLocalCost>, lang: &Language, expr: &str) -> usize {
        let id = egraph.add_expression(lang.parse_no_vars(expr).unwrap());
        egraph.canonical_class(id)
    }

    fn assert_equivalent(
        egraph: &mut EGraph<SimpleMathLocalCost>,
        lang: &Language,
        a: &str,
        b: &str,
    ) {
        assert_eq!(class_of(egraph, lang, a), class_of(egraph, lang, b));
    }

    fn run(
        lang: &Language,
        start_expr: &str,
        rules: &[Rule],
        config: &SaturationConfig,
    ) -> (EGraph<SimpleMathLocalCost>, SaturationStopReason) {
        let mut egraph = make_egraph(lang, start_expr);
        let reason = saturate(&mut egraph, rules, config);
        (egraph, reason)
    }

    #[test]
    fn test_directed_saturator_simple_math() {
        let lang = Language::simple_math();
        let rules = simple_math_rules(&lang);
        let config = SaturationConfig::default();

        let (mut egraph, reason) = run(&lang, "(/ (* (sin 5) 2) 2)", &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        assert_equivalent(&mut egraph, &lang, "(/ (* (sin 5) 2) 2)", "(sin 5)");
    }

    #[test]
    fn test_directed_saturator_no_rules() {
        let lang = Language::simple_math();
        let rules: Vec<Rule> = vec![];
        let config = SaturationConfig::default();

        let (egraph, reason) = run(&lang, "(+ 1 2)", &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        // Ensure no changes occurred
        assert_eq!(egraph.total_node_count(), 3);
    }

    #[test]
    fn test_directed_saturator_no_changes() {
        let lang = Language::simple_math();
        let rules = simple_math_rules(&lang);
        let config = SaturationConfig::default();

        let (egraph, reason) = run(&lang, "(sin 5)", &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        // Ensure no changes occurred
        assert_eq!(egraph.total_node_count(), 2);
    }

    #[test]
    fn test_directed_saturator_multiple_applications_same_rule() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(+ $0 $0)" => "(* $0 2)", // x + x => x * 2
        );
        let config = SaturationConfig::default();

        let (mut egraph, reason) = run(&lang, "(+ (+ 1 1) (+ 1 1))", &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        assert_equivalent(&mut egraph, &lang, "(+ (+ 1 1) (+ 1 1))", "(* (* 1 2) 2)");
    }

    #[test]
    fn test_directed_saturator_rules_enable_other_rules() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(+ $0 $0)" => "(* $0 2)", // x + x => x * 2
            "(* $0 2)" => "(<< $0 1)", // x * 2 => x << 1
        );
        let config = SaturationConfig::default();

        let (mut egraph, reason) = run(&lang, "(+ 1 1)", &rules, &config);
        assert_eq!(reason, SaturationStopReason::Saturated);

        assert_equivalent(&mut egraph, &lang, "(+ 1 1)", "(<< 1 1)");
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

        let (mut egraph, reason) = run(&lang, "(+ 1 1)", &rules, &config);
        assert_eq!(reason, SaturationStopReason::MaxApplications);

        // After 1 application, (+ 1 1) should become (* 1 2)
        assert_equivalent(&mut egraph, &lang, "(+ 1 1)", "(* 1 2)");
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

        let (egraph, reason) = run(&lang, "(+ 1 1)", &rules, &config);
        assert_eq!(reason, SaturationStopReason::Timeout);

        // Ensure no changes occurred
        assert_eq!(egraph.total_node_count(), 2);
    }
}
