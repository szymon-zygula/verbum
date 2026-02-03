//! Reachability analysis for term rewriting systems.
//!
//! This module provides functionality to determine if two expressions can be
//! made equivalent through the application of rewrite rules.

use std::time::{Duration, Instant};

use crate::language::expression::VarFreeExpression;
use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::egraph::saturation::scheduler::Scheduler;
use crate::rewriting::egraph::saturation::{SaturationConfig, SaturationStopReason, check_limits};
use crate::rewriting::egraph::{Analysis, ClassId, DynEGraph, EGraph};
use crate::rewriting::rule::Rule;

/// Reason why reachability analysis stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReachabilityStopReason {
    /// The two expressions were unified (ended up in the same equivalence class).
    ReachedCommonForm { class_id: ClassId },
    /// A configured resource limit was hit.
    Limit(SaturationStopReason),
    /// No more rule applications possible and the classes remain distinct.
    SaturatedNoUnification,
}

/// Result of a reachability analysis.
///
/// Contains the final e-graph, the reason for stopping, and statistics
/// about the analysis process.
#[derive(Clone)]
pub struct ReachabilityResult<A: Analysis> {
    /// The final e-graph after reachability analysis
    pub egraph: EGraph<A>,
    /// The reason why the analysis stopped
    pub reason: ReachabilityStopReason,
    /// The number of rule applications performed
    pub applications: usize,
    /// The duration of the analysis
    pub duration: Duration,
}

/// Check if two terms can reach a common form through equality saturation.
///
/// Uses a custom scheduler factory to control the order of rule applications.
///
/// # Arguments
///
/// * `rules` - The rewrite rules to apply
/// * `expr_a` - The first expression
/// * `expr_b` - The second expression
/// * `config` - Configuration for saturation limits
/// * `matcher` - The matcher to use for pattern matching
/// * `build_scheduler` - A function that creates a scheduler for the rules
///
/// # Returns
///
/// Returns a `ReachabilityResult` containing the outcome
pub fn terms_reachable<A, F>(
    rules: &[Rule],
    expr_a: VarFreeExpression,
    expr_b: VarFreeExpression,
    config: &SaturationConfig,
    matcher: &dyn Matcher,
    build_scheduler: F,
) -> ReachabilityResult<A>
where
    A: Analysis,
    F: FnOnce(&[Rule]) -> Box<dyn Scheduler<A>>,
{
    let start = Instant::now();

    // Build e-graph seeded with expr_a, then add expr_b and identify their classes.
    let (mut egraph, a_root_node_class) = EGraph::<A>::from_expression_with_id(expr_a);
    let b_root_node = egraph.add_expression(expr_b);
    let mut a_class = egraph.canonical_class(a_root_node_class);
    let mut b_class = egraph.canonical_class(egraph.containing_class(b_root_node));

    let mut scheduler = build_scheduler(rules);
    let mut applications = 0;

    let reason = loop {
        // Re-check canonical classes before attempting the next step.
        a_class = egraph.canonical_class(a_class);
        b_class = egraph.canonical_class(b_class);
        if a_class == b_class {
            break ReachabilityStopReason::ReachedCommonForm { class_id: a_class };
        }

        if let Some(limit) = check_limits(&egraph, applications, start, config) {
            break ReachabilityStopReason::Limit(limit);
        }

        let applied = scheduler.apply_next(&mut egraph, matcher);
        if applied == 0 {
            break ReachabilityStopReason::SaturatedNoUnification;
        }

        applications += applied;
    };

    ReachabilityResult {
        egraph,
        reason,
        applications,
        duration: start.elapsed(),
    }
}

/// Convenience wrapper: Round-Robin over `rules`.
pub fn terms_reachable_round_robin<A: Analysis>(
    rules: &[Rule],
    expr_a: VarFreeExpression,
    expr_b: VarFreeExpression,
    config: &SaturationConfig,
    matcher: &dyn Matcher,
) -> ReachabilityResult<A> {
    use crate::rewriting::egraph::saturation::scheduler::RoundRobinScheduler;
    terms_reachable(rules, expr_a, expr_b, config, matcher, |rs| {
        Box::new(RoundRobinScheduler::new(rs.to_vec()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use crate::macros::rules;
    use crate::rewriting::egraph::matching::top_down::TopDownMatcher;

    #[test]
    fn immediate_unification_without_rules() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("1").unwrap();
        let cfg = SaturationConfig::default();

        let res: ReachabilityResult<()> =
            terms_reachable_round_robin(&[], expr.clone(), expr, &cfg, &TopDownMatcher);

        match res.reason {
            ReachabilityStopReason::ReachedCommonForm { class_id: _ } => {}
            other => panic!("expected immediate unification, got {other:?}"),
        }
        assert_eq!(res.applications, 0);
    }

    #[test]
    fn unifies_via_rules() {
        let lang = Language::simple_math();
        let rules = rules!(lang; "(+ $0 0)" => "$0");
        let cfg = SaturationConfig::default();

        let expr_a = lang.parse_no_vars("(+ 1 0)").unwrap();
        let expr_b = lang.parse_no_vars("1").unwrap();

        let res: ReachabilityResult<()> =
            terms_reachable_round_robin(&rules, expr_a, expr_b, &cfg, &TopDownMatcher);

        match res.reason {
            ReachabilityStopReason::ReachedCommonForm { class_id: _ } => {}
            other => panic!("expected unification via rewrite, got {other:?}"),
        }
        assert!(res.applications >= 1);
    }

    #[test]
    fn saturates_no_unification() {
        let lang = Language::simple_math();
        let rules = rules!(lang; "(+ $0 0)" => "$0");
        let cfg = SaturationConfig::default();

        let expr_a = lang.parse_no_vars("(* 2 3)").unwrap();
        let expr_b = lang.parse_no_vars("4").unwrap();

        let res: ReachabilityResult<()> =
            terms_reachable_round_robin(&rules, expr_a, expr_b, &cfg, &TopDownMatcher);

        assert_eq!(res.reason, ReachabilityStopReason::SaturatedNoUnification);
        assert_eq!(res.applications, 0);
    }

    #[test]
    fn respects_max_applications_limit() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "1" => "2",
            "2" => "3",
        );
        let cfg = SaturationConfig {
            max_applications: Some(1),
            ..Default::default()
        };

        let expr_a = lang.parse_no_vars("1").unwrap();
        let expr_b = lang.parse_no_vars("3").unwrap();

        let res: ReachabilityResult<()> =
            terms_reachable_round_robin(&rules, expr_a, expr_b, &cfg, &TopDownMatcher);

        assert_eq!(
            res.reason,
            ReachabilityStopReason::Limit(SaturationStopReason::MaxApplications)
        );
        assert_eq!(res.applications, 1);
    }
}
