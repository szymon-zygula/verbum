use std::time::{Duration, Instant};
use tabled::Tabled;
use serde::Serialize;

use crate::language::expression::VarFreeExpression;
use crate::rewriting::egraph::matching::Matcher;
use crate::rewriting::egraph::saturation::SaturationConfig;
use crate::rewriting::egraph::saturation::scheduler::Scheduler;
use crate::rewriting::egraph::{Analysis, DynEGraph};
use crate::rewriting::reachability::{ReachabilityStopReason, terms_reachable};
use crate::rewriting::rule::Rule;
use super::formatter::{Formattable, format_duration};

fn format_reachability_stop_reason(reason: &ReachabilityStopReason) -> String {
    format!("{:?}", reason)
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u128(duration.as_nanos())
}

fn serialize_reachability_stop_reason<S>(reason: &ReachabilityStopReason, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("{:?}", reason))
}

fn serialize_expr<S>(expr: &VarFreeExpression, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&expr.to_string())
}

#[derive(Clone, Debug, Tabled, Serialize)]
pub struct ReachabilityOutcome {
    #[tabled(rename = "Expr A")]
    #[serde(rename = "Expr A", serialize_with = "serialize_expr")]
    pub expr_a: VarFreeExpression,
    #[tabled(rename = "Expr B")]
    #[serde(rename = "Expr B", serialize_with = "serialize_expr")]
    pub expr_b: VarFreeExpression,
    #[tabled(rename = "Time", display_with = "format_duration")]
    #[serde(rename = "Time (ns)", serialize_with = "serialize_duration")]
    pub time: Duration,
    #[tabled(rename = "Stop Reason", display_with = "format_reachability_stop_reason")]
    #[serde(rename = "Stop Reason", serialize_with = "serialize_reachability_stop_reason")]
    pub stop_reason: ReachabilityStopReason,
    #[tabled(rename = "Applications(avg)")]
    #[serde(rename = "Applications(avg)")]
    pub applications: usize,
    #[tabled(rename = "Nodes")]
    #[serde(rename = "Nodes")]
    pub nodes: usize,
    #[tabled(rename = "Classes")]
    #[serde(rename = "Classes")]
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
    fn calculate_averages(items: &[Self]) -> Option<String> {
        if items.is_empty() {
            return None;
        }

        let num_outcomes = items.len() as u64;
        let mut total_time = Duration::default();
        let mut total_apps: u64 = 0;
        let mut total_nodes: u64 = 0;
        let mut total_classes: u64 = 0;

        for outcome in items {
            total_time += outcome.time;
            total_apps += outcome.applications as u64;
            total_nodes += outcome.nodes as u64;
            total_classes += outcome.classes as u64;
        }

        let avg_time = total_time / num_outcomes.min(u32::MAX as u64) as u32;
        let avg_apps = total_apps / num_outcomes;
        let avg_nodes = total_nodes / num_outcomes;
        let avg_classes = total_classes / num_outcomes;

        // Create a formatted average row
        use tabled::{Table, settings::Style};
        
        #[derive(Tabled)]
        struct AverageRow {
            #[tabled(rename = "Expr A")]
            label: String,
            #[tabled(rename = "Expr B")]
            empty1: String,
            #[tabled(rename = "Time")]
            time: String,
            #[tabled(rename = "Stop Reason")]
            empty2: String,
            #[tabled(rename = "Applications(avg)")]
            applications: u64,
            #[tabled(rename = "Nodes")]
            nodes: u64,
            #[tabled(rename = "Classes")]
            classes: u64,
        }

        let avg_row = AverageRow {
            label: "AVERAGE".to_string(),
            empty1: String::new(),
            time: format!("{:?}", avg_time),
            empty2: String::new(),
            applications: avg_apps,
            nodes: avg_nodes,
            classes: avg_classes,
        };

        let mut table = Table::new(vec![avg_row]);
        table.with(Style::rounded());
        Some(table.to_string())
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
