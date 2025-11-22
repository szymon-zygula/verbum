//! Generic formatting module for benchmark outcomes.
//!
//! This module provides a unified approach to formatting benchmark results
//! using the `tabled` library for pretty-printed tables and the `csv` crate for CSV output.
//! 
//! The `Formattable` trait enables any benchmark outcome type to be formatted
//! consistently without code duplication.

use std::collections::BTreeMap;
use std::time::Duration;
use tabled::{
    Table, Tabled,
    settings::Style,
};

/// A generic trait for formatting benchmark outcomes.
/// This trait is implemented for both Outcome and ReachabilityOutcome types.
pub trait Formattable: Tabled + serde::Serialize {
    /// Calculate averages for numeric fields and return as a map
    fn calculate_averages(items: &[Self]) -> Option<String>
    where
        Self: Sized;
}

/// Pretty table formatter using the tabled library
pub struct PrettyFormatter;

impl PrettyFormatter {
    /// Format a collection of formattable items as a pretty table with averages
    pub fn format<T: Formattable>(items: &[T]) -> String {
        if items.is_empty() {
            return String::new();
        }

        let mut buffer = String::new();
        
        // Create and format the main table
        let mut table = Table::new(items);
        table.with(Style::rounded());
        buffer.push_str(&table.to_string());
        
        // Add averages if available
        if let Some(avg_str) = T::calculate_averages(items) {
            buffer.push('\n');
            buffer.push_str(&avg_str);
        }
        
        buffer
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

/// CSV formatter using the csv crate
pub struct CsvFormatter;

impl CsvFormatter {
    /// Format a collection of formattable items as CSV
    pub fn format<T: Formattable>(items: &[T]) -> Result<String, csv::Error> {
        let mut wtr = csv::Writer::from_writer(vec![]);
        
        for item in items {
            wtr.serialize(item)?;
        }
        
        wtr.flush()?;
        let bytes = wtr.into_inner().map_err(|e| {
            csv::Error::from(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;
        
        String::from_utf8(bytes)
            .map_err(|e| csv::Error::from(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    }

    /// Format outcomes grouped by saturator name as CSV
    /// Note: This creates a simple CSV with all data, without the saturator column
    /// since adding a prefix column would require restructuring the data types
    pub fn format_grouped<T: Formattable>(
        outcomes_map: BTreeMap<String, Vec<T>>,
    ) -> Result<String, csv::Error> {
        let mut all_outcomes = Vec::new();
        for (_name, mut outcomes) in outcomes_map {
            all_outcomes.append(&mut outcomes);
        }
        Self::format(&all_outcomes)
    }
}

/// Format a Duration for display
pub(crate) fn format_duration(duration: &Duration) -> String {
    format!("{:?}", duration)
}
