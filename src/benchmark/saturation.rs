use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tabled::Tabled;
use serde::Serialize;

use crate::{
    language::expression::VarFreeExpression,
    rewriting::{
        egraph::{
            Analysis, DynEGraph, EGraph,
            extraction::Extractor,
            saturation::{SaturationConfig, SaturationStopReason, Saturator},
        },
        system::TermRewritingSystem,
    },
};
use super::formatter::{Formattable, format_duration};

fn format_stop_reason(reason: &SaturationStopReason) -> String {
    format!("{:?}", reason)
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u128(duration.as_nanos())
}

fn serialize_stop_reason<S>(reason: &SaturationStopReason, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&format!("{:?}", reason))
}

fn serialize_expr<S>(expr: &VarFreeExpression, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&expr.to_string())
}

/// Number of runs (after warm-up) used for averaging.
pub const RUN_COUNT: usize = 10;

pub trait OutcomeFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String;
    fn format_saturator_outcomes(&self, outcomes_map: BTreeMap<String, Vec<Outcome>>) -> String;
}

#[derive(Clone, Debug, Tabled, Serialize)]
pub struct Outcome {
    #[tabled(rename = "Original Expression")]
    #[serde(rename = "Original Expression", serialize_with = "serialize_expr")]
    pub original_expression: VarFreeExpression,
    #[tabled(rename = "Extracted Expression")]
    #[serde(rename = "Extracted Expression", serialize_with = "serialize_expr")]
    pub extracted_expression: VarFreeExpression,
    #[tabled(rename = "Time", display_with = "format_duration")]
    #[serde(rename = "Time (ns)", serialize_with = "serialize_duration")]
    pub time: Duration,
    #[tabled(rename = "Stop Reason", display_with = "format_stop_reason")]
    #[serde(rename = "Stop Reason", serialize_with = "serialize_stop_reason")]
    pub stop_reason: SaturationStopReason,
    #[tabled(rename = "Nodes")]
    #[serde(rename = "Nodes")]
    pub nodes: usize,
    #[tabled(rename = "Classes")]
    #[serde(rename = "Classes")]
    pub classes: usize,
    #[tabled(rename = "Min Cost")]
    #[serde(rename = "Min Cost")]
    pub min_cost: usize,
}

impl PartialEq for Outcome {
    fn eq(&self, other: &Self) -> bool {
        self.original_expression == other.original_expression
            && self.extracted_expression == other.extracted_expression
            && self.stop_reason == other.stop_reason
            && self.nodes == other.nodes
            && self.classes == other.classes
            && self.min_cost == other.min_cost
    }
}

impl Eq for Outcome {}

#[derive(Clone, Debug, Default)]
pub struct BenchmarkConfig {
    pub saturation_config: SaturationConfig,
}

fn run_single_benchmark<A, E>(
    trs: &TermRewritingSystem,
    expression: VarFreeExpression,
    config: &BenchmarkConfig,
    extractor: &E,
    saturator: &dyn Saturator<A>,
) -> Outcome
where
    A: Analysis,
    E: Extractor<Cost = usize>,
{
    let (mut egraph, class_id) = EGraph::<A>::from_expression_with_id(expression.clone());

    let start_time = Instant::now();
    let stop_reason = saturator.saturate(&mut egraph, trs.rules(), &config.saturation_config);
    let time = start_time.elapsed();

    let extraction_result = extractor.extract(&egraph, class_id);
    let extracted_expression = match &extraction_result {
        Some(res) => res.winner().clone(),
        None => expression.clone(),
    };
    let min_cost = match extraction_result {
        Some(res) => *res.cost(),
        None => 0, // Default cost if no extraction or error
    };

    Outcome {
        original_expression: expression,
        extracted_expression,
        time,
        stop_reason,
        nodes: egraph.actual_node_count(),
        classes: egraph.class_count(),
        min_cost,
    }
}

pub fn benchmark<A, E>(
    trs: &TermRewritingSystem,
    expressions: &[VarFreeExpression],
    config: &BenchmarkConfig,
    extractor: &E,
    saturator: &dyn Saturator<A>,
) -> Vec<Outcome>
where
    A: Analysis,
    E: Extractor<Cost = usize>,
{
    let mut averaged_outcomes = Vec::with_capacity(expressions.len());

    for expression in expressions {
        let mut expression_outcomes =
            benchmark_multiple_times(trs, config, extractor, saturator, expression.clone());
        let mut averaged_outcome = expression_outcomes.remove(0);

        for (i, outcome) in expression_outcomes.into_iter().enumerate() {
            assert_eq!(
                averaged_outcome,
                outcome,
                "Outcome mismatch in run {} for expression {:?}. Expected {:?}, got {:?}",
                i + 1, // +1 because we removed the first run
                averaged_outcome.original_expression,
                averaged_outcome,
                outcome
            );
            averaged_outcome.time += outcome.time;
        }

        averaged_outcome.time /= RUN_COUNT as u32;
        averaged_outcomes.push(averaged_outcome);
    }

    averaged_outcomes
}

pub fn benchmark_saturators<A, E>(
    trs: &TermRewritingSystem,
    expressions: &[VarFreeExpression],
    config: &BenchmarkConfig,
    extractor: &E,
    saturators: BTreeMap<String, Box<dyn Saturator<A>>>,
) -> BTreeMap<String, Vec<Outcome>>
where
    A: Analysis,
    E: Extractor<Cost = usize>,
{
    saturators
        .into_iter()
        .map(|(name, saturator)| {
            (
                name,
                benchmark(trs, expressions, config, extractor, saturator.as_ref()),
            )
        })
        .collect()
}

/// Benchmarks saturation of `expression` `RUN_COUNT + 1` times,
/// rejecting the first outcome as cache warm-up
fn benchmark_multiple_times<A, E>(
    trs: &TermRewritingSystem,
    config: &BenchmarkConfig,
    extractor: &E,
    saturator: &dyn Saturator<A>,
    expression: VarFreeExpression,
) -> Vec<Outcome>
where
    A: Analysis,
    E: Extractor<Cost = usize>,
{
    let mut expression_outcomes: Vec<Outcome> = Vec::with_capacity(RUN_COUNT + 1);
    for _ in 0..(RUN_COUNT + 1) {
        expression_outcomes.push(run_single_benchmark(
            trs,
            expression.clone(),
            config,
            extractor,
            saturator,
        ));
    }

    expression_outcomes.remove(0);
    // Cache warm-up
    expression_outcomes
}

impl Formattable for Outcome {
    fn calculate_averages(items: &[Self]) -> Option<String> {
        if items.is_empty() {
            return None;
        }

        let num_outcomes = items.len() as u64;
        let mut total_time_sum = Duration::default();
        let mut num_nodes_sum: u64 = 0;
        let mut num_classes_sum: u64 = 0;
        let mut min_cost_sum: u64 = 0;

        for outcome in items {
            total_time_sum += outcome.time;
            num_nodes_sum += outcome.nodes as u64;
            num_classes_sum += outcome.classes as u64;
            min_cost_sum += outcome.min_cost as u64;
        }

        let avg_total_time = total_time_sum / num_outcomes as u32;
        let avg_num_nodes = num_nodes_sum / num_outcomes;
        let avg_classes = num_classes_sum / num_outcomes;
        let avg_min_cost = min_cost_sum / num_outcomes;

        // Create a formatted average row
        use tabled::{Table, settings::Style};
        
        #[derive(Tabled)]
        struct AverageRow {
            #[tabled(rename = "Original Expression")]
            label: String,
            #[tabled(rename = "Extracted Expression")]
            empty1: String,
            #[tabled(rename = "Time")]
            time: String,
            #[tabled(rename = "Stop Reason")]
            empty2: String,
            #[tabled(rename = "Nodes")]
            nodes: u64,
            #[tabled(rename = "Classes")]
            classes: u64,
            #[tabled(rename = "Min Cost")]
            min_cost: u64,
        }

        let avg_row = AverageRow {
            label: "AVERAGE".to_string(),
            empty1: String::new(),
            time: format!("{:?}", avg_total_time),
            empty2: String::new(),
            nodes: avg_num_nodes,
            classes: avg_classes,
            min_cost: avg_min_cost,
        };

        let mut table = Table::new(vec![avg_row]);
        table.with(Style::rounded());
        Some(table.to_string())
    }
}
