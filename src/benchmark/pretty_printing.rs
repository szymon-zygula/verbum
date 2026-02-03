use super::formatter::PrettyFormatter;
use super::{Outcome, OutcomeFormatter, ReachabilityOutcome};
use std::collections::BTreeMap;

pub struct PrettyTableFormatter;

impl OutcomeFormatter for PrettyTableFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String {
        PrettyFormatter::format(outcomes)
    }

    fn format_saturator_outcomes(&self, outcomes_map: BTreeMap<String, Vec<Outcome>>) -> String {
        PrettyFormatter::format_grouped(outcomes_map)
    }
}

impl PrettyTableFormatter {
    /// Format reachability outcomes as a pretty table
    pub fn format_reachability_outcomes(&self, outcomes: &[ReachabilityOutcome]) -> String {
        PrettyFormatter::format(outcomes)
    }
}
