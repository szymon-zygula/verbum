//! Example demonstrating the random rewriter.
//!
//! This example shows how to use the random rewriter to perform
//! destructive term rewriting on expressions.

use verbum::language::Language;
use verbum::rewriting::random::rewrite;
use verbum::rewriting::rule::Rule;

fn main() {
    // Create a simple math language
    let lang = Language::simple_math();

    // Define some rewrite rules
    let rules = vec![
        // Simplification rules
        Rule::from_strings("(+ 0 $0)", "$0", &lang),
        Rule::from_strings("(+ $0 0)", "$0", &lang),
        Rule::from_strings("(* 1 $0)", "$0", &lang),
        Rule::from_strings("(* $0 1)", "$0", &lang),
        // Commutativity
        Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang),
        Rule::from_strings("(* $0 $1)", "(* $1 $0)", &lang),
        // Bit shift optimization
        Rule::from_strings("(* $0 2)", "(<< $0 1)", &lang),
    ];

    // Create an expression to rewrite
    let expr = lang.parse_no_vars("(+ 0 (* 1 (+ 5 0)))").unwrap();
    println!("Original expression: {}", expr);

    // Apply random rewrites
    let rewritten = rewrite(expr.clone(), &rules, 5);
    println!("After 5 random rewrites: {}", rewritten);

    // Try another example with multiplication
    let expr2 = lang.parse_no_vars("(* 2 (+ 3 4))").unwrap();
    println!("\nOriginal expression: {}", expr2);

    let rewritten2 = rewrite(expr2, &rules, 10);
    println!("After 10 random rewrites: {}", rewritten2);

    // Example with nested operations
    let expr3 = lang.parse_no_vars("(+ 0 (+ 0 (+ 0 5)))").unwrap();
    println!("\nOriginal expression: {}", expr3);

    let rewritten3 = rewrite(expr3, &rules, 3);
    println!("After 3 random rewrites: {}", rewritten3);
}
