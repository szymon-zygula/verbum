use super::super::Analysis;
use crate::rewriting::egraph::EGraph;
use crate::rewriting::egraph::saturation::scheduler::Scheduler;
use crate::rewriting::rule::Rule;

pub struct ScheduledSaturator<A> {
    scheduler: Box<dyn Scheduler<A>>,
}

impl<A: Analysis> ScheduledSaturator<A> {
    pub fn new(scheduler: Box<dyn Scheduler<A>>) -> Self {
        Self { scheduler }
    }

    pub fn saturate(&mut self, egraph: &mut EGraph<A>, rules: &[Rule]) {
        while self.scheduler.apply_rule(egraph, rules) {
            // Loop until the scheduler indicates no more rules can be applied.
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::{Language, expression::Literal};
    use crate::rewriting::egraph::{DynEGraph, Node, matching::top_down::TopDownMatcher};

    // A simple scheduler for testing purposes
    struct TestScheduler {
        iterations: usize,
    }

    impl TestScheduler {
        fn new(iterations: usize) -> Self {
            Self { iterations }
        }
    }

    impl<A: Analysis> Scheduler<A> for TestScheduler {
        fn apply_rule(&mut self, egraph: &mut EGraph<A>, rules: &[Rule]) -> bool {
            if self.iterations == 0 {
                return false;
            }
            self.iterations -= 1;

            let mut changed = false;
            for rule in rules {
                changed |= rule.apply(egraph, &TopDownMatcher);
            }
            changed
        }
    }

    #[test]
    fn test_scheduled_saturator_basic() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("1").unwrap());
        let rule = Rule::from_strings("1", "2", &lang);
        let rules = vec![rule];

        let scheduler = Box::new(TestScheduler::new(1)); // Apply rules once
        let mut saturator = ScheduledSaturator::new(scheduler);

        saturator.saturate(&mut egraph, &rules);

        assert_eq!(egraph.class_count(), 1);
        assert_eq!(egraph.actual_node_count(), 2);
        assert!(egraph.node_id(&Node::Literal(Literal::Int(1))).is_some());
        assert!(egraph.node_id(&Node::Literal(Literal::Int(2))).is_some());
    }

    #[test]
    fn test_scheduled_saturator_multiple_iterations() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("1").unwrap());
        let rule1 = Rule::from_strings("1", "2", &lang);
        let rule2 = Rule::from_strings("2", "3", &lang);
        let rules = vec![rule1, rule2];

        let scheduler = Box::new(TestScheduler::new(2)); // Apply rules twice
        let mut saturator = ScheduledSaturator::new(scheduler);

        saturator.saturate(&mut egraph, &rules);

        assert_eq!(egraph.class_count(), 1);
        assert_eq!(egraph.actual_node_count(), 3);
        assert!(egraph.node_id(&Node::Literal(Literal::Int(1))).is_some());
        assert!(egraph.node_id(&Node::Literal(Literal::Int(2))).is_some());
        assert!(egraph.node_id(&Node::Literal(Literal::Int(3))).is_some());
    }
}
