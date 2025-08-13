#![allow(dead_code)]

use language::Language;
use rewriting::egraph::{
    EGraph,
    matching::bottom_up::BottomUpMatcher,
    saturation::{SaturationConfig, saturate},
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
        "(* x0 2)" => "(<< x0 1)",
        "(* x0 1)" => "x0",
        "(/ (* x0 x1) x2)" => "(* x0 (/ x1 x2))",
        "(/ x0 x0)" => "1",
    );

    let mut egraph = EGraph::<()>::from_expression(lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap());

    let _ = saturate(
        &mut egraph,
        &rules,
        BottomUpMatcher,
        &SaturationConfig::default(),
    );

    egraph.save_dot(&lang, "test.dot").unwrap();
}
