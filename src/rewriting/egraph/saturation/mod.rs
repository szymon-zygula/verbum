use std::time::{Duration, Instant};

use crate::rewriting::rule::Rule;

use super::{Analysis, EGraph};

pub mod simple_saturator;
pub use simple_saturator::SimpleSaturator;
pub mod directed_saturator;
pub mod scheduler;
pub mod scheduled_saturator;

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

pub fn check_limits<A: Analysis>(
    egraph: &EGraph<A>,
    applications: usize,
    start: Instant,
    cfg: &SaturationConfig,
) -> Option<SaturationStopReason> {
    if let Some(limit) = cfg.time_limit
        && start.elapsed() >= limit
    {
        return Some(SaturationStopReason::Timeout);
    }

    if let Some(limit) = cfg.max_applications
        && applications >= limit
    {
        return Some(SaturationStopReason::MaxApplications);
    }

    if let Some(limit) = cfg.max_nodes
        && egraph.actual_node_count() >= limit
    {
        return Some(SaturationStopReason::MaxNodes);
    }

    if let Some(limit) = cfg.max_classes
        && egraph.class_count() >= limit
    {
        return Some(SaturationStopReason::MaxClasses);
    }

    None
}

pub trait Saturator<A: Analysis> {
    fn saturate(
        &self,
        egraph: &mut EGraph<A>,
        rules: &[Rule],
        config: &SaturationConfig,
    ) -> SaturationStopReason;
}
