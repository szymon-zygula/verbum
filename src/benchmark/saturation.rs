use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use tabled::Tabled;

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
use super::formatter::{Formattable, format_duration, format_duration_csv};

fn format_stop_reason(reason: &SaturationStopReason) -> String {
    format!("{:?}", reason)
}

/// Number of runs (after warm-up) used for averaging.
pub const RUN_COUNT: usize = 10;

pub trait OutcomeFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String;
    fn format_saturator_outcomes(&self, outcomes_map: BTreeMap<String, Vec<Outcome>>) -> String;
}

#[derive(Clone, Debug, Tabled)]
pub struct Outcome {
    #[tabled(rename = "Original Expression")]
    pub original_expression: VarFreeExpression,
    #[tabled(rename = "Extracted Expression")]
    pub extracted_expression: VarFreeExpression,
    #[tabled(rename = "Time", display_with = "format_duration")]
    pub time: Duration,
    #[tabled(rename = "Stop Reason", display_with = "format_stop_reason")]
    pub stop_reason: SaturationStopReason,
    #[tabled(rename = "Nodes")]
    pub nodes: usize,
    #[tabled(rename = "Classes")]
    pub classes: usize,
    #[tabled(rename = "Min Cost")]
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
    fn to_csv_row(&self) -> Vec<String> {
        vec![
            self.original_expression.to_string(),
            self.extracted_expression.to_string(),
            format_duration_csv(&self.time),
            format!("{:?}", self.stop_reason),
            self.nodes.to_string(),
            self.classes.to_string(),
            self.min_cost.to_string(),
        ]
    }

    fn csv_headers() -> Vec<&'static str> {
        vec![
            "Original Expression",
            "Extracted Expression",
            "Time (ns)",
            "Stop Reason",
            "Nodes",
            "Classes",
            "Min Cost",
        ]
    }
}
