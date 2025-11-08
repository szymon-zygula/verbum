#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::language::expression::LangMultiExpression;
use crate::language::expression::VarFreeExpression;
use rewriting::{
    egraph::{
        EGraph,
        class::simple_math_local_cost::SimpleMathLocalCost,
        extraction::{SimpleExtractor, children_cost_sum},
        matching::bottom_up::BottomUpMatcher,
        saturation::{
            SaturationConfig, Saturator, SimpleSaturator, directed_saturator::DirectedSaturator,
        },
    },
    system::TermRewritingSystem,
};

mod benchmark;
mod data_union_find;
mod did;
mod equation;
mod graph;
mod index_selector;
mod language;
mod macros;
mod rewriting;
mod seen;
mod union_find;
mod utils;

fn save_graph_dot() {
    use graph::Graph;
    use std::fs::File;
    use std::io::Write;

    let mut graph = Graph::new();
    let v0 = graph.add_vertex();
    let v1 = graph.add_vertex();
    let v2 = graph.add_vertex();
    graph.add_edge(v0, v1);
    graph.add_edge(v1, v2);
    graph.add_edge(v2, v0);

    let dot_output = graph.dot();
    let mut file = File::create("graph.dot").expect("Unable to create file");
    file.write_all(dot_output.as_bytes())
        .expect("Unable to write data");
    println!("Saved graph.dot");
}

fn save_data_graph_dot() {
    use graph::DataGraph;
    use std::fs::File;
    use std::io::Write;

    let mut graph = DataGraph::new();
    let v0 = graph.add_vertex("A");
    let v1 = graph.add_vertex("B");
    let v2 = graph.add_vertex("C");
    graph.add_edge(v0, v1);
    graph.add_edge(v1, v2);
    graph.add_edge(v2, v0);

    let dot_output = graph.dot();
    let mut file = File::create("data_graph.dot").expect("Unable to create file");
    file.write_all(dot_output.as_bytes())
        .expect("Unable to write data");
    println!("Saved data_graph.dot");
}

fn save_edge_data_graph_dot() {
    use graph::EdgeDataGraph;
    use std::fs::File;
    use std::io::Write;

    let mut graph = EdgeDataGraph::new();
    let v0 = graph.add_vertex("Node A");
    let v1 = graph.add_vertex("Node B");
    let v2 = graph.add_vertex("Node C");
    graph.add_edge(v0, v1, "Edge 1");
    graph.add_edge(v1, v2, "Edge 2");
    graph.add_edge(v2, v0, "Edge 3");

    let dot_output = graph.dot();
    let mut file = File::create("edge_data_graph.dot").expect("Unable to create file");
    file.write_all(dot_output.as_bytes())
        .expect("Unable to write data");
    println!("Saved edge_data_graph.dot");
}

fn initialize_system() -> TermRewritingSystem {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("rewriting-systems");
    path.push("simple_math.json");

    utils::json::load_json(path).unwrap()
}

fn test_saturation_and_dot_output() {
    let lang = language::Language::simple_math();
    let rules = macros::rules!(lang;
        "(* $0 2)" => "(<< $0 1)",
        "(* $0 1)" => "$0",
        "(/ (* $0 $1) $2)" => "(* $0 (/ $1 $2))",
        "(/ $0 $0)" => "1",
    );

    let expression = lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap();
    let mut egraph = EGraph::<SimpleMathLocalCost>::from_expression(expression);

    let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
    let config = SaturationConfig::default();
    saturator.saturate(&mut egraph, &rules, &config);

    egraph.save_dot(&lang, "output.dot").unwrap();
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

    let extractor = SimpleExtractor::<usize, _, _>::new(
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

    let simple_saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
    let simple_outcomes =
        benchmark::benchmark::<(), _>(&trs, &expressions, &config, &extractor, &simple_saturator);

    let extractor_2 = SimpleExtractor::<usize, _, _>::new(
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
        OutcomeFormatter,
        csv_output::CsvOutputFormatter,
        pretty_printing::{PrettyTableFormatter, ReachabilityOutcomeFormatter},
    };

    let pretty_formatter = PrettyTableFormatter;
    let pretty_output = pretty_formatter.format_saturator_outcomes(map.clone());
    println!("{pretty_output}");

    let csv_formatter = CsvOutputFormatter;
    let csv_output = csv_formatter.format_saturator_outcomes(map);
    println!("\nCSV Output:\n{csv_output}");

    // Demonstrate reachability benchmarking (current: round-robin scheduler injected locally)
    use benchmark::reachability_benchmark_pairs_with_scheduler;
    use rewriting::egraph::matching::top_down::TopDownMatcher;
    use rewriting::egraph::saturation::scheduler::RoundRobinScheduler;

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

    test_saturation_and_dot_output();

    save_graph_dot();
    save_data_graph_dot();
    save_edge_data_graph_dot();
}
