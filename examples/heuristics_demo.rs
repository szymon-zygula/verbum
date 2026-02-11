//! Heuristic benchmark example.
//!
//! This example:
//! 1. Loads a term rewriting system (TRS) and language from JSON
//! 2. Generates a random expression E of specified size with variables
//! 3. Applies random rewrites to create E'
//! 4. Creates a heuristic for E'
//! 5. Times the heuristic evaluation for E and E'

use clap::Parser;
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;
use verbum::benchmark::{
    RandomGenerationConfig, VariableGenerationConfig,
    generate_random_expression_by_size_with_variables,
};
use verbum::language::arities::Arities;
use verbum::language::expression::AnyExpression;
use verbum::rewriting::heuristic::{AbelianPathHeuristic, Heuristic};
use verbum::rewriting::random::rewrite_expression;
use verbum::rewriting::system::TermRewritingSystem;
use verbum::utils::json::load_json;

/// CLI arguments for heuristic benchmark
#[derive(Parser, Debug)]
#[command(author, version, about = "Benchmark heuristic evaluation", long_about = None)]
struct Args {
    /// Expression size (n)
    #[arg(short = 'n', long)]
    size: usize,

    /// Variable count (v) - maximum variable ID will be v-1
    #[arg(short = 'v', long)]
    variables: usize,

    /// Path to directory containing TRS JSON files (language.json, trs.json, and arities.json)
    #[arg(short = 't', long)]
    trs: PathBuf,

    /// Number of random rewrite applications (a)
    #[arg(short = 'a', long)]
    applications: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Load TRS and language
    println!("Loading TRS from {:?}...", args.trs);
    let trs = TermRewritingSystem::from_directory(&args.trs)?;
    let lang = trs.language();
    let rules = trs.rules();

    // Load arities
    let arities_path = args.trs.join("arities.json");
    println!("Loading arities from {:?}...", arities_path);
    let arities: Arities = load_json(&arities_path)?;

    // Generate random expression E
    println!(
        "Generating random expression of size {} with up to {} variables...",
        args.size, args.variables
    );
    let mut rng = rand::thread_rng();

    let mut config = RandomGenerationConfig::from_language(lang);

    // Set symbol arities
    for symbol_id in 0..lang.symbol_count() {
        if let Some(symbol_arities) = arities.get(symbol_id) {
            config.symbol_arities[symbol_id] = symbol_arities.to_vec();
        }
    }

    // Configure variable generation
    if args.variables > 0 {
        config.variable_config = Some(VariableGenerationConfig {
            variable_range: (0, args.variables - 1),
            variable_probability: 0.5,
        });
    }

    let expr_e =
        generate_random_expression_by_size_with_variables(lang, args.size, &mut rng, &config)?;
    let expr_f =
        generate_random_expression_by_size_with_variables(lang, args.size, &mut rng, &config)?;

    println!("Generated expression E: {}", expr_e.with_language(lang));

    // Apply random rewrites to create E'
    println!("Applying {} random rewrites...", args.applications);
    let expr_e_prime = rewrite_expression(expr_e.clone(), rules, args.applications, &mut rng);

    println!(
        "Rewritten expression E': {}",
        expr_e_prime.with_language(lang)
    );

    // Create heuristic for E'
    println!("\nCreating heuristic for target expression E'...");
    let heuristic_start = Instant::now();
    let heuristic = AbelianPathHeuristic::new(&expr_e_prime, &trs, &arities);
    let heuristic_construction_time = heuristic_start.elapsed();
    println!(
        "Heuristic construction time: {:.3}ms",
        heuristic_construction_time.as_secs_f64() * 1000.0
    );

    // Time heuristic evaluation for starting expression E
    println!("\nEvaluating heuristic for starting expression E...");
    let eval_start = Instant::now();
    let distance_e = heuristic.lower_bound_dist(&expr_e);
    let eval_time_e = eval_start.elapsed();
    println!(
        "Heuristic h(E): {} (computed in {:.3}ms)",
        distance_e,
        eval_time_e.as_secs_f64() * 1000.0
    );

    // Time heuristic evaluation for rewritten expression E'
    println!("\nEvaluating heuristic for rewritten expression E'...");
    let eval_start = Instant::now();
    let distance_e_prime = heuristic.lower_bound_dist(&expr_e_prime);
    let eval_time_e_prime = eval_start.elapsed();
    println!(
        "Heuristic h(E'): {} (computed in {:.3}ms)",
        distance_e_prime,
        eval_time_e_prime.as_secs_f64() * 1000.0
    );

    // Time heuristic evaluation for rewritten expression F
    println!("\nEvaluating heuristic for rewritten expression F (unrelated to E and E')...");
    let eval_start = Instant::now();
    let distance_f = heuristic.lower_bound_dist(&expr_f);
    let eval_time_f = eval_start.elapsed();
    println!(
        "Heuristic h(F): {} (computed in {:.3}ms)",
        distance_f,
        eval_time_f.as_secs_f64() * 1000.0
    );

    println!("\n=== Summary ===");
    println!("Expression size: {}", args.size);
    println!("Variables: {}", args.variables);
    println!("Rewrite applications: {}", args.applications);
    println!(
        "Heuristic construction: {:.3}ms",
        heuristic_construction_time.as_secs_f64() * 1000.0
    );
    println!(
        "h(E) evaluation: {:.3}ms",
        eval_time_e.as_secs_f64() * 1000.0
    );
    println!(
        "h(E') evaluation: {:.3}ms",
        eval_time_e_prime.as_secs_f64() * 1000.0
    );
    println!(
        "h(F) evaluation: {:.3}ms",
        eval_time_f.as_secs_f64() * 1000.0
    );

    Ok(())
}
