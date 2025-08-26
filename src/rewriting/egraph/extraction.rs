use std::{collections::HashMap, iter::Sum, marker::PhantomData};

use crate::language::{
    expression::{Literal, VarFreeExpression},
    symbol::Symbol,
};

use super::{ClassId, DynEGraph, Node, NodeId};

#[derive(Clone, Debug)]
pub struct ExtractionResult<C> {
    winner: VarFreeExpression,
    cost: C,
}

impl<C> ExtractionResult<C> {
    pub fn winner(&self) -> &VarFreeExpression {
        &self.winner
    }

    pub fn cost(&self) -> &C {
        &self.cost
    }

    pub fn cost_value(self) -> C {
        self.cost
    }
}

pub trait Extractor {
    type Cost;

    /// Finds the cheapest expression represented by `egraph` that's represented by class with id `equivalent`.
    fn extract(
        &self,
        egraph: &dyn DynEGraph,
        equivalent: ClassId,
    ) -> Option<ExtractionResult<Self::Cost>>;
}

trait_set::trait_set! {
    pub trait SimpleSymbolCost<C> = Fn(&Symbol<ClassId>, &HashMap<ClassId, C>) -> Option<C>;
    pub trait SimpleLiteralCost<C> = Fn(&Literal) -> C;
}

/// A simple extractor, finding the cheapest class, calculating the cost
/// using given functions
pub struct SimpleExtractor<C, SC, LC>
where
    C: Ord + PartialEq + Clone,
    LC: SimpleLiteralCost<C>,
    SC: SimpleSymbolCost<C>,
{
    literal_cost: LC,
    symbol_cost: SC,
    cost: PhantomData<C>,
}

impl<C, SC, LC> SimpleExtractor<C, SC, LC>
where
    C: Ord + PartialEq + Clone,
    LC: SimpleLiteralCost<C>,
    SC: SimpleSymbolCost<C>,
{
    pub fn new(literal_cost: LC, symbol_cost: SC) -> Self {
        Self {
            literal_cost,
            symbol_cost,
            cost: PhantomData,
        }
    }

    fn node_cost(&self, node: &Node, costs: &HashMap<ClassId, C>) -> Option<C> {
        match node {
            Node::Literal(literal) => Some((self.literal_cost)(literal)),
            Node::Symbol(symbol) => (self.symbol_cost)(symbol, costs),
        }
    }

    fn calculate_costs(
        &self,
        egraph: &dyn DynEGraph,
    ) -> (HashMap<ClassId, NodeId>, HashMap<ClassId, C>) {
        let mut work_remaining = true;
        let mut class_costs = HashMap::new();
        let mut node_costs = HashMap::new();
        let mut cheapest_nodes = HashMap::new();

        while work_remaining {
            work_remaining = false;

            for (&class_id, class) in egraph.dyn_classes() {
                for &node_id in class.iter_nodes() {
                    let node = egraph.node(node_id);
                    if let Some(node_cost) = self.node_cost(node, &class_costs) {
                        if let Some(old_cost) = node_costs.get(&node_id)
                            && *old_cost == node_cost
                        {
                            continue;
                        }

                        node_costs.insert(node_id, node_cost);
                        work_remaining = true;
                    }
                }

                if let Some((min_node_id, min_node_cost)) = class
                    .iter_nodes()
                    .filter_map(|node_id| Some((*node_id, node_costs.get(node_id)?)))
                    .min_by_key(|x| x.1)
                {
                    class_costs.insert(class_id, min_node_cost.clone());
                    cheapest_nodes.insert(class_id, min_node_id);
                }
            }
        }

        (cheapest_nodes, class_costs)
    }

    fn extract_expression(
        egraph: &dyn DynEGraph,
        cheapest_nodes: &HashMap<ClassId, NodeId>,
        class_id: ClassId,
    ) -> VarFreeExpression {
        match egraph.node(*cheapest_nodes.get(&class_id).expect(concat!(
            "Found a class for which cost could not be determined.",
            "Have you defined correct costs for all symbols in the language?"
        ))) {
            Node::Literal(literal) => VarFreeExpression::Literal(literal.clone()),
            Node::Symbol(symbol) => VarFreeExpression::Symbol(Symbol {
                id: symbol.id,
                children: symbol
                    .children
                    .iter()
                    .map(|&child_id| Self::extract_expression(egraph, cheapest_nodes, child_id))
                    .collect(),
            }),
        }
    }
}

pub fn children_cost_sum<C>(symbol: &Symbol<ClassId>, costs: &HashMap<ClassId, C>) -> Option<C>
where
    C: Ord + PartialEq + Clone + Sum,
{
    symbol
        .children
        .iter()
        .map(|child_id| costs.get(child_id).cloned())
        .sum::<Option<C>>()
}

impl<C, SC, LC> Extractor for SimpleExtractor<C, SC, LC>
where
    C: Ord + PartialEq + Clone,
    LC: SimpleLiteralCost<C>,
    SC: SimpleSymbolCost<C>,
{
    type Cost = C;

    fn extract(
        &self,
        egraph: &dyn DynEGraph,
        equivalent: ClassId,
    ) -> Option<ExtractionResult<Self::Cost>> {
        let equivalent = egraph.canonical_class(equivalent);
        let (cheapest_nodes, class_costs) = self.calculate_costs(egraph);

        Some(ExtractionResult {
            winner: Self::extract_expression(egraph, &cheapest_nodes, equivalent),
            cost: class_costs.get(&equivalent)?.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        language::Language,
        rewriting::{
            egraph::{
                DynEGraph, EGraph,
                matching::bottom_up::BottomUpMatcher,
                saturation::{SaturationConfig, Saturator, SimpleSaturator},
            },
            rule::Rule,
        },
    };

    use super::{Extractor, SimpleExtractor, children_cost_sum};

    #[test]
    fn literal_cost() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::default();

        let expr_1 = lang.parse_no_vars("4u").unwrap();
        let expr_2 = lang.parse_no_vars("8u").unwrap();

        let node_1_id = egraph.add_expression(expr_1.clone());
        let class_1_id = egraph.containing_class(node_1_id);
        let node_2_id = egraph.add_expression(expr_2);
        let class_2_id = egraph.containing_class(node_2_id);

        let merged_id = egraph.merge_classes(class_1_id, class_2_id).new().unwrap();

        let extractor = SimpleExtractor::<usize, _, _>::new(
            |literal| match literal {
                crate::language::expression::Literal::UInt(x) => *x as usize,
                crate::language::expression::Literal::Int(_) => 0,
            },
            |_, _| Some(0),
        );

        let extraction_result = extractor.extract(&egraph, merged_id).unwrap();

        assert_eq!(extraction_result.winner, expr_1);
        assert_eq!(extraction_result.cost, 4);
    }

    #[test]
    fn symbol_cost() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::default();

        let expr_1 = lang.parse_no_vars("(+ 2u 3u)").unwrap();
        let expr_2 = lang.parse_no_vars("(* 1u 2u)").unwrap();

        let node_1_id = egraph.add_expression(expr_1.clone());
        let class_1_id = egraph.containing_class(node_1_id);
        let node_2_id = egraph.add_expression(expr_2);
        let class_2_id = egraph.containing_class(node_2_id);

        let merged_id = egraph.merge_classes(class_1_id, class_2_id).new().unwrap();

        let extractor = SimpleExtractor::<usize, _, _>::new(
            |literal| match literal {
                crate::language::expression::Literal::UInt(x) => *x as usize,
                crate::language::expression::Literal::Int(_) => 0,
            },
            |symbol, costs| {
                Some(match lang.get_symbol(symbol.id) {
                    "+" => 1usize + children_cost_sum(symbol, costs)?,
                    "*" => 1usize + 2usize * children_cost_sum(symbol, costs)?,
                    _ => None?,
                })
            },
        );

        let extraction_result = extractor.extract(&egraph, merged_id).unwrap();

        assert_eq!(extraction_result.winner, expr_1);
        assert_eq!(extraction_result.cost, 6);
    }

    #[test]
    fn saturated_cost() {
        let lang = Language::simple_math();
        let rules = vec![
            Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang),
            Rule::from_strings("(* $0 1)", "$0", &lang),
            Rule::from_strings("(/ (* $0 $1) $2)", "(* $0 (/ $1 $2))", &lang),
            Rule::from_strings("(/ $0 $0)", "1", &lang),
        ];

        let mut egraph =
            EGraph::<()>::from_expression(lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap());
        let top_class_id = egraph.containing_class(egraph.find_symbols(lang.get_id("/"))[0]);

        let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
        let _ = saturator.saturate(&mut egraph, &rules, &SaturationConfig::default());

        let extractor = SimpleExtractor::<usize, _, _>::new(
            |_| 1,
            |symbol, costs| {
                Some(match lang.get_symbol(symbol.id) {
                    "/" => 8usize + children_cost_sum(symbol, costs)?,
                    "*" => 4usize + children_cost_sum(symbol, costs)?,
                    "<<" => 2usize + children_cost_sum(symbol, costs)?,
                    "sin" => 2usize + children_cost_sum(symbol, costs)?,
                    _ => None?,
                })
            },
        );

        let extraction_result = extractor.extract(&egraph, top_class_id).unwrap();

        assert_eq!(
            extraction_result.winner,
            lang.parse_no_vars("(sin 5)").unwrap()
        );
        assert_eq!(extraction_result.cost, 3);
    }
}
