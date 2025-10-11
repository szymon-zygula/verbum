use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::egraph::{Analysis, EGraph};
use crate::rewriting::rule::Rule;

use super::Scheduler;

/// Round-robin scheduler that cycles through rules and applies the first
/// applicable rule. It advances the starting index after each successful step.
pub struct RoundRobinScheduler {
    rules: Vec<Rule>,
    next_index: usize,
}

impl RoundRobinScheduler {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self {
            rules,
            next_index: 0,
        }
    }
}

impl<A: Analysis> Scheduler<A> for RoundRobinScheduler {
    fn apply_next(&mut self, egraph: &mut EGraph<A>, matcher: &dyn Matcher) -> usize {
        let n = self.rules.len();

        for offset in 0..n {
            let idx = (self.next_index + offset) % n;
            let applied = self.rules[idx].apply(egraph, matcher);
            if applied > 0 {
                self.next_index = (idx + 1) % n;
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
    use crate::rewriting::egraph::EGraph;
    use crate::rewriting::egraph::matching::top_down::TopDownMatcher;
    use crate::rewriting::egraph::saturation::scheduler::Scheduler;
    use crate::rewriting::rule::Rule;

    use super::RoundRobinScheduler;

    #[test]
    fn round_robin_basic_progress() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("1").unwrap());
        let rules = rules![
            &lang;
            "1" => "2",
            "2" => "3",
            "3" => "4",
        ];

        let mut sched = RoundRobinScheduler::new(rules);

        // Step 1: apply 1 -> 2
        let applied1 = sched.apply_next(&mut egraph, &TopDownMatcher);
        assert_eq!(applied1, 1);
        // Step 2: apply 2 -> 3
        let applied2 = sched.apply_next(&mut egraph, &TopDownMatcher);
        assert_eq!(applied2, 1);
        // Step 3: apply 3 -> 4
        let applied3 = sched.apply_next(&mut egraph, &TopDownMatcher);
        assert_eq!(applied3, 1);
        // Step 4: saturated for these rules
        let applied4 = sched.apply_next(&mut egraph, &TopDownMatcher);
        assert_eq!(applied4, 0);
    }

    #[test]
    fn round_robin_no_rules_returns_zero() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("2").unwrap());
        let rules: Vec<Rule> = vec![];

        let mut sched = RoundRobinScheduler::new(rules);
        let applied = sched.apply_next(&mut egraph, &TopDownMatcher);
        assert_eq!(applied, 0);
    }
}
