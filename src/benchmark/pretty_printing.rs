use super::OutcomeFormatter;
use crate::benchmark::Outcome;
use colored::*;
use std::fmt::Write;

pub struct PrettyTableFormatter;

impl OutcomeFormatter for PrettyTableFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String {
        let mut buffer = String::new();

        // Define column headers
        let headers = ["Original Expression",
            "Extracted Expression",
            "Time",
            "Stop Reason",
            "Nodes",
            "Classes",
            "Min Cost"];

        // Determine column widths, capping at MAX_COL_WIDTH
        let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

        for outcome in outcomes {
            col_widths[0] = col_widths[0].max(format!("{}", outcome.original_expression).len());
            col_widths[1] = col_widths[1].max(format!("{}", outcome.extracted_expression).len());
            col_widths[2] = col_widths[2].max(format!("{:?}", outcome.time).len());
            col_widths[3] = col_widths[3].max(format!("{:?}", outcome.stop_reason).len());
            col_widths[4] = col_widths[4].max(format!("{}", outcome.nodes).len());
            col_widths[5] = col_widths[5].max(format!("{}", outcome.classes).len());
            col_widths[6] = col_widths[6].max(format!("{}", outcome.min_cost).len());
        }

        for width in col_widths.iter_mut() {
            *width = (*width).min(MAX_COL_WIDTH);
        }

        // Print header
        for (i, header) in headers.iter().enumerate() {
            write!(
                &mut buffer, 
                "{:<width$} ", 
                format_cell(header.to_string(), col_widths[i]).bold(), 
                width = col_widths[i]
            ).unwrap();
            if i < headers.len() - 1 {
                write!(&mut buffer, "| ").unwrap();
            }
        }
        writeln!(&mut buffer).unwrap();

        // Print separator
        for (i, width) in col_widths.iter().enumerate() {
            write!(&mut buffer, "{:-<width$}-", "", width = width).unwrap();
            if i < headers.len() - 1 {
                write!(&mut buffer, "-").unwrap();
            }
        }
        writeln!(&mut buffer).unwrap();

        // Print rows
        for outcome in outcomes {
            let original_expr_formatted =
                format_cell(format!("{}", outcome.original_expression), col_widths[0]);
            let extracted_expr_formatted =
                format_cell(format!("{}", outcome.extracted_expression), col_widths[1]);
            let time_formatted = format_cell(format!("{:?}", outcome.time), col_widths[2]);
            let stop_reason_formatted =
                format_cell(format!("{:?}", outcome.stop_reason), col_widths[3]);
            let nodes_formatted = format_cell(format!("{}", outcome.nodes), col_widths[4]);
            let classes_formatted = format_cell(format!("{}", outcome.classes), col_widths[5]);
            let min_cost_formatted = format_cell(format!("{}", outcome.min_cost), col_widths[6]);

            let mut lines: Vec<Vec<String>> = vec![];
            let mut max_lines = 0;

            let cells = vec![
                original_expr_formatted,
                extracted_expr_formatted,
                time_formatted,
                stop_reason_formatted,
                nodes_formatted,
                classes_formatted,
                min_cost_formatted,
            ];

            for cell in cells {
                let cell_lines: Vec<String> = cell.split('\n').map(|s| s.to_string()).collect();
                max_lines = max_lines.max(cell_lines.len());
                lines.push(cell_lines);
            }

            for i in 0..max_lines {
                for (j, col_lines) in lines.iter().enumerate() {
                    let content = col_lines.get(i).unwrap_or(&"".to_string()).clone();
                    write!(&mut buffer, "{:<width$} ", content, width = col_widths[j]).unwrap();
                    if j < headers.len() - 1 {
                        write!(&mut buffer, "| ").unwrap();
                    }
                }
                writeln!(&mut buffer).unwrap();
            }
        }
        buffer
    }
}

const MAX_COL_WIDTH: usize = 30;

fn format_cell(content: String, width: usize) -> String {
    if content.len() > width {
        let mut formatted = String::new();
        let mut current_len = 0;
        for c in content.chars() {
            if current_len >= width {
                formatted.push('\n');
                current_len = 0;
            }
            formatted.push(c);
            current_len += 1;
        }
        formatted
    } else {
        format!("{content:<width$}")
    }
}