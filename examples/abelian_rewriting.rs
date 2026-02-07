/// Example demonstrating abelian rewriting features.
///
/// This example shows how to:
/// 1. Convert expressions to abelianized vectors (symbol count vectors)
/// 2. Create abelianized TRS matrices from rewrite rules
/// 3. Use these features with the string language representation
use std::collections::HashMap;
use verbum::{
    language::{Language, expression::AnyExpression},
    rewriting::{
        rule::Rule,
        strings::{
            expression_to_abelian_vector, expression_to_paths, rule_to_induced_rules,
            rules_to_abelian_matrix, to_string_language,
        },
    },
};

/// Helper function to print a vector nicely
fn print_vector(vec: &nalgebra::DVector<i32>, lang: &Language) {
    print!("[");
    for i in 0..vec.len() {
        if i > 0 {
            print!(", ");
        }
        print!("{}: {}", lang.get_symbol(i), vec[i]);
    }
    println!("]");
}

fn main() {
    println!("=== Abelian Rewriting Example ===\n");

    // Create a simple mathematical language
    let lang = Language::default()
        .add_symbol("+") // id: 0, binary operator
        .add_symbol("*") // id: 1, binary operator
        .add_symbol("sin") // id: 2, unary operator
        .add_symbol("cos"); // id: 3, unary operator

    println!("Language symbols:");
    for i in 0..lang.symbol_count() {
        println!("  {}: {}", i, lang.get_symbol(i));
    }
    println!();

    // Example 1: Abelianized vectors for expressions
    println!("=== Abelianized Vectors ===");

    let expr1 = lang.parse("(+ 1 2)").unwrap();
    println!("Expression: (+ 1 2)");
    let vec1 = expression_to_abelian_vector(&expr1, &lang);
    print!("Abelian vector: ");
    print_vector(&vec1, &lang);
    println!();

    let expr2 = lang.parse("(+ (sin 1) (* 2 3))").unwrap();
    println!("Expression: (+ (sin 1) (* 2 3))");
    let vec2 = expression_to_abelian_vector(&expr2, &lang);
    print!("Abelian vector: ");
    print_vector(&vec2, &lang);
    println!();

    let expr3 = lang.parse("(* (+ 1 2) (+ 3 (cos 4)))").unwrap();
    println!("Expression: (* (+ 1 2) (+ 3 (cos 4)))");
    let vec3 = expression_to_abelian_vector(&expr3, &lang);
    print!("Abelian vector: ");
    print_vector(&vec3, &lang);
    println!();

    // Example 2: Abelianized TRS matrix
    println!("=== Abelianized TRS Matrix ===");

    // Create some rewrite rules
    let rules = vec![
        Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang), // Commutativity of +
        Rule::from_strings("(* $0 $1)", "(* $1 $0)", &lang), // Commutativity of *
        Rule::from_strings("(+ $0 0)", "$0", &lang),         // Identity for +
        Rule::from_strings("(sin (cos $0))", "(cos (sin $0))", &lang), // Swap sin/cos
    ];

    println!("Rewrite rules:");
    for (i, rule) in rules.iter().enumerate() {
        println!(
            "  Rule {}: {} -> {}",
            i + 1,
            rule.from().with_language(&lang),
            rule.to().with_language(&lang)
        );
    }
    println!();

    let matrix = rules_to_abelian_matrix(&rules, &lang);
    println!("Abelianized TRS matrix A:");
    println!("(rows = symbols, columns = rules)");
    println!("A[i,j] = count of symbol i in right side - count in left side for rule j");
    println!();
    println!("{}", matrix);
    println!();

    println!("Interpretation:");
    println!("  Column 0 (+ commutativity): All zeros - no symbols added or removed");
    println!("  Column 1 (* commutativity): All zeros - no symbols added or removed");
    println!("  Column 2 (+ identity): -1 in row 0 - removes one + symbol");
    println!("  Column 3 (sin/cos swap): All zeros - both sides have 1 sin and 1 cos");
    println!("  Note: Abelianization only counts symbols, not their positions or nesting");
    println!();

    // Example 3: Combining with string language
    println!("=== Abelian Vectors for String Language Paths ===");

    let mut arities = HashMap::new();
    arities.insert(0, 2); // + has arity 2
    arities.insert(1, 2); // * has arity 2
    arities.insert(2, 1); // sin has arity 1
    arities.insert(3, 1); // cos has arity 1

    let string_lang = to_string_language(&lang, &arities);

    println!("String language symbols:");
    for i in 0..string_lang.symbol_count() {
        println!("  {}: {}", i, string_lang.get_symbol(i));
    }
    println!();

    let expr = lang.parse("(+ (sin 1) (* 2 3))").unwrap();
    println!("Original expression: (+ (sin 1) (* 2 3))");

    let paths = expression_to_paths(&expr, &lang, &string_lang, &arities);
    println!("\nPaths and their abelian vectors:");
    for (i, path) in paths.iter().enumerate() {
        println!("  Path {}: {}", i + 1, path.with_language(&string_lang));
        let path_vec = expression_to_abelian_vector(path, &string_lang);
        print!("    Abelian vector: ");
        print_vector(&path_vec, &string_lang);
    }
    println!();

    // Example 4: Abelianized TRS matrix for stringified rewriting system
    println!("=== Abelianized TRS Matrix for Stringified Rewriting System ===");

    // Use all the rules from the original TRS
    let original_rules_for_stringified = vec![
        Rule::from_strings("(+ $0 $1)", "(+ $1 $0)", &lang), // Commutativity of +
        Rule::from_strings("(* $0 $1)", "(* $1 $0)", &lang), // Commutativity of *
        Rule::from_strings("(+ $0 0)", "$0", &lang),         // Identity for +
        Rule::from_strings("(sin (cos $0))", "(cos (sin $0))", &lang), // Swap sin/cos
    ];

    println!("Original rules:");
    for (i, rule) in original_rules_for_stringified.iter().enumerate() {
        println!(
            "  Rule {}: {} -> {}",
            i + 1,
            rule.from().with_language(&lang),
            rule.to().with_language(&lang)
        );
    }
    println!();

    // Convert to induced string rewriting rules
    let mut all_induced_rules = Vec::new();
    for rule in &original_rules_for_stringified {
        let induced = rule_to_induced_rules(rule, &lang, &string_lang, &arities);
        all_induced_rules.extend(induced);
    }

    println!(
        "Induced string rewriting rules ({} rules):",
        all_induced_rules.len()
    );
    for (i, induced_rule) in all_induced_rules.iter().enumerate() {
        println!(
            "  Rule {}: {} -> {}",
            i + 1,
            induced_rule.from().with_language(&string_lang),
            induced_rule.to().with_language(&string_lang)
        );
    }
    println!();

    // Create abelianized TRS matrix for the stringified rules
    let string_matrix = rules_to_abelian_matrix(&all_induced_rules, &string_lang);
    println!("Abelianized TRS matrix for stringified rewriting system:");
    println!("(rows = string language symbols, columns = induced rules)");
    println!();
    println!("{}", string_matrix);
    println!();

    println!("Interpretation:");
    println!("  Each column represents an induced rule showing net symbol changes.");
    println!("  Rules 1-2: From (+ $0 $1) -> (+ $1 $0) - swap +_1 and +_2");
    println!("  Rules 3-4: From (* $0 $1) -> (* $1 $0) - swap *_1 and *_2");
    println!("  Rule 5: From (+ $0 0) -> $0 - removes +_1 (first arg position)");
    println!("  Rule 6: From (sin (cos $0)) -> (cos (sin $0)) - no net symbol change");
    println!();

    // Example 5: Show how matrix multiplication works
    println!("=== Matrix Application Example ===");

    let simple_rules = vec![
        Rule::from_strings("(+ $0 $1)", "(* $0 $1)", &lang), // + -> *
    ];

    let simple_matrix = rules_to_abelian_matrix(&simple_rules, &lang);
    println!("Rule: (+ $0 $1) -> (* $0 $1)");
    println!("Matrix:");
    println!("{}", simple_matrix);

    let before_expr = lang.parse("(+ 1 2)").unwrap();
    let before_vec = expression_to_abelian_vector(&before_expr, &lang);

    println!("Before expression: (+ 1 2)");
    print!("Before vector: ");
    print_vector(&before_vec, &lang);

    // Apply the rule conceptually
    let after_expr = lang.parse("(* 1 2)").unwrap();
    let after_vec = expression_to_abelian_vector(&after_expr, &lang);

    println!("After expression: (* 1 2)");
    print!("After vector: ");
    print_vector(&after_vec, &lang);

    // The difference should match the matrix column
    let diff = &after_vec - &before_vec;
    print!("Difference (after - before): ");
    print_vector(&diff, &lang);
    println!("This matches column 0 of the matrix!");
}
