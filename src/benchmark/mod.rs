pub mod csv_output;
pub mod pretty_printing;
pub mod random_generation;

use std::time::{Duration, Instant};

use crate::{
    language::expression::VarFreeExpression,
    rewriting::{
        egraph::{
            Analysis, EGraph,
            extraction::{Extractor},
            matching::bottom_up::BottomUpMatcher,
            saturation::{SaturationConfig, SaturationStopReason, saturate},
        },
        system::TermRewritingSystem,
    },
};

pub trait OutcomeFormatter {
    fn format_outcomes(&self, outcomes: &[Outcome]) -> String;
}

#[derive(Clone, Debug)]
pub struct Outcome {
    pub original_expression: VarFreeExpression,
    pub extracted_expression: VarFreeExpression,
    pub time: Duration,
    pub stop_reason: SaturationStopReason,
    pub nodes: usize,
    pub classes: usize,
    pub min_cost: usize,
}

#[derive(Clone, Debug, Default)]
pub struct BenchmarkConfig {
    pub saturation_config: SaturationConfig,
}

pub fn benchmark<A, E>(
    trs: &TermRewritingSystem,
    expressions: Vec<VarFreeExpression>,
    config: &BenchmarkConfig,
    extractor: &E,
) -> Vec<Outcome>
where
    A: Analysis + Default,
    E: Extractor<A, Cost = usize>,
{
    let mut outcomes = Vec::new();

    for expression in expressions {
        let mut egraph = EGraph::<A>::from_expression(expression.clone());

        let start_time = Instant::now();
        let stop_reason = saturate(
            &mut egraph,
            trs.rules(),
            BottomUpMatcher,
            &config.saturation_config,
        );
        let time = start_time.elapsed();

        let extraction_result = extractor.extract(&egraph, egraph.containing_class(0));
        let extracted_expression = match &extraction_result {
            Some(res) => res.winner().clone(),
            None => expression.clone(),
        };
        let min_cost = match extraction_result {
            Some(res) => *res.cost(),
            None => 0, // Default cost if no extraction or error
        };

        outcomes.push(Outcome {
            original_expression: expression,
            extracted_expression,
            time,
            stop_reason,
            nodes: egraph.actual_node_count(),
            classes: egraph.class_count(),
            min_cost,
        });
    }

    outcomes
}
