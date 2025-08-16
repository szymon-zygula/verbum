use super::OutcomeFormatter;
use crate::benchmark::Outcome;
use colored::*;
use std::{fmt::Write, time::Duration};
use std::collections::BTreeMap;
use crate::language::expression::VarFreeExpression;
use crate::rewriting::egraph::saturation::SaturationStopReason;

pub struct PrettyTableFormatter;

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

fn calculate_column_widths(outcomes: &[Outcome], headers: &[&str]) -> Vec<usize> {
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
    col_widths
}

fn print_table_header(buffer: &mut String, headers: &[&str], col_widths: &[usize]) {
    for (i, header) in headers.iter().enumerate() {
        write!(
            buffer,
            "{:<width$} ",
            format_cell(header.to_string(), col_widths[i]).bold(),
            width = col_widths[i]
        )
        .unwrap();
        if i < headers.len() - 1 {
            write!(buffer, "| ").unwrap();
        }
    }
    writeln!(buffer).unwrap();
}

fn print_separator_line(buffer: &mut String, col_widths: &[usize], num_headers: usize) {
    for (i, width) in col_widths.iter().enumerate() {
        write!(buffer, "{:-<width$}-", "", width = width).unwrap();
        if i < num_headers - 1 {
            write!(buffer, "-").unwrap();
        }
    }
    writeln!(buffer).unwrap();
}

fn print_table_row(buffer: &mut String, cells: Vec<String>, col_widths: &[usize], num_headers: usize, is_bold: bool) {
    let mut lines: Vec<Vec<String>> = vec![];
    let mut max_lines = 0;

    for cell in cells {
        let cell_lines: Vec<String> = cell.split('\n').map(|s| s.to_string()).collect();
        max_lines = max_lines.max(cell_lines.len());
        lines.push(cell_lines);
    }

    for i in 0..max_lines {
        for (j, col_lines) in lines.iter().enumerate() {
            let content = col_lines.get(i).unwrap_or(&"".to_string()).clone();
            if is_bold {
                let formatted_content = format!("{:<width$}", content.bold(), width = col_widths[j]);
                write!(buffer, "{} ", formatted_content).unwrap();
            } else {
                let formatted_content = format!("{:<width$}", content, width = col_widths[j]);
                write!(buffer, "{} ", formatted_content).unwrap();
            }
            if j < num_headers - 1 {
                write!(buffer, "| ").unwrap();
            }
        }
        writeln!(buffer).unwrap();
    }
}

fn print_table_content(
    buffer: &mut String,
    outcomes: &[Outcome],
    col_widths: &[usize],
    headers_len: usize,
) {
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

        let cells = vec![
            original_expr_formatted,
            extracted_expr_formatted,
            time_formatted,
            stop_reason_formatted,
            nodes_formatted,
            classes_formatted,
            min_cost_formatted,
        ];
        print_table_row(buffer, cells, col_widths, headers_len, false);
    }
}

impl OutcomeFormatter for PrettyTableFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String {
        let mut buffer = String::new();

        // Define column headers
        let headers = [
            "Original Expression",
            "Extracted Expression",
            "Time",
            "Stop Reason",
            "Nodes",
            "Classes",
            "Min Cost",
        ];

        // Determine column widths, capping at MAX_COL_WIDTH
        let col_widths = calculate_column_widths(outcomes, &headers);

        // Print header
        print_table_header(&mut buffer, &headers, &col_widths);

        // Print separator
        print_separator_line(&mut buffer, &col_widths, headers.len());

        // Print rows
        let mut total_time_sum = Duration::default();
        let mut num_nodes_sum: u64 = 0;
        let mut num_classes_sum: u64 = 0;
        let mut min_cost_sum: u64 = 0;

        for outcome in outcomes {
            total_time_sum += outcome.time;
            num_nodes_sum += outcome.nodes as u64;
            num_classes_sum += outcome.classes as u64;
            min_cost_sum += outcome.min_cost as u64;
        }

        print_table_content(&mut buffer, outcomes, &col_widths, headers.len());

        if !outcomes.is_empty() {
            let num_outcomes = outcomes.len() as u64;
            let avg_total_time = total_time_sum / num_outcomes as u32;
            let avg_num_nodes = num_nodes_sum / num_outcomes;
            let avg_classes = num_classes_sum / num_outcomes;
            let avg_min_cost = min_cost_sum / num_outcomes;

            // Print separator for averages
            writeln!(&mut buffer).unwrap();
            print_separator_line(&mut buffer, &col_widths, headers.len());

            // Print average row
            let original_expr_formatted = format_cell("AVERAGE".to_string(), col_widths[0]);
            let extracted_expr_formatted = format_cell("".to_string(), col_widths[1]);
            let time_formatted = format_cell(format!("{:?}", avg_total_time), col_widths[2]);
            let stop_reason_formatted = format_cell("".to_string(), col_widths[3]);
            let nodes_formatted = format_cell(format!("{}", avg_num_nodes), col_widths[4]);
            let classes_formatted = format_cell(format!("{}", avg_classes), col_widths[5]);
            let min_cost_formatted = format_cell(format!("{}", avg_min_cost), col_widths[6]);

            let cells = vec![
                original_expr_formatted,
                extracted_expr_formatted,
                time_formatted,
                stop_reason_formatted,
                nodes_formatted,
                classes_formatted,
                min_cost_formatted,
            ];
            print_table_row(&mut buffer, cells, &col_widths, headers.len(), true);
        }
        buffer
    }

    fn format_saturator_outcomes(&self, outcomes_map: BTreeMap<String, Vec<Outcome>>) -> String {
        let mut buffer = String::new();
        let mut all_saturator_avg_outcomes: BTreeMap<String, Outcome> = BTreeMap::new();

        for (saturator_name, outcomes) in outcomes_map {
            // Print saturator indicator
            writeln!(&mut buffer, "\n--- Results for Saturator: {} ---\n", saturator_name).unwrap();

            // Print individual table for this saturator
            buffer.push_str(&self.format_outcomes(&outcomes));

            // Calculate average outcome for this saturator
            if !outcomes.is_empty() {
                let num_outcomes = outcomes.len() as u64;
                let mut total_time_sum = Duration::default();
                let mut num_nodes_sum: u64 = 0;
                let mut num_classes_sum: u64 = 0;
                let mut min_cost_sum: u64 = 0;

                for outcome in outcomes {
                    total_time_sum += outcome.time;
                    num_nodes_sum += outcome.nodes as u64;
                    num_classes_sum += outcome.classes as u64;
                    min_cost_sum += outcome.min_cost as u64;
                }

                let avg_total_time = total_time_sum / num_outcomes as u32;
                let avg_num_nodes = num_nodes_sum / num_outcomes;
                let avg_classes = num_classes_sum / num_outcomes;
                let avg_min_cost = min_cost_sum / num_outcomes;

                all_saturator_avg_outcomes.insert(saturator_name, Outcome {
                    original_expression: VarFreeExpression::Literal(crate::language::expression::Literal::Int(0)), // Placeholder
                    extracted_expression: VarFreeExpression::Literal(crate::language::expression::Literal::Int(0)), // Placeholder
                    time: avg_total_time,
                    stop_reason: crate::rewriting::egraph::saturation::SaturationStopReason::Saturated, // Placeholder
                    nodes: avg_num_nodes as usize,
                    classes: avg_classes as usize,
                    min_cost: avg_min_cost as usize,
                });
            }
        }

        // Print overall averages table
        if !all_saturator_avg_outcomes.is_empty() {
            writeln!(&mut buffer, "\n--- Overall Averages Across Saturators ---\n").unwrap();

            // Define headers for the average table
            let avg_headers = [
                "Saturator",
                "Time",
                "Nodes",
                "Classes",
                "Min Cost",
            ];

            // Calculate column widths for the average table
            let mut avg_col_widths: Vec<usize> = avg_headers.iter().map(|h| h.len()).collect();
            for (saturator_name, avg_outcome) in &all_saturator_avg_outcomes {
                avg_col_widths[0] = avg_col_widths[0].max(saturator_name.len());
                avg_col_widths[1] = avg_col_widths[1].max(format!("{:?}", avg_outcome.time).len());
                avg_col_widths[2] = avg_col_widths[2].max(format!("{}", avg_outcome.nodes).len());
                avg_col_widths[3] = avg_col_widths[3].max(format!("{}", avg_outcome.classes).len());
                avg_col_widths[4] = avg_col_widths[4].max(format!("{}", avg_outcome.min_cost).len());
            }
            for width in avg_col_widths.iter_mut() {
                *width = (*width).min(MAX_COL_WIDTH);
            }

            // Print header for average table
            print_table_header(&mut buffer, &avg_headers, &avg_col_widths);
            print_separator_line(&mut buffer, &avg_col_widths, avg_headers.len());

            // Print rows for average table
            for (saturator_name, avg_outcome) in all_saturator_avg_outcomes {
                let saturator_name_formatted = format_cell(saturator_name, avg_col_widths[0]);
                let time_formatted = format_cell(format!("{:?}", avg_outcome.time), avg_col_widths[1]);
                let nodes_formatted = format_cell(format!("{}", avg_outcome.nodes), avg_col_widths[2]);
                let classes_formatted = format_cell(format!("{}", avg_outcome.classes), avg_col_widths[3]);
                let min_cost_formatted = format_cell(format!("{}", avg_outcome.min_cost), avg_col_widths[4]);

                let cells = vec![
                    saturator_name_formatted,
                    time_formatted,
                    nodes_formatted,
                    classes_formatted,
                    min_cost_formatted,
                ];
                print_table_row(&mut buffer, cells, &avg_col_widths, avg_headers.len(), false);
            }
        }

        buffer
    }
}