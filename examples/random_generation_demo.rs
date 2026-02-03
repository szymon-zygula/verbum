// Example demonstrating the random expression generation features

use verbum::benchmark::{
    generate_random_expression, generate_random_expression_by_size,
    generate_random_expression_with_config, LiteralGenerationConfig, RandomGenerationConfig,
};
use verbum::language::Language;

fn main() {
    let lang = Language::simple_math();
    let mut rng = rand::thread_rng();

    println!("=== Depth-Based Generation ===");
    for depth in 1..=5 {
        let expr = generate_random_expression(&lang, depth, &mut rng);
        println!("Depth {}: {:?}", depth, expr);
    }

    println!("\n=== Size-Based Generation ===");
    for size in 1..=10 {
        let expr = generate_random_expression_by_size(&lang, size, &mut rng);
        println!("Target size {}: {:?}", size, expr);
    }

    println!("\n=== Custom Configuration: Common Values ===");
    let literal_config = LiteralGenerationConfig {
        common_int_values: vec![0, 1, 2],
        common_value_probability: 0.9,
        int_range: (-100, 100),
        uint_range: (0, 100),
        int_vs_uint_probability: 1.0, // Always use int
    };

    let config = RandomGenerationConfig::from_language(&lang)
        .with_literal_config(literal_config)
        .with_literal_probability(0.5);

    println!("Expressions with common values (0, 1, 2):");
    for i in 1..=5 {
        let expr = generate_random_expression_with_config(&lang, 3, &mut rng, &config);
        println!("{}: {:?}", i, expr);
    }

    println!("\n=== Custom Configuration: Fixed Arities ===");
    // Make + only accept exactly 3 children
    let plus_id = lang.get_id("+");
    let config_fixed_arity = RandomGenerationConfig::from_language(&lang)
        .with_symbol_arities(plus_id, vec![3]);

    println!("Expressions where + has exactly 3 children:");
    for i in 1..=5 {
        let expr = generate_random_expression_with_config(&lang, 5, &mut rng, &config_fixed_arity);
        println!("{}: {:?}", i, expr);
    }
}
