//! Equality saturation algorithms and configuration.
//!
//! This module provides algorithms for equality saturation - the process of
//! repeatedly applying rewrite rules to an e-graph until a fixed point is reached
//! or resource limits are hit.

use std::time::{Duration, Instant};

use crate::rewriting::rule::Rule;

use super::{Analysis, DynEGraph, EGraph};

pub mod simple_saturator;
pub use simple_saturator::SimpleSaturator;
pub mod directed_saturator;
pub mod scheduled_saturator;
pub mod scheduler;

/// Configuration for equality saturation.
///
/// Defines resource limits that control when saturation should stop.
#[derive(Clone, Debug, Default)]
pub struct SaturationConfig {
    /// Maximum number of nodes in the e-graph
    pub max_nodes: Option<usize>,
    /// Maximum number of equivalence classes
    pub max_classes: Option<usize>,
    /// Maximum number of rule applications
    pub max_applications: Option<usize>,
    /// Maximum time to spend saturating
    pub time_limit: Option<Duration>,
}

/// Reason why saturation stopped.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SaturationStopReason {
    /// Reached a fixed point (no more rules can be applied)
    Saturated,
    /// Hit the maximum node count limit
    MaxNodes,
    /// Hit the maximum class count limit
    MaxClasses,
    /// Hit the maximum applications limit
    MaxApplications,
    /// Hit the time limit
    Timeout,
}

/// Checks if any resource limits have been exceeded.
///
/// # Arguments
///
/// * `egraph` - The e-graph being saturated
/// * `applications` - The number of rule applications so far
/// * `start` - The time when saturation started
/// * `cfg` - The saturation configuration
///
/// # Returns
///
/// Returns `Some(reason)` if a limit was hit, `None` otherwise
pub fn check_limits(
    egraph: &dyn DynEGraph,
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
