use super::OutcomeFormatter;
use crate::benchmark::Outcome;
use std::collections::BTreeMap;

pub struct CsvFormatter;

impl OutcomeFormatter for CsvFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String {
        let mut csv = String::new();
        // Write header
        csv.push_str("Original Expression,Extracted Expression,Time (ns),Stop Reason,Nodes,Classes,Min Cost\n");

        // Write data rows
        for outcome in outcomes {
            csv.push_str(&format!(
                "{},{},{},{:?},{},{},{}\n",
                outcome.original_expression,
                outcome.extracted_expression,
                outcome.time.as_nanos(),
                outcome.stop_reason,
                outcome.nodes,
                outcome.classes,
                outcome.min_cost
            ));
        }
        csv
    }

    fn format_saturator_outcomes(&self, outcomes_map: BTreeMap<String, Vec<Outcome>>) -> String {
        let mut csv = String::new();
        // Write header
        csv.push_str("Saturator Name,Original Expression,Extracted Expression,Time (ns),Stop Reason,Nodes,Classes,Min Cost\n");

        for (saturator_name, outcomes) in outcomes_map {
            for outcome in outcomes {
                csv.push_str(&format!(
                    "{},{},{},{},{:?},{},{},{}\n",
                    saturator_name,
                    outcome.original_expression,
                    outcome.extracted_expression,
                    outcome.time.as_nanos(),
                    outcome.stop_reason,
                    outcome.nodes,
                    outcome.classes,
                    outcome.min_cost
                ));
            }
        }
        csv
    }
}
