pub mod class;
pub mod drawing;
pub mod extraction;
pub mod matching;
pub mod node;
pub mod saturation;

pub use class::Class;
use class::DynClass;
pub use class::analysis::Analysis;
pub use node::Node;

use std::collections::{HashMap, HashSet, hash_map};

use itertools::Itertools;

use crate::{
    language::{
        expression::{Literal, MixedExpression, VarFreeExpression},
        symbol::{Symbol, SymbolId},
    },
    seen::Seen,
    union_find::UnionFind,
};

pub type NodeId = usize;
pub type ClassId = usize;

#[derive(Default, Clone)]
pub struct EGraph<A: Analysis> {
    union_find: UnionFind,
    nodes: HashMap<NodeId, Node>,
    // Always kept behind canonical IDs
    classes: HashMap<ClassId, Class<A>>,
}

impl<A: Analysis> EGraph<A> {
    /// Creates an e-graph representing `expression` and returns it together with ID of the class
    /// containing the root expression.
    pub fn from_expression_with_id(expression: VarFreeExpression) -> (Self, ClassId) {
        let mut egraph = Self::default();

        let class_id = egraph.add_expression(expression);

        (egraph, class_id)
    }

    /// Creates an e-graph representing `expression`.
    pub fn from_expression(expression: VarFreeExpression) -> Self {
        Self::from_expression_with_id(expression).0
    }

    /// Adds a node to the egraph, returning `Old(id)` if the node exists, or `New(id)` if the node
    /// has been added by this call
    fn add_node(&mut self, node: Node) -> Seen<NodeId> {
        if let Some(&old_id) = self.node_id(&node).as_ref() {
            return Seen::Old(old_id);
        }

        let class_id = self.union_find.add();
        let node_id = class_id;

        if let Node::Symbol(symbol) = &node {
            for child in &symbol.children {
                self.add_parent(*child, node_id);
            }
        }

        self.nodes.insert(node_id, node);

        let class = Class::from_node(self, node_id);
        self.classes.insert(class_id, class);

        Seen::New(node_id)
    }

    fn add_parent(&mut self, class_id: ClassId, parent_id: NodeId) {
        self.class_mut(class_id).parents_ids_mut().insert(parent_id);
    }

    pub fn iter_classes(&self) -> hash_map::Iter<'_, usize, Class<A>> {
        self.classes.iter()
    }

    pub fn class(&self, class_id: ClassId) -> &Class<A> {
        let class_id = self.canonical_class(class_id);
        &self.classes[&class_id]
    }

    pub fn class_mut(&mut self, class_id: ClassId) -> &mut Class<A> {
        let class_id = self.canonical_class(class_id);
        self.classes.get_mut(&class_id).unwrap()
    }

    /// Makes all nodes in an eclass have canonical children
    fn make_class_canonical(&mut self, class_id: ClassId) {
        let class_id = self.canonical_class(class_id);
        let class = &self.classes[&class_id];
        for &node_id in class.nodes_ids() {
            // Make node canonical
            for child_id in self.nodes.get_mut(&node_id).unwrap().iter_mut_children() {
                *child_id = self.union_find.find(*child_id);
            }
        }
    }

    fn rebuild_class(&mut self, class_id: ClassId) {
        // Canonicalize nodes in this class
        self.make_class_canonical(class_id);

        let class = &self.classes[&class_id];

        // Find duplicate nodes
        let mut to_remove = Vec::new();
        for (&node_id_1, &node_id_2) in class
            .nodes_ids()
            .iter()
            .cartesian_product(class.nodes_ids().iter())
            .filter(|(id_1, id_2)| id_1 < id_2)
        {
            if self.node(node_id_1) == self.node(node_id_2) {
                to_remove.push(node_id_1);
            }
        }

        // Remove duplicate nodes
        let class = self.classes.get_mut(&class_id).unwrap();
        for node_id in to_remove {
            class.nodes_ids_mut().remove(&node_id);
        }

        // Make all nodes in parents have canonical class IDs
        let class = &self.classes[&class_id];
        for parent in class.parents_ids() {
            for child in self.nodes.get_mut(parent).unwrap().iter_mut_children() {
                *child = self.union_find.find(*child);
            }
        }

        // Check if there are parents who share identical nodes
        let mut to_merge = Vec::new();
        for (parent_1_id, parent_2_id) in class
            .parents_ids()
            .iter()
            .cartesian_product(class.parents_ids().iter())
            // So that we don't compare same things twice
            .filter(|(id_1, id_2)| id_1 < id_2)
        {
            let class_1_id = self.containing_class(*parent_1_id);
            let class_2_id = self.containing_class(*parent_2_id);

            if class_1_id != class_2_id && self.nodes[parent_1_id] == self.nodes[parent_2_id] {
                to_merge.push((class_1_id, class_2_id));
            }
        }

        // Merge parents with identical nodes
        for (class_1_id, class_2_id) in to_merge {
            self.merge_classes(class_1_id, class_2_id);
        }
    }

    /// Returns the list of `NodeId`s of nodes which have the same shape as
    /// `symbol` and are contained in the class with id `class_id`.
    fn same_shape_symbols<E>(&self, class_id: ClassId, symbol: &Symbol<E>) -> Vec<NodeId> {
        self.class(class_id)
            .iter_nodes()
            .filter_map(|&node_id| {
                let node = self.node(node_id);
                let node_symbol = node.try_as_symbol()?;
                node_symbol.same_shape_as(symbol).then_some(node_id)
            })
            .collect_vec()
    }
}

pub trait DynEGraph {
    /// If a given node exists in the e-graph, returns it. Otherwise gives `None`.
    fn node_id(&self, node: &Node) -> Option<NodeId>;

    fn canonical_class(&self, class_id: ClassId) -> ClassId;

    /// Adds `expression` to the e-graph as a new class.
    fn add_expression(&mut self, expression: VarFreeExpression) -> NodeId;

    fn add_mixed_expression(&mut self, expression: MixedExpression) -> Seen<ClassId>;

    fn node_mut(&mut self, node_id: NodeId) -> &mut Node;
    fn class_count(&self) -> usize;

    /// Returns the total number of nodes which were added to the egraph at any point
    fn total_node_count(&self) -> usize;

    /// Returns the current number of nodes in the egraph
    fn actual_node_count(&self) -> usize;

    fn parents(&self, class_id: ClassId) -> &HashSet<NodeId>;

    fn nodes(&self, class_id: ClassId) -> &HashSet<NodeId>;

    fn node(&self, node_id: NodeId) -> &Node;

    fn containing_class(&self, node_id: NodeId) -> ClassId;

    /// Merges given classes, returns the canonical ID of the merged class as `Old(id)`
    /// if the IDs refered to a single class already, or `New(id)` otherwise
    fn merge_classes(&mut self, class_1_id: ClassId, class_2_id: ClassId) -> Seen<ClassId>;

    /// Finds symbols with a specified ID
    fn find_symbols(&self, symbol_id: SymbolId) -> Vec<NodeId>;

    /// Checks if a given literal is contained in the e-graph and if so, returns its class ID.
    fn find_literal(&self, literal: Literal) -> Option<ClassId>;

    /// Checks if a given symbol is contained in the e-graph and if so, returns its class ID.
    fn find_symbol(&self, symbol: Symbol<ClassId>) -> Option<ClassId>;

    /// `true` if the class with id `class_id` contains a node whose type is literal and
    /// is identical to `literal`, `false` otherwise
    fn class_contains_literal(&self, class_id: ClassId, literal: &Literal) -> bool;

    fn dyn_classes(&self) -> Vec<(&usize, &dyn DynClass)>;

    fn dyn_class(&self, class_id: ClassId) -> &dyn DynClass;

    fn dyn_class_mut(&mut self, class_id: ClassId) -> &mut dyn DynClass;
}

impl<A: Analysis> DynEGraph for EGraph<A> {
    /// If a given node exists in the e-graph, returns it. Otherwise gives `None`.
    fn node_id(&self, node: &Node) -> Option<NodeId> {
        // TODO: Add hashcons so that this is speedy
        let node = node.canonical(self);

        for (&candidate_id, candidate_node) in self.nodes.iter() {
            let candidate_node = candidate_node.canonical(self);
            if candidate_node == node {
                return Some(candidate_id);
            }
        }

        None
    }

    fn canonical_class(&self, class_id: ClassId) -> ClassId {
        self.union_find.find(class_id)
    }

    /// Adds `expression` to the e-graph as a new class.
    fn add_expression(&mut self, expression: VarFreeExpression) -> NodeId {
        let node = match expression {
            VarFreeExpression::Literal(literal) => Node::Literal(literal),
            VarFreeExpression::Symbol(symbol) => Node::Symbol(Symbol {
                id: symbol.id,
                children: symbol
                    .children
                    .into_iter()
                    .map(|child| self.add_expression(child))
                    .collect(),
            }),
        };

        self.add_node(node).any()
    }

    fn add_mixed_expression(&mut self, expression: MixedExpression) -> Seen<ClassId> {
        match expression {
            MixedExpression::Literal(literal) => {
                let node_id = self.add_node(Node::Literal(literal));
                node_id.map(|node_id| self.containing_class(node_id))
            }
            MixedExpression::Symbol(symbol) => {
                let node = Node::Symbol(Symbol {
                    id: symbol.id,
                    children: symbol
                        .children
                        .into_iter()
                        .map(|child| self.add_mixed_expression(child).any())
                        .collect(),
                });

                self.add_node(node)
                    .map(|node_id| self.containing_class(node_id))
            }
            MixedExpression::Class(class_id) => Seen::Old(class_id),
        }
    }

    fn node_mut(&mut self, node_id: NodeId) -> &mut Node {
        self.nodes.get_mut(&node_id).unwrap()
    }

    fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// Returns the total number of nodes which were added to the egraph at any point
    fn total_node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the current number of nodes in the egraph
    fn actual_node_count(&self) -> usize {
        self.classes
            .values()
            .map(|class| class.nodes_ids().len())
            .sum()
    }

    fn parents(&self, class_id: ClassId) -> &HashSet<NodeId> {
        self.class(class_id).parents_ids()
    }

    fn nodes(&self, class_id: ClassId) -> &HashSet<NodeId> {
        self.class(class_id).nodes_ids()
    }

    fn node(&self, node_id: NodeId) -> &Node {
        &self.nodes[&node_id]
    }

    fn containing_class(&self, node_id: NodeId) -> ClassId {
        // On creation of a node, its containing class has the same id.
        // The canonical ID of the class might change, but UF will give it to us.
        self.union_find.find(node_id)
    }

    /// Merges given classes, returns the canonical ID of the merged class as `Old(id)`
    /// if the IDs refered to a single class already, or `New(id)` otherwise
    fn merge_classes(&mut self, class_1_id: ClassId, class_2_id: ClassId) -> Seen<ClassId> {
        let class_1_id = self.union_find.find(class_1_id);
        let class_2_id = self.union_find.find(class_2_id);

        if class_1_id == class_2_id {
            return Seen::Old(class_1_id);
        }

        self.union_find.union(class_1_id, class_2_id);

        let class_1 = self.classes.remove(&class_1_id).unwrap();
        self.classes.get_mut(&class_2_id).unwrap().merge(class_1);

        self.rebuild_class(class_2_id);

        Seen::New(class_2_id)
    }
    /// Finds symbols with a specified ID
    fn find_symbols(&self, symbol_id: SymbolId) -> Vec<NodeId> {
        self.nodes
            .iter()
            .filter_map(|(&id, node)| (node.try_as_symbol()?.id == symbol_id).then_some(id))
            .collect()
    }

    /// Checks if a given literal is contained in the e-graph and if so, returns its class ID.
    fn find_literal(&self, literal: Literal) -> Option<ClassId> {
        Some(self.containing_class(self.node_id(&Node::Literal(literal))?))
    }

    /// Checks if a given symbol is contained in the e-graph and if so, returns its class ID.
    fn find_symbol(&self, symbol: Symbol<ClassId>) -> Option<ClassId> {
        Some(self.containing_class(self.node_id(&Node::Symbol(symbol))?))
    }

    /// `true` if the class with id `class_id` contains a node whose type is literal and
    /// is identical to `literal`, `false` otherwise
    fn class_contains_literal(&self, class_id: ClassId, literal: &Literal) -> bool {
        self.class(class_id)
            .iter_nodes()
            .filter(|&&node_id| {
                let Node::Literal(node_literal) = self.node(node_id) else {
                    return false;
                };

                *node_literal == *literal
            })
            .count()
            > 0
    }

    fn dyn_classes(&self) -> Vec<(&usize, &dyn DynClass)> {
        self.classes
            .iter()
            .map(|(k, v)| (k, v as &dyn DynClass))
            .collect_vec()
    }

    fn dyn_class(&self, class_id: ClassId) -> &dyn DynClass {
        let class_id = self.canonical_class(class_id);
        &self.classes[&class_id]
    }

    fn dyn_class_mut(&mut self, class_id: ClassId) -> &mut dyn DynClass {
        let class_id = self.canonical_class(class_id);
        self.classes.get_mut(&class_id).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use crate::{
        language::{
            Language,
            expression::{Literal, VarFreeExpression},
            symbol::Symbol,
        },
        rewriting::egraph::{DynEGraph, Node, class::DynClass},
    };

    use super::EGraph;

    #[test]
    fn from_expression() {
        let lang = Language::simple_math();
        let expression: VarFreeExpression = lang.parse_no_vars("(* 1 2 3 (+ 4 5))").unwrap();

        let graph = EGraph::<()>::from_expression(expression);

        assert_eq!(graph.class_count(), 7);
        assert_eq!(graph.total_node_count(), 7);
        assert_eq!(graph.actual_node_count(), 7);
    }

    #[test]
    fn merge() {
        let lang = Language::simple_math();
        let expression_1: VarFreeExpression = lang.parse_no_vars("(+ 1 5)").unwrap();
        let expression_2: VarFreeExpression = lang.parse_no_vars("(+ 2 4)").unwrap();

        let mut graph = EGraph::<()>::default();

        let node_1_id = graph.add_expression(expression_1);
        let node_2_id = graph.add_expression(expression_2);

        assert_eq!(graph.class_count(), 6);
        assert_eq!(graph.total_node_count(), 6);
        assert_eq!(graph.actual_node_count(), 6);

        let class_1_id = graph.containing_class(node_1_id);
        let class_2_id = graph.containing_class(node_2_id);

        graph.merge_classes(class_1_id, class_2_id);

        let class_1_id = graph.containing_class(node_1_id);
        let class_2_id = graph.containing_class(node_2_id);

        assert_eq!(class_1_id, class_2_id);
        assert_eq!(graph.class_count(), 5);
        assert_eq!(graph.total_node_count(), 6);
        assert_eq!(graph.actual_node_count(), 6);
    }

    #[test]
    fn node_and_node_id() {
        let mut egraph = EGraph::<()>::default();

        let nodes = (0..10)
            .map(|i| Node::Literal(Literal::UInt(i)))
            .collect_vec();
        let node_ids = nodes
            .iter()
            .cloned()
            .map(|node| egraph.add_node(node).any())
            .collect_vec();

        for i in 0..10 {
            assert_eq!(*egraph.node(node_ids[i]), nodes[i]);
            assert_eq!(egraph.node_id(&nodes[i]).unwrap(), node_ids[i]);
        }
    }

    #[test]
    fn parents() {
        let lang = Language::simple_math();
        let expression: VarFreeExpression = lang.parse_no_vars("(+ 1 5)").unwrap();

        let mut graph = EGraph::<()>::default();

        let node_id = graph.add_expression(expression);
        let class_id = graph.containing_class(node_id);

        assert!(graph.parents(class_id).is_empty());
        assert_eq!(graph.nodes(class_id).len(), 1);
        {}
        let Node::Symbol(plus) = graph.node(*graph.nodes(class_id).iter().next().unwrap()) else {
            panic!("Unxepected variant")
        };

        for child_id in &plus.children {
            let parent_ids = &graph.class(*child_id).parents_ids();
            assert_eq!(parent_ids.len(), 1);
            assert_eq!(*parent_ids.iter().next().unwrap(), node_id);
        }
    }

    #[test]
    fn double_node_addition() {
        let mut egraph = EGraph::<()>::default();

        let literal_1 = Node::Literal(Literal::UInt(5));
        let literal_2 = Node::Literal(Literal::UInt(7));

        let node_1_id = egraph.add_node(literal_1.clone()).new().unwrap();
        let node_1_id_ = egraph.add_node(literal_1).old().unwrap();
        assert_eq!(node_1_id, node_1_id_);

        let node_2_id = egraph.add_node(literal_2).any();

        let class_1_id = egraph.containing_class(node_1_id);
        let class_2_id = egraph.containing_class(node_2_id);

        let symbol_1 = Node::Symbol(Symbol {
            id: 0,
            children: vec![class_1_id, class_2_id],
        });

        let node_3_id = egraph.add_node(symbol_1.clone());

        egraph.merge_classes(class_1_id, class_2_id);

        let symbol_2 = Node::Symbol(Symbol {
            id: 0,
            children: vec![class_2_id, class_1_id],
        });
        let node_4_id = egraph.add_node(symbol_2.clone());

        assert_eq!(egraph.node_id(&symbol_1), egraph.node_id(&symbol_2));
        assert_eq!(node_3_id.new().unwrap(), node_4_id.old().unwrap());
    }

    #[test]
    fn literal_sharing() {
        let mut egraph = EGraph::<()>::default();

        let lang = Language::simple_math();
        let expr_1: VarFreeExpression = lang.parse_no_vars("(+ 1 5)").unwrap();
        let expr_2: VarFreeExpression = lang.parse_no_vars("(* 6 5)").unwrap();

        egraph.add_expression(expr_1);
        egraph.add_expression(expr_2);

        assert_eq!(egraph.actual_node_count(), 5);
        assert_eq!(egraph.total_node_count(), 5);
        assert_eq!(egraph.class_count(), 5);
    }

    #[test]
    fn class_merge_with_same_nodes() {
        let mut egraph = EGraph::<()>::default();

        let lang = Language::simple_math();
        let expr_1: VarFreeExpression = lang.parse_no_vars("(+ 1 5)").unwrap();
        let expr_2: VarFreeExpression = lang.parse_no_vars("(+ 1 2)").unwrap();

        let expr_1_id = egraph.add_expression(expr_1);
        let expr_2_id = egraph.add_expression(expr_2);

        let class_5 = egraph.node(expr_1_id).as_symbol().children[1];
        let class_2 = egraph.node(expr_2_id).as_symbol().children[1];

        egraph.merge_classes(class_5, class_2);

        assert_eq!(egraph.total_node_count(), 5);
        assert_eq!(egraph.class_count(), 3);
        assert_eq!(egraph.actual_node_count(), 4);
    }

    #[test]
    fn cascading_rebuild() {
        let lang = Language::simple_math();

        let expr = lang
            .parse_no_vars("(* (+ 5 (sin (* 1 7))) (+ 5 (sin (* 1 8))))")
            .unwrap();

        let mut egraph = EGraph::<()>::from_expression(expr);

        let id_7 = egraph.node_id(&Node::Literal(Literal::Int(7))).unwrap();
        let id_8 = egraph.node_id(&Node::Literal(Literal::Int(8))).unwrap();

        let id_7_class = egraph.containing_class(id_7);
        let id_8_class = egraph.containing_class(id_8);

        assert_eq!(egraph.total_node_count(), 11);
        assert_eq!(egraph.actual_node_count(), 11);
        assert_eq!(egraph.class_count(), 11);

        egraph.merge_classes(id_7_class, id_8_class);

        assert_eq!(egraph.total_node_count(), 11);
        assert_eq!(egraph.actual_node_count(), 8);
        assert_eq!(egraph.class_count(), 7);
    }

    #[test]
    fn self_merge() {
        let lang = Language::simple_math();
        let mut egraph =
            EGraph::<()>::from_expression(lang.parse_no_vars("(+ 2 (sin 5))").unwrap());

        egraph.merge_classes(0, 1);
        egraph.merge_classes(0, 2);

        let class_count = egraph.class_count();

        // These should not panic
        egraph.merge_classes(0, 0);
        egraph.merge_classes(0, 1);
        egraph.merge_classes(0, 2);

        assert_eq!(egraph.class_count(), class_count);
    }

    #[test]
    fn literal_count_analysis() {
        use super::class::literal_count::LiteralCountAnalysis;
        let lang = Language::simple_math();
        let expr = lang.parse_no_vars("(* 1 2 (+ 3 4))").unwrap();
        let mut egraph = EGraph::<LiteralCountAnalysis>::from_expression(expr);

        // The expression is (* 1 2 (+ 3 4))
        // Nodes: *, 1, 2, +, 3, 4
        // Literals: 1, 2, 3, 4 (4 literals)
        // Symbols: *, +

        // After adding the expression, we should have 6 nodes and 6 classes initially.
        // The literals 1, 2, 3, 4 should each be in their own class with count 1.
        // The symbols * and + should be in their own classes with count 0.

        // Find the class for literal 1
        let literal_1_id = egraph.node_id(&Node::Literal(Literal::Int(1))).unwrap();
        let class_1_id = egraph.containing_class(literal_1_id);
        assert_eq!(egraph.class(class_1_id).analysis().count(), 1);

        // Find the class for literal 2
        let literal_2_id = egraph.node_id(&Node::Literal(Literal::Int(2))).unwrap();
        let class_2_id = egraph.containing_class(literal_2_id);
        assert_eq!(egraph.class(class_2_id).analysis().count(), 1);

        // Find the class for literal 3
        let literal_3_id = egraph.node_id(&Node::Literal(Literal::Int(3))).unwrap();
        let class_3_id = egraph.containing_class(literal_3_id);
        assert_eq!(egraph.class(class_3_id).analysis().count(), 1);

        // Find the class for literal 4
        let literal_4_id = egraph.node_id(&Node::Literal(Literal::Int(4))).unwrap();
        let class_4_id = egraph.containing_class(literal_4_id);
        assert_eq!(egraph.class(class_4_id).analysis().count(), 1);

        // Find the class for symbol +
        let plus_id = egraph.find_symbols(lang.get_id("+"))[0];
        let plus_class_id = egraph.containing_class(plus_id);
        assert_eq!(egraph.class(plus_class_id).analysis().count(), 0);

        // Find the class for symbol *
        let mul_id = egraph.find_symbols(lang.get_id("*"))[0];
        let mul_class_id = egraph.containing_class(mul_id);
        assert_eq!(egraph.class(mul_class_id).analysis().count(), 0);

        // Merge classes (3 + 4) and (1 + 2)
        // This will merge the classes containing the literals 3 and 4, and 1 and 2.
        // The analysis should reflect the combined literal count.
        egraph.merge_classes(class_3_id, class_4_id);
        let merged_plus_class_id = egraph.containing_class(class_3_id);
        assert_eq!(egraph.class(merged_plus_class_id).analysis().count(), 2);

        egraph.merge_classes(class_1_id, class_2_id);
        let merged_mul_class_id = egraph.containing_class(class_1_id);
        assert_eq!(egraph.class(merged_mul_class_id).analysis().count(), 2);

        // Merge the class containing the '+' symbol with the class containing '1' and '2'
        // This should not change the literal count of the '+' class, as it only contains symbols.
        egraph.merge_classes(plus_class_id, merged_mul_class_id);
        let final_plus_class_id = egraph.containing_class(plus_class_id);
        assert_eq!(egraph.class(final_plus_class_id).analysis().count(), 2);
    }
}
