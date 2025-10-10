use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::egraph::{Analysis, EGraph};
use crate::rewriting::rule::Rule;

use super::Scheduler;

/// Round-robin scheduler that cycles through rules and applies the first
/// applicable rule. It advances the starting index after each successful step.
#[derive(Default)]
pub struct RoundRobinScheduler {
    next_index: usize,
}

impl RoundRobinScheduler {
    pub fn new() -> Self {
        Self { next_index: 0 }
    }
}

impl<A: Analysis> Scheduler<A> for RoundRobinScheduler {
    fn apply_next(
        &mut self,
        egraph: &mut EGraph<A>,
        rules: &[Rule],
        matcher: &dyn Matcher,
    ) -> usize {
        let n = rules.len();

        for offset in 0..n {
            let idx = (self.next_index + offset) % n;
            let applied = rules[idx].apply(egraph, matcher);
            if applied > 0 {
                self.next_index = (idx + 1) % n;
                return applied;
            }
        }

        0
    }
}
