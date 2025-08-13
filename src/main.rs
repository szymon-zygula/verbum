#![allow(dead_code)]

use language::Language;
use rewriting::system::TermRewritingSystem;

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

    // Something is wrong here, the extracted expressions are completely wrong. Why?
    let trs = TermRewritingSystem::new(lang.clone(), rules);
    let expressions = vec![
        lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap(),
        lang.parse_no_vars("(+ 1 1)").unwrap(),
    ];

    let config = benchmark::BenchmarkConfig::default();
    let extractor = crate::rewriting::egraph::extraction::SimpleExtractor::<usize, _, _, ()>::new(
        |_| 1,
        |_, _| Some(0),
    );
    let outcomes = benchmark::benchmark(&trs, expressions, &config, &extractor);

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
