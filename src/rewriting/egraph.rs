use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::{
    language::{
        expression::{Literal, VarFreeExpression},
        symbol::Symbol,
    },
    union_find::UnionFind,
};

pub type NodeId = usize;
pub type ClassId = usize;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    Literal(Literal),
    Symbol(Symbol<ClassId>),
}

impl Node {
    fn iter_mut_children(&mut self) -> std::slice::IterMut<ClassId> {
        match self {
            Node::Literal(_) => std::slice::IterMut::default(),
            Node::Symbol(symbol) => symbol.children.iter_mut(),
        }
    }

    fn iter_children(&self) -> std::slice::Iter<ClassId> {
        match self {
            Node::Literal(_) => std::slice::Iter::default(),
            Node::Symbol(symbol) => symbol.children.iter(),
        }
    }

    fn canonical(&self, graph: &EGraph) -> Self {
        let mut cloned = self.clone();
        cloned.make_canonical(graph);
        cloned
    }

    fn make_canonical(&mut self, graph: &EGraph) {
        for child_id in self.iter_mut_children() {
            *child_id = graph.canonical_class(*child_id);
        }
    }

    /// Assume `self` is a symbol and return it, otherwise panic.
    fn as_symbol(&self) -> &Symbol<ClassId> {
        match self {
            Node::Literal(literal) => panic!("Expected node to be a symbol, is {:?}", literal),
            Node::Symbol(symbol) => symbol,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Class {
    nodes_ids: HashSet<NodeId>,
    parents_ids: HashSet<NodeId>,
}

impl Class {
    fn from_node(node_id: NodeId) -> Self {
        Self {
            nodes_ids: HashSet::from([node_id]),
            parents_ids: HashSet::new(),
        }
    }

    fn merge(&mut self, other: Self) {
        self.nodes_ids.extend(other.nodes_ids);
        self.parents_ids.extend(other.parents_ids)
    }
}

// TODO: matching (bottom up for now?)
// TODO: rule application
#[derive(Default)]
struct EGraph {
    union_find: UnionFind,
    nodes: HashMap<NodeId, Node>,
    // Always kept behind canonical IDs
    classes: HashMap<ClassId, Class>,
}

impl EGraph {
    fn from_expression(expression: VarFreeExpression) -> Self {
        let mut egraph = Self::default();

        egraph.add_expression(expression);

        egraph
    }

    pub fn add_expression(&mut self, expression: VarFreeExpression) -> NodeId {
        let node = match expression {
            VarFreeExpression::Literal(literal) => Node::Literal(literal),
            VarFreeExpression::Symbol(symbol) => {
                let mut child_ids = Vec::new();
                for child in symbol.children {
                    child_ids.push(self.add_expression(child));
                }

                Node::Symbol(Symbol {
                    id: symbol.id,
                    children: child_ids,
                })
            }
        };

        self.add_node(node)
    }

    fn add_node(&mut self, node: Node) -> NodeId {
        if let Some(&old_id) = self.node_id(&node).as_ref() {
            return old_id;
        }

        let class_id = self.union_find.add();
        let node_id = class_id;

        if let Node::Symbol(symbol) = &node {
            for child in &symbol.children {
                self.add_parent(*child, node_id);
            }
        }

        self.nodes.insert(node_id, node);

        let class = Class::from_node(node_id);
        self.classes.insert(class_id, class);
        node_id
    }

    /// If a given node exists in the e-graph, returns it. Otherwise gives `None`.
    fn node_id(&self, node: &Node) -> Option<NodeId> {
        // TODO: Add hashcons so that this is speedy
        // Are hashcons really that much better when they hold uncanonical, potentially outdated keys anyway...?
        let node = node.canonical(self);

        for (&candidate_id, candidate_node) in self.nodes.iter() {
            let candidate_node = candidate_node.canonical(self);
            if candidate_node == node {
                return Some(candidate_id);
            }
        }

        None
    }

    fn add_parent(&mut self, class_id: ClassId, parent_id: NodeId) {
        self.class_mut(class_id).parents_ids.insert(parent_id);
    }

    fn canonical_class(&self, class_id: ClassId) -> ClassId {
        self.union_find.find(class_id)
    }

    pub fn class(&self, class_id: ClassId) -> &Class {
        let class_id = self.canonical_class(class_id);
        &self.classes[&class_id]
    }

    pub fn class_mut(&mut self, class_id: ClassId) -> &mut Class {
        let class_id = self.canonical_class(class_id);
        self.classes.get_mut(&class_id).unwrap()
    }

    pub fn node(&self, node_id: NodeId) -> &Node {
        &self.nodes[&node_id]
    }

    pub fn node_mut(&mut self, node_id: NodeId) -> &mut Node {
        self.nodes.get_mut(&node_id).unwrap()
    }

    pub fn class_count(&self) -> usize {
        self.classes.len()
    }

    /// Returns the total number of nodes which were added to the egraph at any point
    pub fn total_node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the current number of nodes in the egraph
    pub fn actual_node_count(&self) -> usize {
        self.classes
            .iter()
            .map(|(_, class)| class.nodes_ids.len())
            .sum()
    }

    pub fn parents(&self, class_id: ClassId) -> &HashSet<NodeId> {
        &self.class(class_id).parents_ids
    }

    pub fn nodes(&self, class_id: ClassId) -> &HashSet<NodeId> {
        &self.class(class_id).nodes_ids
    }

    pub fn containing_class(&self, node_id: NodeId) -> ClassId {
        // On creation of a node, its containing class has the same id.
        // The canonical ID of the class might change, but UF will give it to us.
        self.union_find.find(node_id)
    }

    pub fn merge_classes(&mut self, class_1_id: ClassId, class_2_id: ClassId) {
        let class_1_id = self.union_find.find(class_1_id);
        let class_2_id = self.union_find.find(class_2_id);

        self.union_find.union(class_1_id, class_2_id);

        let class_1 = self.classes.remove(&class_1_id).unwrap();
        self.classes.get_mut(&class_2_id).unwrap().merge(class_1);

        self.rebuild_class(class_2_id);
    }

    /// Makes all nodes in an eclass have canonical children
    fn make_class_canonical(&mut self, class_id: ClassId) {
        let class_id = self.canonical_class(class_id);
        let class = &self.classes[&class_id];
        for &node_id in &class.nodes_ids {
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
            .nodes_ids
            .iter()
            .cartesian_product(class.nodes_ids.iter())
            .filter(|(id_1, id_2)| id_1 < id_2)
        {
            if self.node(node_id_1) == self.node(node_id_2) {
                to_remove.push(node_id_1);
            }
        }

        // Remove duplicate nodes
        let class = self.classes.get_mut(&class_id).unwrap();
        for node_id in to_remove {
            class.nodes_ids.remove(&node_id);
        }

        // Make all nodes in parents have canonical class IDs
        let class = &self.classes[&class_id];
        for parent in &class.parents_ids {
            for child in self.nodes.get_mut(&parent).unwrap().iter_mut_children() {
                *child = self.union_find.find(*child);
            }
        }

        // Check if there are parents who share identical nodes
        let mut to_merge = Vec::new();
        for (parent_1_id, parent_2_id) in class
            .parents_ids
            .iter()
            .cartesian_product(class.parents_ids.iter())
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
        rewriting::egraph::Node,
    };

    use super::EGraph;

    #[test]
    fn from_expression() {
        let lang = Language::math();
        let expression: VarFreeExpression = lang.parse_no_vars("(* 1 2 3 (+ 4 5))").unwrap();

        let graph = EGraph::from_expression(expression);

        assert_eq!(graph.class_count(), 7);
        assert_eq!(graph.total_node_count(), 7);
        assert_eq!(graph.actual_node_count(), 7);
    }

    #[test]
    fn merge() {
        let lang = Language::math();
        let expression_1: VarFreeExpression = lang.parse_no_vars("(+ 1 5)").unwrap();
        let expression_2: VarFreeExpression = lang.parse_no_vars("(+ 2 4)").unwrap();

        let mut graph = EGraph::default();

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
        let mut egraph = EGraph::default();

        let nodes = (0..10)
            .map(|i| Node::Literal(Literal::UInt(i)))
            .collect_vec();
        let node_ids = nodes
            .iter()
            .cloned()
            .map(|node| egraph.add_node(node))
            .collect_vec();

        for i in 0..10 {
            assert_eq!(*egraph.node(node_ids[i]), nodes[i]);
            assert_eq!(egraph.node_id(&nodes[i]).unwrap(), node_ids[i]);
        }
    }

    #[test]
    fn parents() {
        let lang = Language::math();
        let expression: VarFreeExpression = lang.parse_no_vars("(+ 1 5)").unwrap();

        let mut graph = EGraph::default();

        let node_id = graph.add_expression(expression);
        let class_id = graph.containing_class(node_id);

        assert!(graph.parents(class_id).is_empty());
        assert_eq!(graph.nodes(class_id).len(), 1);
        {}
        let Node::Symbol(plus) = graph.node(*graph.nodes(class_id).iter().next().unwrap()) else {
            panic!("Unxepected variant")
        };

        for child_id in &plus.children {
            let parent_ids = &graph.class(*child_id).parents_ids;
            assert_eq!(parent_ids.len(), 1);
            assert_eq!(*parent_ids.iter().next().unwrap(), node_id);
        }
    }

    #[test]
    fn double_node_addition() {
        let mut egraph = EGraph::default();

        let literal_1 = Node::Literal(Literal::UInt(5));
        let literal_2 = Node::Literal(Literal::UInt(7));

        let node_1_id = egraph.add_node(literal_1.clone());
        let node_1_id_ = egraph.add_node(literal_1);
        assert_eq!(node_1_id, node_1_id_);

        let node_2_id = egraph.add_node(literal_2);

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
        assert_eq!(node_3_id, node_4_id);
    }

    #[test]
    fn literal_sharing() {
        let mut egraph = EGraph::default();

        let lang = Language::math();
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
        let mut egraph = EGraph::default();

        let lang = Language::math();
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
        let lang = Language::math();

        let expr = lang
            .parse_no_vars("(* (+ 5 (sin (* 1 7))) (+ 5 (sin (* 1 8))))")
            .unwrap();

        let mut egraph = EGraph::from_expression(expr);

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
}
