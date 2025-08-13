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
    let outcomes = benchmark::benchmark::<()>(&trs, expressions, &config);

    benchmark::pretty_printing::print_table(&outcomes);

    use benchmark::csv_output::{CsvFormatter, OutcomeFormatter};
    let csv_formatter = CsvFormatter;
    let csv_output = csv_formatter.format_outcomes(&outcomes);
    println!("\nCSV Output:\n{csv_output}");
}
