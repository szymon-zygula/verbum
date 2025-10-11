use std::time::Instant;

use super::super::Analysis;
use crate::rewriting::egraph::EGraph;
use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::egraph::saturation::scheduler::Scheduler;
use crate::rewriting::egraph::saturation::{SaturationConfig, SaturationStopReason, check_limits};

pub struct ScheduledSaturator<A> {
    scheduler: Box<dyn Scheduler<A>>,
}

impl<A: Analysis> ScheduledSaturator<A> {
    pub fn new(scheduler: Box<dyn Scheduler<A>>) -> Self {
        Self { scheduler }
    }

    pub fn run(
        &mut self,
        egraph: &mut EGraph<A>,
        config: &SaturationConfig,
        matcher: &dyn Matcher,
    ) -> SaturationStopReason {
        let start = Instant::now();
        let mut applications: usize = 0;

        loop {
            if let Some(reason) = check_limits(egraph, applications, start, config) {
                return reason;
            }

            let applied = self.scheduler.apply_next(egraph, matcher);
            if applied == 0 {
                return SaturationStopReason::Saturated;
            }

            applications += applied;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::{Language, expression::Literal};
    use crate::rewriting::egraph::matching::top_down::TopDownMatcher;
    use crate::rewriting::egraph::{DynEGraph, Node};
    use crate::rewriting::rule::Rule;

    // A simple scheduler for testing purposes
    struct TestScheduler {
        iterations: usize,
        rules: Vec<Rule>,
    }

    impl TestScheduler {
        fn new(iterations: usize, rules: Vec<Rule>) -> Self {
            Self { iterations, rules }
        }
    }

    impl<A: Analysis> Scheduler<A> for TestScheduler {
        fn apply_next(&mut self, egraph: &mut EGraph<A>, matcher: &dyn Matcher) -> usize {
            if self.iterations == 0 {
                return 0;
            }
            self.iterations -= 1;

            let mut total = 0usize;
            for rule in &self.rules {
                total += rule.apply(egraph, matcher);
            }
            total
        }
    }

    #[test]
    fn test_scheduled_saturator_basic() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("1").unwrap());
        let rules = vec![Rule::from_strings("1", "2", &lang)];

        let scheduler = Box::new(TestScheduler::new(1, rules)); // Apply rules once
        let mut saturator = ScheduledSaturator::new(scheduler);

        let reason = saturator.run(&mut egraph, &SaturationConfig::default(), &TopDownMatcher);
        assert_eq!(reason, SaturationStopReason::Saturated);

        assert_eq!(egraph.class_count(), 1);
        assert_eq!(egraph.actual_node_count(), 2);
        assert!(egraph.node_id(&Node::Literal(Literal::Int(1))).is_some());
        assert!(egraph.node_id(&Node::Literal(Literal::Int(2))).is_some());
    }

    #[test]
    fn test_scheduled_saturator_multiple_iterations() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("1").unwrap());
        let rules = vec![
            Rule::from_strings("1", "2", &lang),
            Rule::from_strings("2", "3", &lang),
        ];

        let scheduler = Box::new(TestScheduler::new(2, rules)); // Apply rules twice
        let mut saturator = ScheduledSaturator::new(scheduler);

        let reason = saturator.run(&mut egraph, &SaturationConfig::default(), &TopDownMatcher);
        assert_eq!(reason, SaturationStopReason::Saturated);

        assert_eq!(egraph.class_count(), 1);
        assert_eq!(egraph.actual_node_count(), 3);
        assert!(egraph.node_id(&Node::Literal(Literal::Int(1))).is_some());
        assert!(egraph.node_id(&Node::Literal(Literal::Int(2))).is_some());
        assert!(egraph.node_id(&Node::Literal(Literal::Int(3))).is_some());
    }
}
