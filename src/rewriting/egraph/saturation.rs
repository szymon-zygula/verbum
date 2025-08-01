use crate::rewriting::rule::Rule;

use super::{EGraph, matching::Matcher};

fn saturate(egraph: &mut EGraph, rules: &Vec<Rule>, matcher: impl Matcher) {
    while rules
        .iter()
        .any(|rule| !rule.apply(egraph, &matcher).is_empty())
    {}
}

#[cfg(test)]
mod tests {
    use crate::{
        language::Language,
        rewriting::{
            egraph::{EGraph, matching::bottom_up::BottomUpMatcher},
            rule::Rule,
        },
    };

    use super::saturate;

    #[test]
    fn classical_test() {
        let lang = Language::math();
        let rules = vec![
            Rule::from_strings("(* x0 2)", "(<< x0 1)", &lang),
            Rule::from_strings("(/ (* x0 x1) x2)", "(* x0 (/ x1 x2))", &lang),
            Rule::from_strings("(/ x1 x1)", "1", &lang),
        ];

        let mut egraph =
            EGraph::from_expression(lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap());

        saturate(&mut egraph, &rules, BottomUpMatcher);
    }
}
