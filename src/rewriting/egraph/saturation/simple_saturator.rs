use crate::rewriting::rule::Rule;

use super::{Analysis, EGraph, SaturationConfig, SaturationStopReason, Saturator};
use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::egraph::saturation::scheduled_saturator::ScheduledSaturator;
use crate::rewriting::egraph::saturation::scheduler::RoundRobinScheduler;

pub struct SimpleSaturator {
    matcher: Box<dyn Matcher>,
}

impl SimpleSaturator {
    pub fn new(matcher: Box<dyn Matcher>) -> Self {
        Self { matcher }
    }
}

impl<A: Analysis> Saturator<A> for SimpleSaturator {
    fn saturate(
        &self,
        egraph: &mut EGraph<A>,
        rules: &[Rule],
        config: &SaturationConfig,
    ) -> SaturationStopReason {
        let scheduler = Box::new(RoundRobinScheduler::new());
        let mut saturator = ScheduledSaturator::new(scheduler);
        saturator.run(egraph, rules, config, &*self.matcher)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        language::Language,
        rewriting::{
            egraph::{DynEGraph, EGraph, matching::bottom_up::BottomUpMatcher},
            rule::Rule,
        },
    };

    use super::SimpleSaturator;
    use super::{SaturationConfig, SaturationStopReason, Saturator};

    #[test]
    fn classical_test() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang),
            Rule::from_strings("(* $0 1)", "$0", &lang),
            Rule::from_strings("(/ (* $0 $1) $2)", "(* $0 (/ $1 $2))", &lang),
            Rule::from_strings("(/ $0 $0)", "1", &lang),
        ];

        let mut egraph =
            EGraph::<()>::from_expression(lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap());

        let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
        let reason = saturator.saturate(&mut egraph, &rules, &SaturationConfig::default());
        assert_eq!(reason, SaturationStopReason::Saturated);

        assert_eq!(egraph.class_count(), 5);
        assert_eq!(egraph.actual_node_count(), 9);
    }

    #[test]
    fn stops_on_max_applications() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang),
            Rule::from_strings("(* $0 1)", "$0", &lang),
        ];
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("(* 3 2)").unwrap());

        let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
        let reason = saturator.saturate(
            &mut egraph,
            &rules,
            &SaturationConfig {
                max_applications: Some(1),
                ..Default::default()
            },
        );
        assert_eq!(reason, SaturationStopReason::MaxApplications);
    }

    #[test]
    fn stops_on_timeout() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang),
            Rule::from_strings("(* $0 1)", "$0", &lang),
        ];
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("(* 3 2)").unwrap());

        let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
        let reason = saturator.saturate(
            &mut egraph,
            &rules,
            &SaturationConfig {
                time_limit: Some(Duration::from_millis(0)),
                ..Default::default()
            },
        );
        assert_eq!(reason, SaturationStopReason::Timeout);
    }

    #[test]
    fn stops_on_max_nodes() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang),
            Rule::from_strings("(* $0 1)", "$0", &lang),
        ];
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("(* 3 2)").unwrap());

        let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
        let reason = saturator.saturate(
            &mut egraph,
            &rules,
            &SaturationConfig {
                max_nodes: Some(4),
                ..Default::default()
            },
        );
        assert_eq!(reason, SaturationStopReason::MaxNodes);
    }

    #[test]
    fn stops_on_max_classes() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang),
            Rule::from_strings("(* $0 1)", "$0", &lang),
        ];
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("(* 3 2)").unwrap());

        let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
        let reason = saturator.saturate(
            &mut egraph,
            &rules,
            &SaturationConfig {
                max_classes: Some(3),
                ..Default::default()
            },
        );
        assert_eq!(reason, SaturationStopReason::MaxClasses);
    }
}
