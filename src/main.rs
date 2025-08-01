#![allow(dead_code)]

use language::Language;
use rewriting::{
    egraph::{EGraph, matching::bottom_up::BottomUpMatcher, saturation::saturate},
    rule::Rule,
};

mod data_union_find;
mod index_selector;
mod language;
mod rewriting;
mod seen;
mod union_find;

fn main() {
    let lang = Language::math();
    let rules = vec![
        Rule::from_strings("(* x0 2)", "(<< x0 1)", &lang),
        Rule::from_strings("(* x0 1)", "x0", &lang),
        Rule::from_strings("(/ (* x0 x1) x2)", "(* x0 (/ x1 x2))", &lang),
        Rule::from_strings("(/ x0 x0)", "1", &lang),
    ];

    let mut egraph = EGraph::from_expression(lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap());

    saturate(&mut egraph, &rules, BottomUpMatcher);

    egraph.save_dot(&lang, "test.dot").unwrap();
}
