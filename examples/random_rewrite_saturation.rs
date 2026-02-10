//! Example demonstrating the random rewriter with saturation.
//!
//! This example shows how to use the random rewriter to perform
//! destructive term rewriting on expressions, loading the language
//! and term rewriting system from JSON configuration files.

use std::path::PathBuf;
use verbum::rewriting::random::rewrite;
use verbum::rewriting::system::TermRewritingSystem;

fn main() {
    // Load language and TRS from JSON files
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("jsons");
    path.push("random-rewrite-saturation");

    let trs = TermRewritingSystem::from_directory(path).unwrap();
    let lang = trs.language();
    let rules = trs.rules();

    // Create an expression to rewrite
    let expr = lang.parse_no_vars("(+ 0 (* 1 (+ 5 0)))").unwrap();
    println!("Original expression: {}", expr);

    // Apply random rewrites
    let rewritten = rewrite(expr.clone(), rules, 5);
    println!("After 5 random rewrites: {}", rewritten);

    // Try another example with multiplication
    let expr2 = lang.parse_no_vars("(* 2 (+ 3 4))").unwrap();
    println!("\nOriginal expression: {}", expr2);

    let rewritten2 = rewrite(expr2, rules, 10);
    println!("After 10 random rewrites: {}", rewritten2);

    // Example with nested operations
    let expr3 = lang.parse_no_vars("(+ 0 (+ 0 (+ 0 5)))").unwrap();
    println!("\nOriginal expression: {}", expr3);

    let rewritten3 = rewrite(expr3, rules, 3);
    println!("After 3 random rewrites: {}", rewritten3);
}
