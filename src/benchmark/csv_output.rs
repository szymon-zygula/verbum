use super::{OutcomeFormatter, Outcome};
use super::formatter::CsvFormatter;
use std::collections::BTreeMap;

pub struct CsvOutputFormatter;

impl OutcomeFormatter for CsvOutputFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String {
        CsvFormatter::format(outcomes).unwrap_or_else(|e| format!("Error formatting CSV: {}", e))
    }

    fn format_saturator_outcomes(&self, outcomes_map: BTreeMap<String, Vec<Outcome>>) -> String {
        CsvFormatter::format_grouped(outcomes_map).unwrap_or_else(|e| format!("Error formatting CSV: {}", e))
    }
}
