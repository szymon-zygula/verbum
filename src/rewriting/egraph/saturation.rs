use std::time::{Duration, Instant};

use crate::rewriting::rule::Rule;

use super::{EGraph, matching::Matcher};

#[derive(Clone, Debug, Default)]
pub struct SaturationConfig {
    pub max_nodes: Option<usize>,
    pub max_classes: Option<usize>,
    pub max_applications: Option<usize>,
    pub time_limit: Option<Duration>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SaturationStopReason {
    Saturated,
    MaxNodes,
    MaxClasses,
    MaxApplications,
    Timeout,
}

fn check_limits(
    egraph: &EGraph,
    applications: usize,
    start: Instant,
    cfg: &SaturationConfig,
) -> Option<SaturationStopReason> {
    if let Some(limit) = cfg.time_limit && start.elapsed() >= limit {
        return Some(SaturationStopReason::Timeout);
    }

    if let Some(limit) = cfg.max_applications && applications >= limit {
        return Some(SaturationStopReason::MaxApplications);
    }

    if let Some(limit) = cfg.max_nodes && egraph.actual_node_count() >= limit {
        return Some(SaturationStopReason::MaxNodes);
    }

    if let Some(limit) = cfg.max_classes && egraph.class_count() >= limit {
        return Some(SaturationStopReason::MaxClasses);
    }

    None
}

pub fn saturate(
    egraph: &mut EGraph,
    rules: &[Rule],
    matcher: impl Matcher,
    config: &SaturationConfig,
) -> SaturationStopReason {
    let start = Instant::now();
    let mut applications: usize = 0;

    // Early check in case initial graph already violates limits
    if let Some(reason) = check_limits(egraph, applications, start, config) {
        return reason;
    }

    loop {
        let mut any_applied = false;

        for rule in rules.iter() {
            // Check time on each rule iteration
            if let Some(SaturationStopReason::Timeout) = check_limits(egraph, applications, start, config) {
                return SaturationStopReason::Timeout;
            }

            if rule.apply(egraph, &matcher) {
                applications += 1;
                any_applied = true;

                if let Some(reason) = check_limits(egraph, applications, start, config) {
                    return reason;
                }
            }
        }

        if !any_applied {
            return SaturationStopReason::Saturated;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        language::Language,
        rewriting::{
            egraph::{EGraph, matching::bottom_up::BottomUpMatcher},
            rule::Rule,
        },
    };

    use super::{saturate, SaturationConfig, SaturationStopReason};

    #[test]
    fn classical_test() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* x0 2)", "(<< x0 1)", &lang),
            Rule::from_strings("(* x0 1)", "x0", &lang),
            Rule::from_strings("(/ (* x0 x1) x2)", "(* x0 (/ x1 x2))", &lang),
            Rule::from_strings("(/ x0 x0)", "1", &lang),
        ];

        let mut egraph =
            EGraph::from_expression(lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap());

        let reason = saturate(&mut egraph, &rules, BottomUpMatcher, &SaturationConfig::default());
        assert_eq!(reason, SaturationStopReason::Saturated);

        assert_eq!(egraph.class_count(), 5);
        assert_eq!(egraph.actual_node_count(), 9);
    }

    #[test]
    fn stops_on_max_applications() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* x0 2)", "(<< x0 1)", &lang),
            Rule::from_strings("(* x0 1)", "x0", &lang),
        ];
        let mut egraph = EGraph::from_expression(lang.parse_no_vars("(* 3 2)").unwrap());

        let reason = saturate(
            &mut egraph,
            &rules,
            BottomUpMatcher,
            &SaturationConfig { max_applications: Some(1), ..Default::default() },
        );
        assert_eq!(reason, SaturationStopReason::MaxApplications);
    }

    #[test]
    fn stops_on_timeout() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* x0 2)", "(<< x0 1)", &lang),
            Rule::from_strings("(* x0 1)", "x0", &lang),
        ];
        let mut egraph = EGraph::from_expression(lang.parse_no_vars("(* 3 2)").unwrap());

        let reason = saturate(
            &mut egraph,
            &rules,
            BottomUpMatcher,
            &SaturationConfig { time_limit: Some(Duration::from_millis(0)), ..Default::default() },
        );
        assert_eq!(reason, SaturationStopReason::Timeout);
    }

    #[test]
    fn stops_on_max_nodes() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* x0 2)", "(<< x0 1)", &lang),
            Rule::from_strings("(* x0 1)", "x0", &lang),
        ];
        let mut egraph = EGraph::from_expression(lang.parse_no_vars("(* 3 2)").unwrap());

        let reason = saturate(
            &mut egraph,
            &rules,
            BottomUpMatcher,
            &SaturationConfig { max_nodes: Some(4), ..Default::default() },
        );
        assert_eq!(reason, SaturationStopReason::MaxNodes);
    }

    #[test]
    fn stops_on_max_classes() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* x0 2)", "(<< x0 1)", &lang),
            Rule::from_strings("(* x0 1)", "x0", &lang),
        ];
        let mut egraph = EGraph::from_expression(lang.parse_no_vars("(* 3 2)").unwrap());

        let reason = saturate(
            &mut egraph,
            &rules,
            BottomUpMatcher,
            &SaturationConfig { max_classes: Some(3), ..Default::default() },
        );
        assert_eq!(reason, SaturationStopReason::MaxClasses);
    }
}
