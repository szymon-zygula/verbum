use crate::benchmark::Outcome;

// XXX: Should be moved to `mod.rs`
pub trait OutcomeFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String;
}

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
}
