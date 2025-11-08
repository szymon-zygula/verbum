use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::egraph::{Analysis, EGraph};

/// The `Scheduler` trait defines a strategy for choosing which rule to try next
/// and applies it using the provided matcher. It returns the number of
/// applications performed in this step (0 means no rule applied, i.e., saturated).
pub trait Scheduler<A: Analysis> {
    fn apply_next(&mut self, egraph: &mut EGraph<A>, matcher: &dyn Matcher) -> usize;
}

pub mod cost_directed;
pub mod round_robin;

pub use cost_directed::CostDirectedScheduler;
pub use round_robin::RoundRobinScheduler;
