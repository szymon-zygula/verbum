#![allow(dead_code)]

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use crate::language::expression::VarFreeExpression;
use crate::language::Language;
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
    rule::Rule,
    system::TermRewritingSystem,
};
use serde::Deserialize;

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

// Helper structs for loading JSON files
#[derive(Deserialize)]
struct RulesFile {
    rules: Vec<RuleSpec>,
}

#[derive(Deserialize)]
struct RuleSpec {
    from: String,
    to: String,
}

#[derive(Deserialize)]
struct ExpressionsFile {
    expressions: Vec<String>,
}

#[derive(Deserialize)]
struct CostsFile {
    costs: HashMap<String, usize>,
}

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
    let mut base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_path.push("jsons");
    base_path.push("simple-math");

    // Load language
    let mut lang_path = base_path.clone();
    lang_path.push("language.json");
    let language: Language = utils::json::load_json(lang_path).unwrap();

    // Load rules
    let mut rules_path = base_path.clone();
    rules_path.push("trs.json");
    let rules_file: RulesFile = utils::json::load_json(rules_path).unwrap();

    // Parse rules using the language
    let rules: Vec<Rule> = rules_file
        .rules
        .into_iter()
        .map(|rule_spec| Rule::from_strings(&rule_spec.from, &rule_spec.to, &language))
        .collect();

    TermRewritingSystem::new(language, rules)
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
    let lang = trs.language();

    // Load expressions from new location
    let mut base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_path.push("jsons");
    base_path.push("simple-math");

    let mut expr_path = base_path.clone();
    expr_path.push("small.json");
    let expressions_file: ExpressionsFile = utils::json::load_json(expr_path).unwrap();

    // Parse expressions using the language
    let expressions: Vec<VarFreeExpression> = expressions_file
        .expressions
        .iter()
        .map(|expr_str| {
            lang.parse_no_vars(expr_str).expect(
                "All expressions in small.json should be variable-free for benchmarking",
            )
        })
        .collect();

    // Load costs from JSON
    let mut costs_path = base_path.clone();
    costs_path.push("costs.json");
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
        OutcomeFormatter, csv_output::CsvFormatter, pretty_printing::PrettyTableFormatter,
    };

    let pretty_formatter = PrettyTableFormatter;
    let pretty_output = pretty_formatter.format_saturator_outcomes(map.clone());
    println!("{pretty_output}");

    let csv_formatter = CsvFormatter;
    let csv_output = csv_formatter.format_saturator_outcomes(map);
    println!("\nCSV Output:\n{csv_output}");

    test_saturation_and_dot_output();

    save_graph_dot();
    save_data_graph_dot();
    save_edge_data_graph_dot();
}
