#![allow(dead_code)]

use language::Language;
use rewriting::{
    egraph::{
        extraction::{SimpleExtractor, children_cost_sum},
        saturation::{SaturationConfig, DefaultSaturator},
        matching::bottom_up::BottomUpMatcher,
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

fn main() {
    let lang = Language::simple_math();
    let rules = rules!(lang;
        "(* $0 2)" => "(<< $0 1)",
        "(* $0 1)" => "$0",
        "(/ (* $0 $1) $2)" => "(* $0 (/ $1 $2))",
        "(/ $0 $0)" => "1",
    );

    let trs = TermRewritingSystem::new(lang.clone(), rules);
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

    let saturator = DefaultSaturator::new(BottomUpMatcher);

    let outcomes = benchmark::benchmark(&trs, expressions, &config, &extractor, &saturator);

    use benchmark::{
        OutcomeFormatter, csv_output::CsvFormatter, pretty_printing::PrettyTableFormatter,
    };

    let pretty_formatter = PrettyTableFormatter;
    let pretty_output = pretty_formatter.format_outcomes(&outcomes);
    println!("{pretty_output}");

    let csv_formatter = CsvFormatter;
    let csv_output = csv_formatter.format_outcomes(&outcomes);
    println!("\nCSV Output:\n{csv_output}");
}

