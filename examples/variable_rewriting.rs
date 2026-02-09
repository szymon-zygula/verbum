//! Example demonstrating rewriting of expressions with variables.
//!
//! This example shows how to use the random rewriter to perform
//! destructive term rewriting on expressions that contain variables.
//! Variables in expressions are treated as symbolic constants.

use verbum::language::Language;
use verbum::language::expression::LangExpression;
use verbum::rewriting::direct::with_vars;
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
    ];

    println!("=== Rewriting Expressions with Variables ===\n");

    // Example 1: Commute addition with expression variable
    // Expression: (+ $5 2) where $5 is a variable in the expression
    let expr1 = lang.parse("(+ $5 2)").unwrap();
    println!("Expression: {}", LangExpression::borrowed(&expr1, &lang));
    println!("Applying commutativity rule: (+ $0 $1) -> (+ $1 $0)");
    let rewritten1 = with_vars::rewrite(expr1, &rules, 1);
    println!("Result: {}\n", LangExpression::borrowed(&rewritten1, &lang));

    // Example 2: Simplify expression with variables
    // Expression: (+ 0 (* 1 $3))
    let expr2 = lang.parse("(+ 0 (* 1 $3))").unwrap();
    println!("Expression: {}", LangExpression::borrowed(&expr2, &lang));
    println!("Applying simplification rules");
    let rewritten2 = with_vars::rewrite(expr2, &rules, 2);
    println!("Result: {}\n", LangExpression::borrowed(&rewritten2, &lang));

    // Example 3: Multiple variables
    // Expression: (* $0 (+ $1 0))
    let expr3 = lang.parse("(* $0 (+ $1 0))").unwrap();
    println!("Expression: {}", LangExpression::borrowed(&expr3, &lang));
    println!("Applying rules (should simplify nested addition)");
    let rewritten3 = with_vars::rewrite(expr3, &rules, 3);
    println!("Result: {}\n", LangExpression::borrowed(&rewritten3, &lang));

    // Example 4: No rewrite possible
    let expr4 = lang.parse("$7").unwrap();
    println!("Expression: {}", LangExpression::borrowed(&expr4, &lang));
    println!("Applying rules (no rules match a single variable)");
    let rewritten4 = with_vars::rewrite(expr4, &rules, 5);
    println!("Result: {}\n", LangExpression::borrowed(&rewritten4, &lang));

    println!("=== Key Points ===");
    println!("1. Variables in expressions ($5, $3, $0, etc.) are treated as symbolic constants");
    println!("2. Pattern variables in rules (also $0, $1, etc.) can match any subexpression");
    println!("3. This allows rules to be applied to expressions containing variables");
    println!("4. The rule (+ $0 $1) -> (+ $1 $0) matches (+ $5 2) with $0=$5 and $1=2");
}
