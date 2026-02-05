//! Induced string rewriting for term rewriting systems.
//!
//! This module implements "induced string rewriting" which converts
//! expressions and rewrite rules into a string language where all symbols
//! have arity 0 or 1.

use crate::language::{
    Language,
    expression::{Expression, OwnedPath, VariableId},
    symbol::{Symbol, SymbolId},
};
use crate::rewriting::rule::Rule;
use std::collections::HashMap;

/// Converts a language to its induced string language.
///
/// In the string language:
/// - 0-arity symbols remain unchanged
/// - 1-arity symbols remain unchanged
/// - n-arity symbols (n > 1) are converted to n different 1-arity symbols:
///   symbol `s` becomes `s_1`, `s_2`, ..., `s_n`
///
/// # Arguments
///
/// * `lang` - The original language to convert
/// * `topology` - A mapping from symbol IDs to their typical arity (number of children)
///
/// # Returns
///
/// Returns a new `Language` representing the string language
pub fn to_string_language(lang: &Language, topology: &HashMap<SymbolId, usize>) -> Language {
    let mut string_lang = Language::default();
    
    for symbol_id in 0..lang.symbol_count() {
        let symbol_name = lang.get_symbol(symbol_id);
        let arity = topology.get(&symbol_id).copied().unwrap_or(0);
        
        match arity {
            0 | 1 => {
                // Keep 0-arity and 1-arity symbols as-is
                string_lang = string_lang.add_symbol(symbol_name);
            }
            n => {
                // For n-arity symbols (n > 1), create n different 1-arity symbols
                for i in 1..=n {
                    let indexed_name = format!("{}_{}", symbol_name, i);
                    string_lang = string_lang.add_symbol(&indexed_name);
                }
            }
        }
    }
    
    string_lang
}

/// Extracts all paths from the root to each leaf in an expression.
///
/// Each path is represented as a sequence of indexed symbols in the string language.
/// When a path goes through the i-th child of a symbol, that symbol is represented
/// as its indexed version in the string language.
///
/// # Arguments
///
/// * `expr` - The expression to extract paths from
/// * `lang` - The original language
/// * `string_lang` - The induced string language
/// * `topology` - A mapping from symbol IDs to their arity
///
/// # Returns
///
/// Returns a vector of expressions, each representing a path from root to a leaf
pub fn expression_to_paths(
    expr: &Expression,
    lang: &Language,
    string_lang: &Language,
    topology: &HashMap<SymbolId, usize>,
) -> Vec<Expression> {
    let mut paths = Vec::new();
    expression_to_paths_impl(expr, lang, string_lang, topology, &mut Vec::new(), &mut paths);
    paths
}

fn expression_to_paths_impl(
    expr: &Expression,
    lang: &Language,
    string_lang: &Language,
    topology: &HashMap<SymbolId, usize>,
    current_path: &mut Vec<Expression>,
    paths: &mut Vec<Expression>,
) {
    match expr {
        Expression::Literal(lit) => {
            // We've reached a leaf, add the complete path
            let mut path = current_path.clone();
            path.push(Expression::Literal(lit.clone()));
            
            // Convert the path into a nested expression
            if let Some(path_expr) = build_path_expression(&path) {
                paths.push(path_expr);
            }
        }
        Expression::Variable(var_id) => {
            // Variables are also leaves
            let mut path = current_path.clone();
            path.push(Expression::Variable(*var_id));
            
            if let Some(path_expr) = build_path_expression(&path) {
                paths.push(path_expr);
            }
        }
        Expression::Symbol(symbol) => {
            let arity = topology.get(&symbol.id).copied().unwrap_or(symbol.children.len());
            
            if symbol.children.is_empty() {
                // 0-arity symbol is a leaf
                let mut path = current_path.clone();
                path.push(Expression::Symbol(Symbol {
                    id: symbol.id,
                    children: vec![],
                }));
                
                if let Some(path_expr) = build_path_expression(&path) {
                    paths.push(path_expr);
                }
            } else {
                // Process each child
                for (child_idx, child) in symbol.children.iter().enumerate() {
                    // Get the indexed symbol name for this child position
                    let indexed_symbol = get_indexed_symbol(
                        symbol.id,
                        child_idx,
                        lang,
                        string_lang,
                        arity,
                    );
                    
                    current_path.push(indexed_symbol);
                    expression_to_paths_impl(child, lang, string_lang, topology, current_path, paths);
                    current_path.pop();
                }
            }
        }
    }
}

/// Builds a nested expression from a path (list of expressions).
///
/// For example, [a, b, c] becomes (a (b c))
fn build_path_expression(path: &[Expression]) -> Option<Expression> {
    if path.is_empty() {
        return None;
    }
    
    if path.len() == 1 {
        return Some(path[0].clone());
    }
    
    // Build from the end backwards
    let mut result = path[path.len() - 1].clone();
    
    for i in (0..path.len() - 1).rev() {
        let symbol = path[i].clone();
        
        match symbol {
            Expression::Symbol(mut sym) => {
                sym.children = vec![result];
                result = Expression::Symbol(sym);
            }
            _ => {
                // If it's not a symbol, we can't build a proper path
                // This shouldn't happen in normal usage
                return None;
            }
        }
    }
    
    Some(result)
}

/// Gets the indexed symbol expression for a given child position.
fn get_indexed_symbol(
    symbol_id: SymbolId,
    child_idx: usize,
    lang: &Language,
    string_lang: &Language,
    arity: usize,
) -> Expression {
    let symbol_name = lang.get_symbol(symbol_id);
    
    let string_symbol_name = if arity <= 1 {
        // 0-arity or 1-arity symbols stay the same
        symbol_name.to_string()
    } else {
        // n-arity symbols become indexed
        format!("{}_{}", symbol_name, child_idx + 1)
    };
    
    let string_symbol_id = string_lang.get_id(&string_symbol_name);
    
    Expression::Symbol(Symbol {
        id: string_symbol_id,
        children: vec![],
    })
}

/// Converts a rewriting rule into induced string rewriting rules.
///
/// For each variable x in the left-hand side, for each occurrence of x in the
/// left-hand side, and for each occurrence of x in the right-hand side,
/// generates a rule rewriting the path from root to the left occurrence
/// into the path from root to the right occurrence.
///
/// # Arguments
///
/// * `rule` - The original rewriting rule
/// * `lang` - The original language
/// * `string_lang` - The induced string language
/// * `topology` - A mapping from symbol IDs to their arity
///
/// # Returns
///
/// Returns a vector of induced rules in the string language
pub fn rule_to_induced_rules(
    rule: &Rule,
    lang: &Language,
    string_lang: &Language,
    topology: &HashMap<SymbolId, usize>,
) -> Vec<Rule> {
    let mut induced_rules = Vec::new();
    
    // Find all variable occurrences in left and right sides
    let left_vars = rule.from().find_all_variables();
    let right_vars = rule.to().find_all_variables();
    
    // For each variable that appears in both sides
    for (var_id, left_paths) in &left_vars {
        if let Some(right_paths) = right_vars.get(var_id) {
            // For each occurrence in left, and each occurrence in right
            for left_path in left_paths {
                for right_path in right_paths {
                    // Generate the path expressions
                    let left_path_expr = path_to_expression(
                        rule.from(),
                        left_path,
                        lang,
                        string_lang,
                        topology,
                        *var_id,
                    );
                    
                    let right_path_expr = path_to_expression(
                        rule.to(),
                        right_path,
                        lang,
                        string_lang,
                        topology,
                        *var_id,
                    );
                    
                    if let (Some(left_expr), Some(right_expr)) = (left_path_expr, right_path_expr) {
                        induced_rules.push(Rule::from_expressions(left_expr, right_expr));
                    }
                }
            }
        }
    }
    
    induced_rules
}

/// Constructs a path expression from the root to a specific variable occurrence.
fn path_to_expression(
    expr: &Expression,
    target_path: &OwnedPath,
    lang: &Language,
    string_lang: &Language,
    topology: &HashMap<SymbolId, usize>,
    var_id: VariableId,
) -> Option<Expression> {
    let mut path_elements = Vec::new();
    path_to_expression_impl(
        expr,
        target_path,
        &mut vec![],
        lang,
        string_lang,
        topology,
        &mut path_elements,
    );
    
    // Add the variable at the end
    path_elements.push(Expression::Variable(var_id));
    
    build_path_expression(&path_elements)
}

fn path_to_expression_impl(
    expr: &Expression,
    target_path: &OwnedPath,
    current_path: &mut Vec<usize>,
    lang: &Language,
    string_lang: &Language,
    topology: &HashMap<SymbolId, usize>,
    path_elements: &mut Vec<Expression>,
) {
    // Check if we've reached the target
    if current_path.len() == target_path.0.len() {
        return;
    }
    
    if let Expression::Symbol(symbol) = expr {
        let next_idx = target_path.0[current_path.len()];
        let arity = topology.get(&symbol.id).copied().unwrap_or(symbol.children.len());
        
        // Add the indexed symbol for this step
        let indexed_symbol = get_indexed_symbol(
            symbol.id,
            next_idx,
            lang,
            string_lang,
            arity,
        );
        path_elements.push(indexed_symbol);
        
        // Continue to the next child
        if next_idx < symbol.children.len() {
            current_path.push(next_idx);
            path_to_expression_impl(
                &symbol.children[next_idx],
                target_path,
                current_path,
                lang,
                string_lang,
                topology,
                path_elements,
            );
            current_path.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;

    #[test]
    fn test_to_string_language_simple() {
        let lang = Language::default()
            .add_symbol("+")    // id: 0
            .add_symbol("sin"); // id: 1
        
        let mut topology = HashMap::new();
        topology.insert(0, 2); // + has arity 2
        topology.insert(1, 1); // sin has arity 1
        
        let string_lang = to_string_language(&lang, &topology);
        
        // + should become +_1 and +_2
        assert_eq!(string_lang.get_symbol(0), "+_1");
        assert_eq!(string_lang.get_symbol(1), "+_2");
        // sin should stay the same
        assert_eq!(string_lang.get_symbol(2), "sin");
    }

    #[test]
    fn test_to_string_language_zero_arity() {
        let lang = Language::default()
            .add_symbol("x")    // id: 0, will be 0-arity
            .add_symbol("*");   // id: 1
        
        let mut topology = HashMap::new();
        topology.insert(0, 0); // x has arity 0
        topology.insert(1, 2); // * has arity 2
        
        let string_lang = to_string_language(&lang, &topology);
        
        // x should stay the same
        assert_eq!(string_lang.get_symbol(0), "x");
        // * should become *_1 and *_2
        assert_eq!(string_lang.get_symbol(1), "*_1");
        assert_eq!(string_lang.get_symbol(2), "*_2");
    }

    #[test]
    fn test_expression_to_paths_simple() {
        let lang = Language::default()
            .add_symbol("+")
            .add_symbol("sin");
        
        let mut topology = HashMap::new();
        topology.insert(0, 2);
        topology.insert(1, 1);
        
        let string_lang = to_string_language(&lang, &topology);
        
        // Expression: (+ (sin 1) 2)
        let expr = lang.parse("(+ (sin 1) 2)").unwrap();
        let paths = expression_to_paths(&expr, &lang, &string_lang, &topology);
        
        // Should have 2 paths: one to 1, one to 2
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_expression_to_paths_with_variable() {
        let lang = Language::default()
            .add_symbol("+")
            .add_symbol("*");
        
        let mut topology = HashMap::new();
        topology.insert(0, 2);
        topology.insert(1, 2);
        
        let string_lang = to_string_language(&lang, &topology);
        
        // Expression: (+ $0 (* 2 3))
        let expr = lang.parse("(+ $0 (* 2 3))").unwrap();
        let paths = expression_to_paths(&expr, &lang, &string_lang, &topology);
        
        // Should have 3 paths: one to $0, one to 2, one to 3
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_rule_to_induced_rules_simple() {
        let lang = Language::default()
            .add_symbol("+")
            .add_symbol("*");
        
        let mut topology = HashMap::new();
        topology.insert(0, 2); // + has arity 2
        topology.insert(1, 2); // * has arity 2
        
        let string_lang = to_string_language(&lang, &topology);
        
        // Rule: (+ $0 $1) -> (* $0 $1)
        let rule = Rule::from_strings("(+ $0 $1)", "(* $0 $1)", &lang);
        let induced_rules = rule_to_induced_rules(&rule, &lang, &string_lang, &topology);
        
        // Should have 2 induced rules (one for each variable)
        // $0: +_1 $0 -> *_1 $0
        // $1: +_2 $1 -> *_2 $1
        assert_eq!(induced_rules.len(), 2);
    }

    #[test]
    fn test_rule_to_induced_rules_repeated_variable() {
        let lang = Language::default()
            .add_symbol("+")
            .add_symbol("*");
        
        let mut topology = HashMap::new();
        topology.insert(0, 2); // + has arity 2
        topology.insert(1, 2); // * has arity 2
        
        let string_lang = to_string_language(&lang, &topology);
        
        // Rule: (+ $0 $0) -> (* $0 $0)
        // Variable $0 appears twice on each side
        let rule = Rule::from_strings("(+ $0 $0)", "(* $0 $0)", &lang);
        let induced_rules = rule_to_induced_rules(&rule, &lang, &string_lang, &topology);
        
        // Should have 4 induced rules (2 left occurrences × 2 right occurrences)
        assert_eq!(induced_rules.len(), 4);
    }

    #[test]
    fn test_to_string_language_ternary() {
        let lang = Language::default()
            .add_symbol("if"); // id: 0, ternary operator
        
        let mut topology = HashMap::new();
        topology.insert(0, 3); // if has arity 3
        
        let string_lang = to_string_language(&lang, &topology);
        
        // if should become if_1, if_2, and if_3
        assert_eq!(string_lang.get_symbol(0), "if_1");
        assert_eq!(string_lang.get_symbol(1), "if_2");
        assert_eq!(string_lang.get_symbol(2), "if_3");
        assert_eq!(string_lang.symbol_count(), 3);
    }

    #[test]
    fn test_expression_to_paths_detailed() {
        let lang = Language::default()
            .add_symbol("+")    // id: 0
            .add_symbol("sin"); // id: 1
        
        let mut topology = HashMap::new();
        topology.insert(0, 2);
        topology.insert(1, 1);
        
        let string_lang = to_string_language(&lang, &topology);
        
        // Expression: (+ (sin 1) 2)
        // Path 1: +_1 -> sin -> 1
        // Path 2: +_2 -> 2
        let expr = lang.parse("(+ (sin 1) 2)").unwrap();
        let paths = expression_to_paths(&expr, &lang, &string_lang, &topology);
        
        assert_eq!(paths.len(), 2);
        
        // Verify the first path represents: (+_1 (sin 1))
        // It should be a nested expression
        let path1 = &paths[0];
        let path2 = &paths[1];
        
        // Both should be Symbol expressions
        assert!(matches!(path1, Expression::Symbol(_)));
        assert!(matches!(path2, Expression::Symbol(_)));
    }

    #[test]
    fn test_rule_to_induced_rules_detailed() {
        let lang = Language::default()
            .add_symbol("+")    // id: 0
            .add_symbol("*");   // id: 1
        
        let mut topology = HashMap::new();
        topology.insert(0, 2);
        topology.insert(1, 2);
        
        let string_lang = to_string_language(&lang, &topology);
        
        // Rule: (+ $0 $1) -> (+ $1 $0)  (commutativity)
        let rule = Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang);
        let induced_rules = rule_to_induced_rules(&rule, &lang, &string_lang, &topology);
        
        // $0 appears at position 0 in left, and position 1 in right
        // $1 appears at position 1 in left, and position 0 in right
        // So we get 2 induced rules:
        // 1. +_1 $0 -> +_2 $0  (left pos 0 -> right pos 1)
        // 2. +_2 $1 -> +_1 $1  (left pos 1 -> right pos 0)
        assert_eq!(induced_rules.len(), 2);
        
        // Check that all induced rules have variables
        for induced_rule in &induced_rules {
            assert!(induced_rule.from().variables().len() > 0);
            assert!(induced_rule.to().variables().len() > 0);
        }
    }
}
