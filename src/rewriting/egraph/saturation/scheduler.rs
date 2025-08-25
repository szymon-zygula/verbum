use super::super::Analysis;
use crate::rewriting::egraph::EGraph;
use crate::rewriting::rule::Rule;

/// The `Scheduler` trait defines a strategy for applying rewrite rules to an e-graph.
/// Implementors of this trait can control the order and conditions under which rules
/// are applied during the equality saturation process.
pub trait Scheduler<A: Analysis> {
    /// Applies a set of rewrite rules to the given e-graph.
    ///
    /// This method is called repeatedly by the `ScheduledSaturator` until it returns `false`,
    /// indicating that no more rules can be applied or a saturation condition has been met.
    ///
    /// `true` if any rules were successfully applied or if the saturation process should continue;
    /// `false` if no rules were applied and the saturation process should stop.
    fn apply_rule(&mut self, egraph: &mut EGraph<A>, rules: &[Rule]) -> bool;
}
