#![allow(dead_code)]

use std::collections::BTreeMap;

use language::Language;
use rewriting::{
    egraph::{
        class::simple_math_local_cost::SimpleMathLocalCost,
        extraction::{SimpleExtractor, children_cost_sum},
        matching::bottom_up::BottomUpMatcher,
        saturation::{
            SaturationConfig, SimpleSaturator, directed_saturator::DirectedSaturator,
        },
    },
    system::TermRewritingSystem,
};

mod benchmark;
mod data_union_find;
mod index_selector;
mod language;
mod rewriting;
mod seen;
mod union_find;
#[macro_use]
mod macros;

fn get_trs() -> TermRewritingSystem {
    let lang = Language::simple_math();
    let rules = rules!(lang;
        "(* $0 2)" => "(<< $0 1)",
        "(* $0 1)" => "$0",
        "(/ (* $0 $1) $2)" => "(* $0 (/ $1 $2))",
        "(/ $0 $0)" => "1",
    );
    TermRewritingSystem::new(lang, rules)
}

fn main() {
    let trs = get_trs();
    let lang = trs.language();

    let expressions = vec![
        lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap(),
        lang.parse_no_vars("(+ 1 1)").unwrap(),
    ];

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
