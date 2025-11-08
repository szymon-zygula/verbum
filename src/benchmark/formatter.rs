use std::collections::BTreeMap;
use std::time::Duration;
use tabled::{
    Table, Tabled,
    settings::Style,
};

/// A generic trait for formatting benchmark outcomes.
/// This trait is implemented for both Outcome and ReachabilityOutcome types.
pub trait Formattable: Tabled {
    /// Convert the outcome to a row of strings for CSV output
    fn to_csv_row(&self) -> Vec<String>;
    
    /// Get CSV headers for this type
    fn csv_headers() -> Vec<&'static str>;
}

/// Pretty table formatter using the tabled library
pub struct PrettyFormatter;

impl PrettyFormatter {
    /// Format a collection of formattable items as a pretty table
    pub fn format<T: Formattable>(items: &[T]) -> String {
        if items.is_empty() {
            return String::new();
        }

        let mut table = Table::new(items);
        table.with(Style::rounded());
        
        table.to_string()
    }

    /// Format outcomes grouped by saturator name
    pub fn format_grouped<T: Formattable>(
        outcomes_map: BTreeMap<String, Vec<T>>,
    ) -> String {
        let mut buffer = String::new();

        for (name, outcomes) in outcomes_map {
            buffer.push_str(&format!("\n--- Results for Saturator: {} ---\n\n", name));
            buffer.push_str(&Self::format(&outcomes));
            buffer.push('\n');
        }

        buffer
    }
}

/// CSV formatter
pub struct CsvFormatter;

impl CsvFormatter {
    /// Format a collection of formattable items as CSV
    pub fn format<T: Formattable>(items: &[T]) -> String {
        let mut csv = String::new();
        
        // Write header
        let headers = T::csv_headers();
        csv.push_str(&headers.join(","));
        csv.push('\n');

        // Write data rows
        for item in items {
            let row = item.to_csv_row();
            csv.push_str(&row.join(","));
            csv.push('\n');
        }

        csv
    }

    /// Format outcomes grouped by saturator name as CSV
    pub fn format_grouped<T: Formattable>(
        outcomes_map: BTreeMap<String, Vec<T>>,
    ) -> String {
        let mut csv = String::new();
        
        // Write header with saturator name column
        let mut headers = vec!["Saturator"];
        headers.extend(T::csv_headers());
        csv.push_str(&headers.join(","));
        csv.push('\n');

        // Write data rows
        for (saturator_name, outcomes) in outcomes_map {
            for outcome in outcomes {
                let mut row = vec![saturator_name.clone()];
                row.extend(outcome.to_csv_row());
                csv.push_str(&row.join(","));
                csv.push('\n');
            }
        }

        csv
    }
}

/// Format a Duration for display
pub(crate) fn format_duration(duration: &Duration) -> String {
    format!("{:?}", duration)
}

/// Format a Duration for CSV (as nanoseconds)
pub(crate) fn format_duration_csv(duration: &Duration) -> String {
    duration.as_nanos().to_string()
}
