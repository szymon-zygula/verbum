//! Induced string rewriting for term rewriting systems.
//!
//! This module implements "induced string rewriting" which converts
//! expressions and rewrite rules into a string language where all symbols
//! have arity 0 or 1.

use crate::language::{
    Language,
    arities::Arities,
    expression::{Expression, OwnedPath, VariableId},
    symbol::{Symbol, SymbolId},
};
use crate::rewriting::rule::Rule;
use nalgebra::{DMatrix, DVector};

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
/// * `arities` - A mapping from symbol IDs to their typical arity (number of children)
///
/// # Returns
///
/// Returns a new `Language` representing the string language
pub fn to_string_language(lang: &Language, arities: &Arities) -> Language {
    let mut string_lang = Language::default();

    for symbol_id in 0..lang.symbol_count() {
        let symbol_name = lang.get_symbol(symbol_id);
        let arity = arities.get_first(symbol_id).unwrap_or(0);

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
/// * `arities` - A mapping from symbol IDs to their arity
///
/// # Returns
///
/// Returns a vector of expressions, each representing a path from root to a leaf
pub fn expression_to_paths(
    expr: &Expression,
    lang: &Language,
    string_lang: &Language,
    arities: &Arities,
) -> Vec<Expression> {
    let mut paths = Vec::new();
    expression_to_paths_impl(
        expr,
        lang,
        string_lang,
        arities,
        &mut Vec::new(),
        &mut paths,
    );
    paths
}

fn expression_to_paths_impl(
    expr: &Expression,
    lang: &Language,
    string_lang: &Language,
    arities: &Arities,
    current_path: &mut Vec<Expression>,
    paths: &mut Vec<Expression>,
) {
    match expr {
        Expression::Literal(lit) => {
            handle_literal_path(lit, current_path.clone(), paths);
        }
        Expression::Variable(var_id) => {
            handle_variable_path(*var_id, current_path.clone(), paths);
        }
        Expression::Symbol(symbol) => {
            handle_symbol_path(symbol, lang, string_lang, arities, current_path, paths);
        }
    }
}

/// Handles path building for literal expressions
fn handle_literal_path(
    lit: &crate::language::expression::Literal,
    mut current_path: Vec<Expression>,
    paths: &mut Vec<Expression>,
) {
    // We've reached a leaf, add the complete path
    current_path.push(Expression::Literal(lit.clone()));

    // Convert the path into a nested expression
    if let Some(path_expr) = build_path_expression(&current_path) {
        paths.push(path_expr);
    }
}

/// Handles path building for variable expressions
fn handle_variable_path(
    var_id: VariableId,
    mut current_path: Vec<Expression>,
    paths: &mut Vec<Expression>,
) {
    // Variables are also leaves
    current_path.push(Expression::Variable(var_id));

    if let Some(path_expr) = build_path_expression(&current_path) {
        paths.push(path_expr);
    }
}

/// Handles path building for symbol expressions
fn handle_symbol_path(
    symbol: &Symbol<Expression>,
    lang: &Language,
    string_lang: &Language,
    arities: &Arities,
    current_path: &mut Vec<Expression>,
    paths: &mut Vec<Expression>,
) {
    let arity = arities
        .get_first(symbol.id)
        .unwrap_or(symbol.children.len());

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
            let indexed_symbol_id =
                to_string_symbol(symbol.id, child_idx, lang, string_lang, arity);

            current_path.push(Expression::Symbol(Symbol {
                id: indexed_symbol_id,
                children: vec![],
            }));
            expression_to_paths_impl(child, lang, string_lang, arities, current_path, paths);
            current_path.pop();
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

    for symbol in path.iter().rev().skip(1).cloned() {
        let Expression::Symbol(mut sym) = symbol else {
            // If it's not a symbol, we can't build a proper path
            // This shouldn't happen in normal usage
            return None;
        };

        sym.children = vec![result];
        result = Expression::Symbol(sym);
    }

    Some(result)
}

/// Converts a symbol ID to its corresponding string language symbol ID
fn to_string_symbol(
    symbol_id: SymbolId,
    child_idx: usize,
    lang: &Language,
    string_lang: &Language,
    arity: usize,
) -> SymbolId {
    let symbol_name = lang.get_symbol(symbol_id);

    let string_symbol_name = if arity <= 1 {
        // 0-arity or 1-arity symbols stay the same
        symbol_name.to_string()
    } else {
        // n-arity symbols become indexed
        format!("{}_{}", symbol_name, child_idx + 1)
    };

    string_lang.get_id(&string_symbol_name)
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
/// * `arities` - A mapping from symbol IDs to their arity
///
/// # Returns
///
/// Returns a vector of induced rules in the string language
pub fn rule_to_induced_rules(
    rule: &Rule,
    lang: &Language,
    string_lang: &Language,
    arities: &Arities,
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
                        arities,
                        *var_id,
                    );

                    let right_path_expr = path_to_expression(
                        rule.to(),
                        right_path,
                        lang,
                        string_lang,
                        arities,
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
    arities: &Arities,
    var_id: VariableId,
) -> Option<Expression> {
    // Walk down the expression following the path
    let mut current_expr = expr;
    let mut path_elements: Vec<Expression> = Vec::new();

    for &child_idx in &target_path.0 {
        let Expression::Symbol(symbol) = current_expr else {
            // Path goes through a non-symbol, which shouldn't happen
            return None;
        };

        let arity = arities
            .get_first(symbol.id)
            .unwrap_or(symbol.children.len());

        // Convert symbol to its string language version
        let indexed_symbol_id = to_string_symbol(symbol.id, child_idx, lang, string_lang, arity);

        path_elements.push(Expression::Symbol(Symbol {
            id: indexed_symbol_id,
            children: vec![],
        }));

        // Move to the next level
        current_expr = symbol.children.get(child_idx)?;
    }

    // Add the variable at the end
    path_elements.push(Expression::Variable(var_id));

    build_path_expression(&path_elements)
}

/// Converts an expression to its abelianized vector.
///
/// The abelianized vector has dimension equal to the number of symbols in the language.
/// Each coordinate represents the count of how many times that symbol appears in the expression.
///
/// # Arguments
///
/// * `expr` - The expression to abelianize
/// * `lang` - The language containing the symbols
///
/// # Returns
///
/// Returns a `DVector` where the i-th entry is the count of symbol i in the expression
pub fn expression_to_abelian_vector(expr: &Expression, lang: &Language) -> DVector<i32> {
    let symbol_count = lang.symbol_count();
    let mut counts = vec![0i32; symbol_count];

    count_symbols_impl(expr, &mut counts);

    DVector::from_vec(counts)
}

/// Helper function to recursively count symbols in an expression
fn count_symbols_impl(expr: &Expression, counts: &mut [i32]) {
    match expr {
        Expression::Literal(_) | Expression::Variable(_) => {
            // Literals and variables don't contribute to symbol counts
        }
        Expression::Symbol(symbol) => {
            // Increment count for this symbol
            counts[symbol.id] += 1;

            // Recursively count symbols in children
            for child in &symbol.children {
                count_symbols_impl(child, counts);
            }
        }
    }
}

/// Creates an abelianized TRS matrix from a set of rewrite rules.
///
/// The matrix A has:
/// - Number of rows = number of symbols in the language
/// - Number of columns = number of rules
/// - A[i,j] = r[i,j] - l[i,j]
///   where l[i,j] and r[i,j] are the counts of symbol i in the left and right
///   sides of rule j, respectively
///
/// # Arguments
///
/// * `rules` - The rewrite rules to convert to a matrix
/// * `lang` - The language containing the symbols
///
/// # Returns
///
/// Returns a `DMatrix` representing the abelianized TRS
pub fn rules_to_abelian_matrix(rules: &[Rule], lang: &Language) -> DMatrix<i32> {
    let symbol_count = lang.symbol_count();
    let rule_count = rules.len();

    // Create matrix with dimensions: symbols x rules
    // nalgebra DMatrix::from_vec expects column-major order:
    // element (row, col) is at index col * nrows + row
    let mut matrix_data = vec![0i32; symbol_count * rule_count];

    for (rule_idx, rule) in rules.iter().enumerate() {
        let left_vec = expression_to_abelian_vector(rule.from(), lang);
        let right_vec = expression_to_abelian_vector(rule.to(), lang);
        let diff_vec = right_vec - left_vec;

        for symbol_idx in 0..symbol_count {
            matrix_data[rule_idx * symbol_count + symbol_idx] = diff_vec[symbol_idx];
        }
    }

    DMatrix::from_vec(symbol_count, rule_count, matrix_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use std::collections::HashMap;

    #[test]
    fn test_to_string_language_simple() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("sin"); // id: 1

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2); // + has arity 2
        arities_map.insert(1, 1); // sin has arity 1
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // + should become +_1 and +_2
        assert_eq!(string_lang.get_symbol(0), "+_1");
        assert_eq!(string_lang.get_symbol(1), "+_2");
        // sin should stay the same
        assert_eq!(string_lang.get_symbol(2), "sin");
    }

    #[test]
    fn test_to_string_language_zero_arity() {
        let lang = Language::default()
            .add_symbol("x") // id: 0, will be 0-arity
            .add_symbol("*"); // id: 1

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 0); // x has arity 0
        arities_map.insert(1, 2); // * has arity 2
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // x should stay the same
        assert_eq!(string_lang.get_symbol(0), "x");
        // * should become *_1 and *_2
        assert_eq!(string_lang.get_symbol(1), "*_1");
        assert_eq!(string_lang.get_symbol(2), "*_2");
    }

    #[test]
    fn test_expression_to_paths_simple() {
        let lang = Language::default().add_symbol("+").add_symbol("sin");

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2);
        arities_map.insert(1, 1);
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // Expression: (+ (sin 1) 2)
        let expr = lang.parse("(+ (sin 1) 2)").unwrap();
        let paths = expression_to_paths(&expr, &lang, &string_lang, &arities);

        // Should have 2 paths: one to 1, one to 2
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_expression_to_paths_with_variable() {
        let lang = Language::default().add_symbol("+").add_symbol("*");

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2);
        arities_map.insert(1, 2);
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // Expression: (+ $0 (* 2 3))
        let expr = lang.parse("(+ $0 (* 2 3))").unwrap();
        let paths = expression_to_paths(&expr, &lang, &string_lang, &arities);

        // Should have 3 paths: one to $0, one to 2, one to 3
        assert_eq!(paths.len(), 3);
    }

    #[test]
    fn test_rule_to_induced_rules_simple() {
        let lang = Language::default().add_symbol("+").add_symbol("*");

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2); // + has arity 2
        arities_map.insert(1, 2); // * has arity 2
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // Rule: (+ $0 $1) -> (* $0 $1)
        let rule = Rule::from_strings("(+ $0 $1)", "(* $0 $1)", &lang);
        let induced_rules = rule_to_induced_rules(&rule, &lang, &string_lang, &arities);

        // Should have 2 induced rules (one for each variable)
        // $0: +_1 $0 -> *_1 $0
        // $1: +_2 $1 -> *_2 $1
        assert_eq!(induced_rules.len(), 2);
    }

    #[test]
    fn test_rule_to_induced_rules_repeated_variable() {
        let lang = Language::default().add_symbol("+").add_symbol("*");

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2); // + has arity 2
        arities_map.insert(1, 2); // * has arity 2
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // Rule: (+ $0 $0) -> (* $0 $0)
        // Variable $0 appears twice on each side
        let rule = Rule::from_strings("(+ $0 $0)", "(* $0 $0)", &lang);
        let induced_rules = rule_to_induced_rules(&rule, &lang, &string_lang, &arities);

        // Should have 4 induced rules (2 left occurrences × 2 right occurrences)
        assert_eq!(induced_rules.len(), 4);
    }

    #[test]
    fn test_to_string_language_ternary() {
        let lang = Language::default().add_symbol("if"); // id: 0, ternary operator

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 3); // if has arity 3
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // if should become if_1, if_2, and if_3
        assert_eq!(string_lang.get_symbol(0), "if_1");
        assert_eq!(string_lang.get_symbol(1), "if_2");
        assert_eq!(string_lang.get_symbol(2), "if_3");
        assert_eq!(string_lang.symbol_count(), 3);
    }

    #[test]
    fn test_expression_to_paths_detailed() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("sin"); // id: 1

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2);
        arities_map.insert(1, 1);
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // Expression: (+ (sin 1) 2)
        // Path 1: +_1 -> sin -> 1
        // Path 2: +_2 -> 2
        let expr = lang.parse("(+ (sin 1) 2)").unwrap();
        let paths = expression_to_paths(&expr, &lang, &string_lang, &arities);

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
            .add_symbol("+") // id: 0
            .add_symbol("*"); // id: 1

        let mut arities_map = HashMap::new();
        arities_map.insert(0, 2);
        arities_map.insert(1, 2);
        let arities = Arities::from(arities_map);

        let string_lang = to_string_language(&lang, &arities);

        // Rule: (+ $0 $1) -> (+ $1 $0)  (commutativity)
        let rule = Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang);
        let induced_rules = rule_to_induced_rules(&rule, &lang, &string_lang, &arities);

        // $0 appears at position 0 in left, and position 1 in right
        // $1 appears at position 1 in left, and position 0 in right
        // So we get 2 induced rules:
        // 1. +_1 $0 -> +_2 $0  (left pos 0 -> right pos 1)
        // 2. +_2 $1 -> +_1 $1  (left pos 1 -> right pos 0)
        assert_eq!(induced_rules.len(), 2);

        // Check that all induced rules have variables
        for induced_rule in &induced_rules {
            assert!(!induced_rule.from().variables().is_empty());
            assert!(!induced_rule.to().variables().is_empty());
        }
    }

    #[test]
    fn test_expression_to_abelian_vector_simple() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("*") // id: 1
            .add_symbol("sin"); // id: 2

        // Expression: (+ 1 2) - only has one + symbol
        let expr = lang.parse("(+ 1 2)").unwrap();
        let vec = super::expression_to_abelian_vector(&expr, &lang);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 1); // one +
        assert_eq!(vec[1], 0); // no *
        assert_eq!(vec[2], 0); // no sin
    }

    #[test]
    fn test_expression_to_abelian_vector_nested() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("*") // id: 1
            .add_symbol("sin"); // id: 2

        // Expression: (+ (sin 1) (* 2 3))
        // Has: 1 +, 1 *, 1 sin
        let expr = lang.parse("(+ (sin 1) (* 2 3))").unwrap();
        let vec = super::expression_to_abelian_vector(&expr, &lang);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0], 1); // one +
        assert_eq!(vec[1], 1); // one *
        assert_eq!(vec[2], 1); // one sin
    }

    #[test]
    fn test_expression_to_abelian_vector_repeated() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("*"); // id: 1

        // Expression: (+ (* 1 2) (+ 3 (* 4 5)))
        // Has: 2 +, 2 *
        let expr = lang.parse("(+ (* 1 2) (+ 3 (* 4 5)))").unwrap();
        let vec = super::expression_to_abelian_vector(&expr, &lang);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 2); // two +
        assert_eq!(vec[1], 2); // two *
    }

    #[test]
    fn test_expression_to_abelian_vector_with_variables() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("*"); // id: 1

        // Expression: (+ $0 (* $1 $2))
        // Has: 1 +, 1 *
        // Variables don't count as symbols
        let expr = lang.parse("(+ $0 (* $1 $2))").unwrap();
        let vec = super::expression_to_abelian_vector(&expr, &lang);

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 1); // one +
        assert_eq!(vec[1], 1); // one *
    }

    #[test]
    fn test_rules_to_abelian_matrix_single_rule() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("*"); // id: 1

        // Rule: (+ $0 $1) -> (* $0 $1)
        // Left: 1 +, 0 *
        // Right: 0 +, 1 *
        // Difference: -1 +, +1 *
        let rule = Rule::from_strings("(+ $0 $1)", "(* $0 $1)", &lang);
        let matrix = super::rules_to_abelian_matrix(&[rule], &lang);

        assert_eq!(matrix.nrows(), 2); // 2 symbols
        assert_eq!(matrix.ncols(), 1); // 1 rule

        assert_eq!(matrix[(0, 0)], -1); // + count difference
        assert_eq!(matrix[(1, 0)], 1); // * count difference
    }

    #[test]
    fn test_rules_to_abelian_matrix_multiple_rules() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("*") // id: 1
            .add_symbol("sin"); // id: 2

        // Rule 1: (+ $0 $1) -> (* $0 $1)
        // Difference: -1 +, +1 *, 0 sin
        let rule1 = Rule::from_strings("(+ $0 $1)", "(* $0 $1)", &lang);

        // Rule 2: (sin $0) -> (+ $0 (* 1 $0))
        // Left: 0 +, 0 *, 1 sin
        // Right: 1 +, 1 *, 0 sin
        // Difference: +1 +, +1 *, -1 sin
        let rule2 = Rule::from_strings("(sin $0)", "(+ $0 (* 1 $0))", &lang);

        let matrix = super::rules_to_abelian_matrix(&[rule1, rule2], &lang);

        assert_eq!(matrix.nrows(), 3); // 3 symbols
        assert_eq!(matrix.ncols(), 2); // 2 rules

        // Rule 1 column (index 0)
        assert_eq!(matrix[(0, 0)], -1); // + count difference
        assert_eq!(matrix[(1, 0)], 1); // * count difference
        assert_eq!(matrix[(2, 0)], 0); // sin count difference

        // Rule 2 column (index 1)
        assert_eq!(matrix[(0, 1)], 1); // + count difference
        assert_eq!(matrix[(1, 1)], 1); // * count difference
        assert_eq!(matrix[(2, 1)], -1); // sin count difference
    }

    #[test]
    fn test_rules_to_abelian_matrix_identity_rule() {
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("*"); // id: 1

        // Identity-like rule: (+ $0 $1) -> (+ $1 $0)
        // Left: 1 +, 0 *
        // Right: 1 +, 0 *
        // Difference: 0 +, 0 *
        let rule = Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang);
        let matrix = super::rules_to_abelian_matrix(&[rule], &lang);

        assert_eq!(matrix.nrows(), 2);
        assert_eq!(matrix.ncols(), 1);

        assert_eq!(matrix[(0, 0)], 0); // no change in +
        assert_eq!(matrix[(1, 0)], 0); // no change in *
    }

    #[test]
    fn test_matrix_indexing() {
        // Verify that our matrix indexing is correct
        let lang = Language::default()
            .add_symbol("+") // id: 0
            .add_symbol("*"); // id: 1

        // Rule: (+ $0 $1) -> (* $0 $1)
        // Symbol 0 (+): left=1, right=0, diff=-1
        // Symbol 1 (*): left=0, right=1, diff=+1
        let rule = Rule::from_strings("(+ $0 $1)", "(* $0 $1)", &lang);
        let matrix = super::rules_to_abelian_matrix(&[rule], &lang);

        // Check dimensions
        assert_eq!(matrix.nrows(), 2); // 2 symbols
        assert_eq!(matrix.ncols(), 1); // 1 rule

        // Verify values
        assert_eq!(matrix[(0, 0)], -1, "Symbol 0 (+) should have -1");
        assert_eq!(matrix[(1, 0)], 1, "Symbol 1 (*) should have +1");
    }
}
