#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use serde::Deserialize;
use verbum::benchmark;
use verbum::benchmark::reachability_benchmark_pairs_with_scheduler;
use verbum::language::expression::load_expressions_from_file;
use verbum::rewriting::egraph::matching::top_down::TopDownMatcher;
use verbum::rewriting::egraph::saturation::scheduler::RoundRobinScheduler;
use verbum::{
    rewriting::{
        egraph::{
            class::simple_math_local_cost::SimpleMathLocalCost,
            extraction::{SimpleExtractor, children_cost_sum},
            matching::bottom_up::BottomUpMatcher,
            saturation::{
                SaturationConfig, SimpleSaturator, directed_saturator::DirectedSaturator,
            },
        },
        system::TermRewritingSystem,
    },
    utils,
};

// Helper struct for loading costs from JSON
#[derive(Deserialize)]
struct CostsFile {
    costs: HashMap<String, usize>,
}

fn initialize_system() -> TermRewritingSystem {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("jsons");
    path.push("simple-math");

    TermRewritingSystem::from_directory(path).unwrap()
}

fn main() {
    let trs = initialize_system();
    let lang = trs.language();

    // Load expressions and costs
    let mut base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_path.push("jsons");
    base_path.push("simple-math");

    let expr_path = base_path.join("small.json");
    let expressions = load_expressions_from_file(expr_path, lang).unwrap();

    let costs_path = base_path.join("costs.json");
    let costs_file: CostsFile = utils::json::load_json(costs_path).unwrap();
    let costs = costs_file.costs;

    let config = benchmark::BenchmarkConfig {
        saturation_config: SaturationConfig {
            max_nodes: Some(1000),
            max_classes: Some(1000),
            max_applications: Some(1000),
            ..Default::default()
        },
    };

    let extractor = SimpleExtractor::<usize, _, _>::new(
        |_| 1,
        |symbol, children_costs| {
            let symbol_name = lang.get_symbol(symbol.id);
            let symbol_cost = costs.get(symbol_name).copied().unwrap_or(0);
            Some(symbol_cost + children_cost_sum(symbol, children_costs)?)
        },
    );

    let simple_saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
    let simple_outcomes =
        benchmark::benchmark::<(), _>(&trs, &expressions, &config, &extractor, &simple_saturator);

    let extractor_2 = SimpleExtractor::<usize, _, _>::new(
        |_| 1,
        |symbol, children_costs| {
            let symbol_name = lang.get_symbol(symbol.id);
            let symbol_cost = costs.get(symbol_name).copied().unwrap_or(0);
            Some(symbol_cost + children_cost_sum(symbol, children_costs)?)
        },
    );

    let directed_saturator = DirectedSaturator::new(Box::new(BottomUpMatcher));
    let directed_outcomes = benchmark::benchmark::<SimpleMathLocalCost, SimpleExtractor<usize, _, _>>(
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
        OutcomeFormatter, csv_output::CsvOutputFormatter, pretty_printing::PrettyTableFormatter,
    };

    let pretty_formatter = PrettyTableFormatter;
    let pretty_output = pretty_formatter.format_saturator_outcomes(map.clone());
    println!("{pretty_output}");

    let csv_formatter = CsvOutputFormatter;
    let csv_output = csv_formatter.format_saturator_outcomes(map);
    println!("\nCSV Output:\n{csv_output}");

    // Demonstrate reachability benchmarking (current: round-robin scheduler injected locally)
    let reach_pairs = vec![
        (
            lang.parse_no_vars("(+ 1 0)").unwrap(),
            lang.parse_no_vars("1").unwrap(),
        ), // should unify via rule
        (
            lang.parse_no_vars("(* 2 3)").unwrap(),
            lang.parse_no_vars("4").unwrap(),
        ), // likely no unification
    ];

    let reach_cfg = SaturationConfig {
        max_applications: Some(50),
        ..Default::default()
    };
    let reach_outcomes = reachability_benchmark_pairs_with_scheduler::<(), _>(
        trs.rules(),
        &reach_pairs,
        &reach_cfg,
        &TopDownMatcher,
        3,
        |rs| Box::new(RoundRobinScheduler::new(rs.to_vec())),
    );
    println!("\nReachability Outcomes:");
    let reach_table = pretty_formatter.format_reachability_outcomes(&reach_outcomes);
    println!("{reach_table}");
}
