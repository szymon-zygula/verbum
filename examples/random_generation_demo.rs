// Example demonstrating the random expression generation features

use std::path::PathBuf;
use verbum::benchmark::{
    LiteralGenerationConfig, RandomGenerationConfig,
    generate_random_expression_by_size_with_config, generate_random_expression_with_config,
};
use verbum::language::{Language, arities::Arities};
use verbum::rewriting::system::TermRewritingSystem;
use verbum::utils;

fn default_math_config(lang: &Language, arities: &Arities) -> RandomGenerationConfig {
    let mut config = RandomGenerationConfig::from_language(lang);
    // Set up arities from the loaded arities structure
    for symbol_id in 0..lang.symbol_count() {
        if let Some(symbol_arities) = arities.get(symbol_id) {
            config.symbol_arities[symbol_id] = symbol_arities.to_vec();
        }
    }
    config
}

fn main() {
    // Load language and arities from JSON files
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("jsons");
    path.push("random-generation");

    // Load the term rewriting system (language + rules)
    let trs = TermRewritingSystem::from_directory(&path).unwrap();
    let lang = trs.language();

    // Load arities
    let arities_path = path.join("arities.json");
    let arities: Arities = utils::json::load_json(arities_path).unwrap();

    let mut rng = rand::thread_rng();
    let config = default_math_config(lang, &arities);

    println!("=== Depth-Based Generation ===");
    for depth in 1..=5 {
        let expr = generate_random_expression_with_config(lang, depth, &mut rng, &config);
        println!("Depth {}: {:?}", depth, expr);
    }

    println!("\n=== Size-Based Generation ===");
    for size in 1..=10 {
        match generate_random_expression_by_size_with_config(lang, size, &mut rng, &config) {
            Ok(expr) => println!("Target size {}: {:?}", size, expr),
            Err(e) => println!("Target size {}: Error - {}", size, e),
        }
    }

    println!("\n=== Custom Configuration: Common Values ===");
    let literal_config = LiteralGenerationConfig {
        common_int_values: vec![0, 1, 2],
        common_value_probability: 0.9,
        int_range: (-100, 100),
        uint_range: (0, 100),
        int_vs_uint_probability: 1.0, // Always use int
    };

    let config_common = default_math_config(lang, &arities)
        .with_literal_config(literal_config)
        .with_literal_probability(0.5);

    println!("Expressions with common values (0, 1, 2):");
    for i in 1..=5 {
        let expr = generate_random_expression_with_config(lang, 3, &mut rng, &config_common);
        println!("{}: {:?}", i, expr);
    }

    println!("\n=== Custom Configuration: Fixed Arities ===");
    // Make + only accept exactly 3 children
    let plus_id = lang.get_id("+");
    let config_fixed_arity = default_math_config(lang, &arities).with_symbol_arities(plus_id, vec![3]);

    println!("Expressions where + has exactly 3 children:");
    for i in 1..=5 {
        let expr = generate_random_expression_with_config(lang, 5, &mut rng, &config_fixed_arity);
        println!("{}: {:?}", i, expr);
    }
}
