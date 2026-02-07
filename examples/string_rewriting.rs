/// Example demonstrating the induced string rewriting functionality.
///
/// This example shows how to:
/// 1. Convert a language to its string language representation
/// 2. Convert expressions to root-to-leaf paths
/// 3. Convert rewriting rules to induced string rewriting rules
use std::collections::HashMap;
use verbum::{
    language::{Language, expression::AnyExpression},
    rewriting::{
        rule::Rule,
        strings::{expression_to_paths, rule_to_induced_rules, to_string_language},
    },
};

/// Helper function to print symbol information
fn print_symbol_info(lang: &Language, arities: &HashMap<usize, usize>) {
    for id in 0..lang.symbol_count() {
        let name = lang.get_symbol(id);
        let arity = arities.get(&id).copied().unwrap_or(0);
        println!("  {} (id: {}, arity: {})", name, id, arity);
    }
}

fn main() {
    // Create a simple mathematical language
    let lang = Language::default()
        .add_symbol("+") // id: 0, binary operator
        .add_symbol("*") // id: 1, binary operator
        .add_symbol("sin") // id: 2, unary operator
        .add_symbol("x"); // id: 3, constant (0-arity)

    // Define the arities (arity of each symbol)
    let mut arities = HashMap::new();
    arities.insert(0, 2); // + has arity 2
    arities.insert(1, 2); // * has arity 2
    arities.insert(2, 1); // sin has arity 1
    arities.insert(3, 0); // x has arity 0

    println!("=== String Language Conversion ===");
    println!("Original language symbols:");
    print_symbol_info(&lang, &arities);

    // Convert to string language
    let string_lang = to_string_language(&lang, &arities);
    println!("\nString language symbols:");
    for id in 0..string_lang.symbol_count() {
        println!("  {} (id: {})", string_lang.get_symbol(id), id);
    }

    println!("\n=== Expression to Paths ===");
    // Create an expression: (+ (sin 1) (* 2 3))
    let expr = lang.parse("(+ (sin 1) (* 2 3))").unwrap();
    println!("Original expression: (+ (sin 1) (* 2 3))");
    println!("This has 3 leaves: 1, 2, and 3");

    let paths = expression_to_paths(&expr, &lang, &string_lang, &arities);
    println!("\nPaths from root to leaves: {} paths", paths.len());
    for (i, path) in paths.iter().enumerate() {
        println!("  Path {}: {}", i + 1, path.with_language(&string_lang));
    }

    println!("\n=== Rule to Induced Rules ===");
    // Create a commutativity rule: (+ $0 $1) -> (+ $1 $0)
    let rule = Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang);
    println!("Original rule: (+ $0 $1) -> (+ $1 $0)");

    let induced_rules = rule_to_induced_rules(&rule, &lang, &string_lang, &arities);
    println!("\nInduced rules: {} rules", induced_rules.len());
    for (i, induced_rule) in induced_rules.iter().enumerate() {
        println!(
            "  Rule {}: {} -> {}",
            i + 1,
            induced_rule.from().with_language(&string_lang),
            induced_rule.to().with_language(&string_lang)
        );
    }

    println!("\n=== Another Example with Repeated Variables ===");
    // Create a rule with repeated variable: (+ $0 $0) -> (* 2 $0)
    let rule2 = Rule::from_strings("(+ $0 $0)", "(* 2 $0)", &lang);
    println!("Original rule: (+ $0 $0) -> (* 2 $0)");
    println!("Variable $0 appears twice on the left, once on the right");

    let induced_rules2 = rule_to_induced_rules(&rule2, &lang, &string_lang, &arities);
    println!("\nInduced rules: {} rules", induced_rules2.len());
    for (i, induced_rule) in induced_rules2.iter().enumerate() {
        println!(
            "  Rule {}: {} -> {}",
            i + 1,
            induced_rule.from().with_language(&string_lang),
            induced_rule.to().with_language(&string_lang)
        );
    }
}
