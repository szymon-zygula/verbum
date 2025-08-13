use std::time::Duration;
pub mod random_generation;

use crate::rewriting::egraph::saturation::SaturationStopReason;

#[derive(Clone, Debug)]
struct Outcome<C> {
    pub nodes: usize,
    pub classes: usize,
    pub time: Duration,
    pub min_cost: C,
    pub stop_reason: SaturationStopReason,
}
