use std::time::{Duration, Instant};
use tabled::Tabled;

use crate::language::expression::VarFreeExpression;
use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::egraph::saturation::SaturationConfig;
use crate::rewriting::egraph::saturation::scheduler::Scheduler;
use crate::rewriting::egraph::{Analysis, DynEGraph};
use crate::rewriting::reachability::{ReachabilityStopReason, terms_reachable};
use crate::rewriting::rule::Rule;
use super::formatter::{Formattable, format_duration, format_duration_csv};

fn format_reachability_stop_reason(reason: &ReachabilityStopReason) -> String {
    format!("{:?}", reason)
}

#[derive(Clone, Debug, Tabled)]
pub struct ReachabilityOutcome {
    #[tabled(rename = "Expr A")]
    pub expr_a: VarFreeExpression,
    #[tabled(rename = "Expr B")]
    pub expr_b: VarFreeExpression,
    #[tabled(rename = "Time", display_with = "format_duration")]
    pub time: Duration,
    #[tabled(rename = "Stop Reason", display_with = "format_reachability_stop_reason")]
    pub stop_reason: ReachabilityStopReason,
    #[tabled(rename = "Applications(avg)")]
    pub applications: usize,
    #[tabled(rename = "Nodes")]
    pub nodes: usize,
    #[tabled(rename = "Classes")]
    pub classes: usize,
}

/// Run a single reachability attempt with a custom scheduler factory.
pub fn run_single_with_scheduler<A: Analysis, F>(
    rules: &[Rule],
    expr_a: VarFreeExpression,
    expr_b: VarFreeExpression,
    cfg: &SaturationConfig,
    matcher: &dyn Matcher,
    build_scheduler: F,
) -> ReachabilityOutcome
where
    F: Clone + FnOnce(&[Rule]) -> Box<dyn Scheduler<A>>,
{
    let start = Instant::now();
    let res = terms_reachable(
        rules,
        expr_a.clone(),
        expr_b.clone(),
        cfg,
        matcher,
        build_scheduler.clone(),
    );
    let time = start.elapsed();
    let nodes = res.egraph.actual_node_count();
    let classes = res.egraph.class_count();

    ReachabilityOutcome {
        expr_a,
        expr_b,
        time,
        stop_reason: res.reason,
        applications: res.applications,
        nodes,
        classes,
    }
}

/// Generic benchmarking over pairs using a custom scheduler factory closure.
pub fn benchmark_pairs_with_scheduler<A: Analysis, F>(
    rules: &[Rule],
    pairs: &[(VarFreeExpression, VarFreeExpression)],
    cfg: &SaturationConfig,
    matcher: &dyn Matcher,
    runs: usize,
    build_scheduler: F,
) -> Vec<ReachabilityOutcome>
where
    F: Clone + FnOnce(&[Rule]) -> Box<dyn Scheduler<A>>,
{
    let mut out = Vec::with_capacity(pairs.len());
    for (a, b) in pairs {
        let mut collected = Vec::with_capacity(runs + 1);
        for _ in 0..(runs + 1) {
            collected.push(run_single_with_scheduler::<A, F>(
                rules,
                a.clone(),
                b.clone(),
                cfg,
                matcher,
                build_scheduler.clone(),
            ));
        }
        let mut avg = collected.remove(0); // warm-up discard
        for c in collected {
            assert_eq!(avg.stop_reason, c.stop_reason);
            assert_eq!(avg.nodes, c.nodes);
            assert_eq!(avg.classes, c.classes);
            avg.time += c.time;
            avg.applications += c.applications;
        }
        avg.time /= runs as u32;
        avg.applications /= runs;
        out.push(avg);
    }
    out
}

impl Formattable for ReachabilityOutcome {
    fn to_csv_row(&self) -> Vec<String> {
        vec![
            self.expr_a.to_string(),
            self.expr_b.to_string(),
            format_duration_csv(&self.time),
            format!("{:?}", self.stop_reason),
            self.applications.to_string(),
            self.nodes.to_string(),
            self.classes.to_string(),
        ]
    }

    fn csv_headers() -> Vec<&'static str> {
        vec![
            "Expr A",
            "Expr B",
            "Time (ns)",
            "Stop Reason",
            "Applications(avg)",
            "Nodes",
            "Classes",
        ]
    }
}

#[cfg(test)]
mod tests {
    /// Benchmark reachability for multiple expression pairs.
    /// Performs `runs + 1` executions (dropping the first as warm-up) and averages time/applications.
    pub fn benchmark_pairs<A: Analysis>(
        rules: &[Rule],
        pairs: &[(VarFreeExpression, VarFreeExpression)],
        cfg: &SaturationConfig,
        matcher: &dyn Matcher,
        runs: usize,
    ) -> Vec<ReachabilityOutcome> {
        benchmark_pairs_with_scheduler::<(), _>(rules, pairs, cfg, matcher, runs, |rs| {
            Box::new(RoundRobinScheduler::new(rs.to_vec()))
        })
    }

    use super::*;
    use crate::language::Language;
    use crate::macros::rules;
    use crate::rewriting::egraph::matching::top_down::TopDownMatcher;
    use crate::rewriting::egraph::saturation::scheduler::RoundRobinScheduler;

    #[test]
    fn identical_expressions_unify_immediately() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("1").unwrap();
        let cfg = SaturationConfig::default();
        let outcomes = benchmark_pairs::<()>(
            &[],
            &[(expr.clone(), expr.clone())],
            &cfg,
            &TopDownMatcher,
            3,
        );
        assert_eq!(outcomes.len(), 1);
        let o = &outcomes[0];
        match &o.stop_reason {
            ReachabilityStopReason::ReachedCommonForm { .. } => {}
            other => panic!("expected ReachedCommonForm, got {other:?}"),
        }
        assert_eq!(o.applications, 0);
    }

    #[test]
    fn unifies_via_rewrites() {
        let lang = Language::simple_math();
        let rules = rules!(lang; "(+ $0 0)" => "$0");
        let a = lang.parse_no_vars("(+ 1 0)").unwrap();
        let b = lang.parse_no_vars("1").unwrap();
        let cfg = SaturationConfig::default();
        let outcomes = benchmark_pairs::<()>(&rules, &[(a, b)], &cfg, &TopDownMatcher, 3);
        assert_eq!(outcomes.len(), 1);
        let o = &outcomes[0];
        match &o.stop_reason {
            ReachabilityStopReason::ReachedCommonForm { .. } => {}
            other => panic!("expected unification, got {other:?}"),
        }
        assert!(o.applications >= 1);
    }

    #[test]
    fn saturates_without_unification() {
        let lang = Language::simple_math();
        let rules = rules!(lang; "(+ $0 0)" => "$0");
        let a = lang.parse_no_vars("(* 2 3)").unwrap();
        let b = lang.parse_no_vars("4").unwrap();
        let cfg = SaturationConfig::default();
        let outcomes = benchmark_pairs::<()>(&rules, &[(a, b)], &cfg, &TopDownMatcher, 3);
        assert_eq!(outcomes.len(), 1);
        let o = &outcomes[0];
        assert_eq!(
            o.stop_reason,
            ReachabilityStopReason::SaturatedNoUnification
        );
    }

    #[test]
    fn respects_max_applications() {
        let lang = Language::simple_math();
        let rules = rules!(lang; "1" => "2", "2" => "3");
        let a = lang.parse_no_vars("1").unwrap();
        let b = lang.parse_no_vars("3").unwrap();
        let cfg = SaturationConfig {
            max_applications: Some(1),
            ..Default::default()
        };
        let outcomes = benchmark_pairs::<()>(&rules, &[(a, b)], &cfg, &TopDownMatcher, 3);
        assert_eq!(outcomes.len(), 1);
        let o = &outcomes[0];
        match &o.stop_reason {
            ReachabilityStopReason::Limit(lim) => assert_eq!(
                *lim,
                crate::rewriting::egraph::saturation::SaturationStopReason::MaxApplications
            ),
            other => panic!("expected limit stop, got {other:?}"),
        }
    }
}
