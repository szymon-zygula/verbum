#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::language::expression::LangMultiExpression;
use crate::language::expression::VarFreeExpression;
use rewriting::{
    egraph::{
        class::simple_math_local_cost::SimpleMathLocalCost,
        extraction::{SimpleExtractor, children_cost_sum},
        matching::bottom_up::BottomUpMatcher,
        saturation::{SaturationConfig, SimpleSaturator, directed_saturator::DirectedSaturator},
    },
    system::TermRewritingSystem,
};

mod benchmark;
mod data_union_find;
mod index_selector;
mod language;
mod macros;
mod rewriting;
mod seen;
mod union_find;
mod utils;

fn initialize_system() -> TermRewritingSystem {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("rewriting-systems");
    path.push("simple_math.json");

    utils::json::load_json(path).unwrap()
}

fn main() {
    let trs = initialize_system();

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("expressions");
    path.push("simple-math.json");

    let multi_expressions: LangMultiExpression = utils::json::load_json(path).unwrap();
    let lang = multi_expressions.language();
    let expressions_with_vars = multi_expressions.expressions();

    let expressions: Vec<VarFreeExpression> = expressions_with_vars
        .iter()
        .map(|expr| {
            expr.without_variables().expect(
                "All expressions in simple-math.json should be variable-free for benchmarking",
            )
        })
        .collect();

    let config = benchmark::BenchmarkConfig {
        saturation_config: SaturationConfig {
            max_nodes: Some(1000),
            max_classes: Some(1000),
            max_applications: Some(1000),
            ..Default::default()
        },
    };

    let extractor = SimpleExtractor::<usize, _, _, ()>::new(
        |_| 1,
        |symbol, costs| {
            Some(
                match lang.get_symbol(symbol.id) {
                    "+" => 1usize,
                    "-" => 1usize,
                    "/" => 8usize,
                    "*" => 4usize,
                    "<<" => 2usize,
                    "sin" => 2usize,
                    _ => return None,
                } + children_cost_sum(symbol, costs)?,
            )
        },
    );

    let simple_saturator = SimpleSaturator::new(BottomUpMatcher);
    let simple_outcomes =
        benchmark::benchmark(&trs, &expressions, &config, &extractor, &simple_saturator);

    let extractor_2 = SimpleExtractor::<usize, _, _, _>::new(
        |_| 1,
        |symbol, costs| {
            Some(
                match lang.get_symbol(symbol.id) {
                    "+" => 1usize,
                    "-" => 1usize,
                    "/" => 8usize,
                    "*" => 4usize,
                    "<<" => 2usize,
                    "sin" => 2usize,
                    _ => return None,
                } + children_cost_sum(symbol, costs)?,
            )
        },
    );

    let directed_saturator = DirectedSaturator::new(BottomUpMatcher);
    let directed_outcomes = benchmark::benchmark::<
        SimpleMathLocalCost,
        SimpleExtractor<usize, _, _, SimpleMathLocalCost>,
    >(
        &trs,
        &expressions,
        &config,
        &extractor_2,
        &directed_saturator,
    );

    let map = BTreeMap::from([
        (String::from("Simple"), simple_outcomes),
        (String::from("Directed"), directed_outcomes),
    ]);

    use benchmark::{
        OutcomeFormatter, csv_output::CsvFormatter, pretty_printing::PrettyTableFormatter,
    };

    let pretty_formatter = PrettyTableFormatter;
    let pretty_output = pretty_formatter.format_saturator_outcomes(map.clone());
    println!("{pretty_output}");

    let csv_formatter = CsvFormatter;
    let csv_output = csv_formatter.format_saturator_outcomes(map);
    println!("\nCSV Output:\n{csv_output}");
}
