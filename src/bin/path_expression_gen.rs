//! Binary for generating random expressions and extracting abelianized path data.
//!
//! This binary:
//! 1. Loads a term rewriting system (TRS) and language from JSON
//! 2. Generates a random expression E of specified size with variables
//! 3. Applies random rewrites to create E'
//! 4. Creates an abelianized matrix A from the TRS rules
//! 5. Finds common variables in E and E'
//! 6. Extracts paths to those variables
//! 7. Outputs abelianized path data as JSON

use clap::Parser;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::path::PathBuf;
use verbum::benchmark::{
    RandomGenerationConfig, VariableGenerationConfig, generate_random_expression_by_size_with_variables,
};
use verbum::language::arities::Arities;
use verbum::language::expression::{AnyExpression, Expression, VariableId};
use verbum::rewriting::random::rewrite_expression;
use verbum::rewriting::strings::{
    expression_to_abelian_vector, expression_to_paths, rules_to_abelian_matrix, to_string_language,
};
use verbum::rewriting::system::TermRewritingSystem;
use verbum::utils::json::{load_json, save_json};

/// CLI arguments for path expression generation
#[derive(Parser, Debug)]
#[command(author, version, about = "Generate random expressions and extract abelianized path data", long_about = None)]
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

    /// Output JSON file path
    #[arg(short = 'o', long)]
    output: PathBuf,
}

/// Output structure for JSON serialization
#[derive(Serialize, Deserialize, Debug)]
struct OutputData {
    /// Abelianized paths from E to variable k
    #[serde(rename = "P_a")]
    p_a: Vec<Vec<i32>>,

    /// Abelianized paths from E' to variable k
    #[serde(rename = "P_a_prime")]
    p_a_prime: Vec<Vec<i32>>,

    /// Abelianized TRS matrix for string language
    #[serde(rename = "A")]
    a: Vec<Vec<i32>>,

    /// The chosen variable k
    #[serde(rename = "k")]
    k: VariableId,

    /// Original expression E (for reference)
    #[serde(rename = "E")]
    e: String,

    /// Rewritten expression E' (for reference)
    #[serde(rename = "E_prime")]
    e_prime: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Step 1: Load TRS and language
    println!("Loading TRS from {:?}...", args.trs);
    let trs = TermRewritingSystem::from_directory(&args.trs)?;
    let lang = trs.language();
    let rules = trs.rules();

    // Step 2: Load arities from the same directory
    let arities_path = args.trs.join("arities.json");
    println!("Loading arities from {:?}...", arities_path);
    let arities: Arities = load_json(&arities_path)?;

    // Step 3: Generate random expression E with size n and up to v variables
    println!("Generating random expression of size {} with up to {} variables...", args.size, args.variables);
    let mut rng = rand::thread_rng();
    
    // Create generation config
    let mut config = RandomGenerationConfig::from_language(lang);
    
    // Set symbol arities from the loaded arities
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
    
    let expr_e = generate_random_expression_by_size_with_variables(
        lang,
        args.size,
        &mut rng,
        &config,
    )?;
    
    println!("Generated expression E: {}", expr_e.with_language(lang));

    // Step 4: Apply a random rewrites to E creating E'
    println!("Applying {} random rewrites...", args.applications);
    let expr_e_prime = rewrite_expression(expr_e.clone(), rules, args.applications, &mut rng);
    
    println!("Rewritten expression E': {}", expr_e_prime.with_language(lang));

    // Step 5: Create abelianized stringified matrix A for trs
    println!("Creating abelianized matrix for TRS...");
    let string_lang = to_string_language(lang, &arities);
    
    // Convert all rules to induced string rules
    let mut all_induced_rules = Vec::new();
    for rule in rules {
        let induced = verbum::rewriting::strings::rule_to_induced_rules(
            rule,
            lang,
            &string_lang,
            &arities,
        );
        all_induced_rules.extend(induced);
    }
    
    let matrix_a = rules_to_abelian_matrix(&all_induced_rules, &string_lang);

    // Step 6: Choose a random variable k appearing in both E and E'
    println!("Finding common variables...");
    let vars_e: HashSet<VariableId> = expr_e.variables();
    let vars_e_prime: HashSet<VariableId> = expr_e_prime.variables();
    
    let common_vars: Vec<VariableId> = vars_e.intersection(&vars_e_prime).copied().collect();
    
    if common_vars.is_empty() {
        return Err("No common variables found between E and E'. Try again with different parameters or more variables.".into());
    }
    
    let k = *common_vars.choose(&mut rng)
        .ok_or("Failed to choose a random variable")?;
    
    println!("Chosen variable k: ${}", k);

    // Step 7: Extract all paths from root to leaves which are variable k
    println!("Extracting paths to variable ${} in E...", k);
    let all_paths_e = expression_to_paths(&expr_e, lang, &string_lang, &arities);
    let paths_to_k_e: Vec<_> = all_paths_e.iter()
        .filter(|path| {
            // Check if this path ends with variable k
            is_path_to_variable(path, k)
        })
        .collect();
    
    println!("Extracting paths to variable ${} in E'...", k);
    let all_paths_e_prime = expression_to_paths(&expr_e_prime, lang, &string_lang, &arities);
    let paths_to_k_e_prime: Vec<_> = all_paths_e_prime.iter()
        .filter(|path| {
            // Check if this path ends with variable k
            is_path_to_variable(path, k)
        })
        .collect();
    
    println!("Found {} paths to ${} in E", paths_to_k_e.len(), k);
    println!("Found {} paths to ${} in E'", paths_to_k_e_prime.len(), k);

    // Step 8: Map paths to abelianized versions
    println!("Converting paths to abelianized vectors...");
    let p_a: Vec<Vec<i32>> = paths_to_k_e.iter()
        .map(|path| {
            let vec = expression_to_abelian_vector(path, &string_lang);
            vec.as_slice().to_vec()
        })
        .collect();
    
    let p_a_prime: Vec<Vec<i32>> = paths_to_k_e_prime.iter()
        .map(|path| {
            let vec = expression_to_abelian_vector(path, &string_lang);
            vec.as_slice().to_vec()
        })
        .collect();

    // Step 9: Convert matrix to Vec<Vec<i32>> for JSON serialization
    let a_data: Vec<Vec<i32>> = (0..matrix_a.nrows())
        .map(|row| {
            (0..matrix_a.ncols())
                .map(|col| matrix_a[(row, col)])
                .collect()
        })
        .collect();

    // Step 10: Save to JSON
    println!("Saving results to {:?}...", args.output);
    let output = OutputData {
        p_a,
        p_a_prime,
        a: a_data,
        k,
        e: format!("{}", expr_e.with_language(lang)),
        e_prime: format!("{}", expr_e_prime.with_language(lang)),
    };
    
    save_json(&output, &args.output)?;
    
    println!("Successfully saved results!");
    println!("\nSummary:");
    println!("  Expression size: {}", args.size);
    println!("  Variables: 0-{}", args.variables - 1);
    println!("  Random applications: {}", args.applications);
    println!("  Chosen variable: ${}", k);
    println!("  Paths in E: {}", output.p_a.len());
    println!("  Paths in E': {}", output.p_a_prime.len());
    println!("  Matrix dimensions: {}x{}", output.a.len(), if output.a.is_empty() { 0 } else { output.a[0].len() });
    
    Ok(())
}

/// Checks if a path expression ends with the given variable
fn is_path_to_variable(path: &Expression, var_id: VariableId) -> bool {
    // Navigate to the deepest nested child
    let mut current = path;
    loop {
        match current {
            Expression::Variable(id) => {
                return *id == var_id;
            }
            Expression::Literal(_) => {
                return false;
            }
            Expression::Symbol(symbol) => {
                if symbol.children.is_empty() {
                    return false;
                }
                // For a path, there should be exactly one child
                current = &symbol.children[0];
            }
        }
    }
}
