use super::{OutcomeFormatter, Outcome, ReachabilityOutcome};
use super::formatter::PrettyFormatter;
use std::collections::BTreeMap;
use tabled::{Table, settings::Style};

pub struct PrettyTableFormatter;

/// A trait for formatting collections of `ReachabilityOutcome` values.
/// This trait exists for backwards compatibility but delegates to the unified formatter.
pub trait ReachabilityOutcomeFormatter {
    fn format_reachability_outcomes(&self, outcomes: &[ReachabilityOutcome]) -> String;
}

impl OutcomeFormatter for PrettyTableFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String {
        if outcomes.is_empty() {
            return String::new();
        }

        let mut table = Table::new(outcomes);
        table.with(Style::rounded());
        table.to_string()
    }

    fn format_saturator_outcomes(&self, outcomes_map: BTreeMap<String, Vec<Outcome>>) -> String {
        PrettyFormatter::format_grouped(outcomes_map)
    }
}

impl ReachabilityOutcomeFormatter for PrettyTableFormatter {
    fn format_reachability_outcomes(&self, outcomes: &[ReachabilityOutcome]) -> String {
        if outcomes.is_empty() {
            return String::new();
        }

        let mut table = Table::new(outcomes);
        table.with(Style::rounded());
        table.to_string()
    }
}
