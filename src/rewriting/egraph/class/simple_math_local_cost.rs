use std::{
    iter::Sum,
    ops::{Add, Sub},
};

use crate::language::{Language, expression::Literal, symbol::SymbolId};

use super::local_cost::LocalCost;

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpleMathLocalCost(i32);

impl Sum for SimpleMathLocalCost {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        Self(iter.map(|x| x.0).sum())
    }
}

impl Add for SimpleMathLocalCost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sub for SimpleMathLocalCost {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl LocalCost for SimpleMathLocalCost {
    fn symbol_cost(symbol_id: SymbolId) -> Self {
        let lang = Language::simple_math();
        Self(match lang.get_symbol(symbol_id) {
            "+" => 1,
            "-" => 1,
            "*" => 4,
            "/" => 8,
            "<<" => 2,
            "sin" => 2,
            _ => 1,
        })
    }

    fn literal_cost(_: &Literal) -> Self {
        Self(1)
    }
}

#[cfg(test)]
mod tests {
    use crate::language::Language;
    use crate::rewriting::egraph::{
        DynEGraph, EGraph,
        matching::bottom_up::BottomUpMatcher,
        saturation::{SaturationConfig, Saturator, SimpleSaturator},
    };
    use crate::rewriting::rule::Rule;

    use super::SimpleMathLocalCost;

    #[test]
    fn test_literal_cost() {
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("5").unwrap();
        let egraph = EGraph::<SimpleMathLocalCost>::from_expression(expr);
        let class_id = egraph.containing_class(egraph.node_id(egraph.node(0)).unwrap());
        assert_eq!(egraph.class(class_id).analysis().0, 1);

        let expr = lang.parse_no_vars("-10").unwrap();
        let egraph = EGraph::<SimpleMathLocalCost>::from_expression(expr);
        let class_id = egraph.containing_class(egraph.node_id(egraph.node(0)).unwrap());
        assert_eq!(egraph.class(class_id).analysis().0, 1);
    }

    #[test]
    fn test_symbol_cost() {
        let lang = Language::simple_math();
        let (egraph, root_class_id) = EGraph::<SimpleMathLocalCost>::from_expression_with_id(
            lang.parse_no_vars("(+ 1 2)").unwrap(),
        );
        // Cost of '+' is 1, children are 1 and 2. Total cost should be 1 + 1 + 2 = 4
        assert_eq!(egraph.class(root_class_id).analysis().0, 3);

        let (egraph, root_class_id) = EGraph::<SimpleMathLocalCost>::from_expression_with_id(
            lang.parse_no_vars("(* 3 4)").unwrap(),
        );
        // Cost of '*' is 4, children are 1 and 1. Total cost should be 4 + 1 + 1 = 6
        assert_eq!(egraph.class(root_class_id).analysis().0, 6);
    }

    #[test]
    fn test_merge_cost() {
        let lang = Language::simple_math();
        let (mut egraph, root_class_id_1) = EGraph::<SimpleMathLocalCost>::from_expression_with_id(
            lang.parse_no_vars("(+ 1 2)").unwrap(),
        ); // Cost 1+1+1 = 3
        let (_egraph_2, root_class_id_2) = EGraph::<SimpleMathLocalCost>::from_expression_with_id(
            lang.parse_no_vars("(* 1 2)").unwrap(),
        ); // Cost 4+1+1 = 6

        egraph.merge_classes(root_class_id_1, root_class_id_2);
        let merged_class_id = egraph.containing_class(root_class_id_1);

        // The merged class should have the minimum cost of the two merged classes
        assert_eq!(egraph.class(merged_class_id).analysis().0, 3);
    }

    #[test]
    fn test_saturated_cost_with_local_cost() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang),
            Rule::from_strings("(* $0 1)", "$0", &lang),
            Rule::from_strings("(/ (* $0 $1) $2)", "(* $0 (/ $1 $2))", &lang),
            Rule::from_strings("(/ $0 $0)", "1", &lang),
        ];

        let (mut egraph, top_class_id) = EGraph::<SimpleMathLocalCost>::from_expression_with_id(
            lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap(),
        );

        let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
        let _ = saturator.saturate(&mut egraph, &rules, &SaturationConfig::default());

        let extracted_cost = egraph.class(top_class_id).analysis().0;

        // The expression "(/ (* (sin 5) 2) 2)" should simplify to "(sin 5)"
        // Cost of "sin" is 2, cost of "5" (literal) is 1. Total cost = 2 + 1 = 3
        assert_eq!(extracted_cost, 3);
    }
}
