//! Random expression generation for testing and benchmarking.
//!
//! This module provides utilities for generating random expressions,
//! useful for benchmarking term rewriting systems.

use rand::Rng;

use crate::language::{
    Language,
    expression::{Literal, VarFreeExpression},
    symbol::Symbol,
};

/// Configuration for random expression generation.
#[derive(Clone, Debug)]
pub struct RandomGenerationConfig {
    /// Maps symbol IDs to their allowed arities.
    /// If a symbol is not in this map, a default arity will be used.
    pub symbol_arities: Vec<Vec<usize>>,
    
    /// Configuration for generating random literals.
    pub literal_config: LiteralGenerationConfig,
    
    /// Probability of generating a literal at each non-leaf node (0.0 to 1.0).
    pub literal_probability: f64,
}

/// Configuration for generating random literals.
#[derive(Clone, Debug)]
pub struct LiteralGenerationConfig {
    /// Common integer values that should be generated with higher probability.
    pub common_int_values: Vec<i64>,
    
    /// Probability of selecting a common value (0.0 to 1.0).
    pub common_value_probability: f64,
    
    /// Range for random integers when not using common values.
    pub int_range: (i64, i64),
    
    /// Range for random unsigned integers when not using common values.
    pub uint_range: (u64, u64),
    
    /// Probability of generating signed integers vs unsigned (0.0 to 1.0).
    /// 0.0 = always unsigned, 1.0 = always signed.
    pub int_vs_uint_probability: f64,
}

impl Default for LiteralGenerationConfig {
    fn default() -> Self {
        Self {
            common_int_values: vec![-1, 0, 1, 2],
            common_value_probability: 0.4,
            int_range: (-100, 100),
            uint_range: (0, 100),
            int_vs_uint_probability: 0.5,
        }
    }
}

impl RandomGenerationConfig {
    /// Creates a configuration from a language with default settings.
    pub fn from_language(language: &Language) -> Self {
        let symbol_arities = (0..language.symbol_count())
            .map(|id| {
                let symbol_name = language.get_symbol(id);
                // Default arities based on common mathematical symbols
                match symbol_name {
                    "+" | "-" | "*" | "/" | "<<" | ">>" => vec![2],
                    "sin" | "cos" | "tan" | "exp" | "log" | "sqrt" => vec![1],
                    _ => vec![0, 1, 2], // Allow variable arity for unknown symbols
                }
            })
            .collect();
        
        Self {
            symbol_arities,
            literal_config: LiteralGenerationConfig::default(),
            literal_probability: 0.3,
        }
    }
    
    /// Sets the allowed arities for a specific symbol.
    pub fn with_symbol_arities(mut self, symbol_id: usize, arities: Vec<usize>) -> Self {
        if symbol_id < self.symbol_arities.len() {
            self.symbol_arities[symbol_id] = arities;
        }
        self
    }
    
    /// Sets the literal generation configuration.
    pub fn with_literal_config(mut self, literal_config: LiteralGenerationConfig) -> Self {
        self.literal_config = literal_config;
        self
    }
    
    /// Sets the probability of generating literals at non-leaf nodes.
    pub fn with_literal_probability(mut self, probability: f64) -> Self {
        self.literal_probability = probability.clamp(0.0, 1.0);
        self
    }
}

/// Generates a random expression up to a maximum depth.
///
/// # Arguments
///
/// * `language` - The language defining available symbols
/// * `max_depth` - The maximum depth of the expression tree (number of nested levels)
/// * `rng` - The random number generator to use
///
/// # Returns
///
/// Returns a randomly generated expression
pub fn generate_random_expression(
    language: &Language,
    max_depth: usize,
    rng: &mut impl Rng,
) -> VarFreeExpression {
    let config = RandomGenerationConfig::from_language(language);
    generate_random_expression_with_config(language, max_depth, rng, &config)
}

/// Generates a random expression up to a maximum depth with custom configuration.
///
/// # Arguments
///
/// * `language` - The language defining available symbols
/// * `max_depth` - The maximum depth of the expression tree (number of nested levels)
/// * `rng` - The random number generator to use
/// * `config` - Configuration for random generation
///
/// # Returns
///
/// Returns a randomly generated expression
pub fn generate_random_expression_with_config(
    language: &Language,
    max_depth: usize,
    rng: &mut impl Rng,
    config: &RandomGenerationConfig,
) -> VarFreeExpression {
    generate_random_expression_recursive(language, max_depth, rng, 0, config)
}

fn generate_random_expression_recursive(
    language: &Language,
    max_depth: usize,
    rng: &mut impl Rng,
    current_depth: usize,
    config: &RandomGenerationConfig,
) -> VarFreeExpression {
    if current_depth >= max_depth {
        // Generate a literal if max_depth is reached
        return generate_random_literal(rng, &config.literal_config);
    }

    // Decide whether to generate a literal or a symbol
    if rng.gen_bool(config.literal_probability) || language.symbol_count() == 0 {
        // Generate a literal based on configured probability, or if no symbols are defined
        generate_random_literal(rng, &config.literal_config)
    } else {
        // Generate a symbol
        let symbol_id = rng.gen_range(0..language.symbol_count());
        
        // Get allowed arities for this symbol
        let allowed_arities = &config.symbol_arities[symbol_id];
        let arity = if allowed_arities.is_empty() {
            0
        } else {
            allowed_arities[rng.gen_range(0..allowed_arities.len())]
        };

        let children: Vec<VarFreeExpression> = (0..arity)
            .map(|_| {
                generate_random_expression_recursive(language, max_depth, rng, current_depth + 1, config)
            })
            .collect();

        VarFreeExpression::Symbol(Symbol {
            id: symbol_id,
            children,
        })
    }
}

fn generate_random_literal(rng: &mut impl Rng, config: &LiteralGenerationConfig) -> VarFreeExpression {
    // Check if we should use a common value
    if rng.gen_bool(config.common_value_probability) && !config.common_int_values.is_empty() {
        let value = config.common_int_values[rng.gen_range(0..config.common_int_values.len())];
        return VarFreeExpression::Literal(Literal::Int(value));
    }
    
    // Generate a random value
    if rng.gen_bool(config.int_vs_uint_probability) {
        VarFreeExpression::Literal(Literal::Int(rng.gen_range(config.int_range.0..=config.int_range.1)))
    } else {
        VarFreeExpression::Literal(Literal::UInt(rng.gen_range(config.uint_range.0..=config.uint_range.1)))
    }
}

/// Generates a random expression with an exact total size (node count).
///
/// # Arguments
///
/// * `language` - The language defining available symbols
/// * `target_size` - The desired total number of nodes in the expression tree
/// * `rng` - The random number generator to use
///
/// # Returns
///
/// Returns a randomly generated expression with approximately the target size
///
/// # Note
///
/// Due to the random nature of generation, the exact size might vary slightly
/// from the target. The function tries to get as close as possible.
pub fn generate_random_expression_by_size(
    language: &Language,
    target_size: usize,
    rng: &mut impl Rng,
) -> VarFreeExpression {
    let config = RandomGenerationConfig::from_language(language);
    generate_random_expression_by_size_with_config(language, target_size, rng, &config)
}

/// Generates a random expression with an exact total size using custom configuration.
///
/// # Arguments
///
/// * `language` - The language defining available symbols
/// * `target_size` - The desired total number of nodes in the expression tree
/// * `rng` - The random number generator to use
/// * `config` - Configuration for random generation
///
/// # Returns
///
/// Returns a randomly generated expression with approximately the target size
pub fn generate_random_expression_by_size_with_config(
    language: &Language,
    target_size: usize,
    rng: &mut impl Rng,
    config: &RandomGenerationConfig,
) -> VarFreeExpression {
    if target_size == 0 {
        return generate_random_literal(rng, &config.literal_config);
    }
    
    generate_random_expression_by_size_recursive(language, target_size, rng, config)
}

fn generate_random_expression_by_size_recursive(
    language: &Language,
    remaining_size: usize,
    rng: &mut impl Rng,
    config: &RandomGenerationConfig,
) -> VarFreeExpression {
    if remaining_size <= 1 || language.symbol_count() == 0 {
        // Must generate a leaf node (literal)
        return generate_random_literal(rng, &config.literal_config);
    }
    
    // For size-based generation, only generate literals at leaves
    // Generate a symbol
    let symbol_id = rng.gen_range(0..language.symbol_count());
    
    // Get allowed arities for this symbol
    let allowed_arities = &config.symbol_arities[symbol_id];
    if allowed_arities.is_empty() {
        // No children allowed, treat as leaf
        return VarFreeExpression::Symbol(Symbol {
            id: symbol_id,
            children: vec![],
        });
    }
    
    // Filter arities that can fit within remaining size
    let viable_arities: Vec<usize> = allowed_arities
        .iter()
        .copied()
        .filter(|&arity| arity > 0 && arity < remaining_size)
        .collect();
    
    if viable_arities.is_empty() {
        // Can't create symbol with children, generate literal instead
        return generate_random_literal(rng, &config.literal_config);
    }
    
    let arity = viable_arities[rng.gen_range(0..viable_arities.len())];
    
    // Distribute remaining size (minus 1 for current node) among children
    let size_for_children = remaining_size - 1;
    let mut child_sizes = distribute_size(size_for_children, arity, rng);
    
    let children: Vec<VarFreeExpression> = child_sizes
        .iter_mut()
        .map(|&mut child_size| {
            generate_random_expression_by_size_recursive(
                language,
                child_size.max(1),
                rng,
                config,
            )
        })
        .collect();
    
    VarFreeExpression::Symbol(Symbol {
        id: symbol_id,
        children,
    })
}

/// Distributes a total size among a number of children.
/// Returns a vector of sizes for each child.
fn distribute_size(total: usize, num_children: usize, rng: &mut impl Rng) -> Vec<usize> {
    if num_children == 0 {
        return vec![];
    }
    
    if num_children == 1 {
        return vec![total];
    }
    
    // Give each child at least 1 node
    let mut sizes = vec![1; num_children];
    let remaining = total.saturating_sub(num_children);
    
    // Randomly distribute the remaining size more efficiently
    // Generate random split points
    if remaining > 0 {
        let mut split_points: Vec<usize> = (0..num_children - 1)
            .map(|_| rng.gen_range(0..=remaining))
            .collect();
        split_points.push(0);
        split_points.push(remaining);
        split_points.sort_unstable();
        
        for i in 0..num_children {
            sizes[i] += split_points[i + 1] - split_points[i];
        }
    }
    
    sizes
}

#[cfg(test)]
mod tests {
    use super::{
        generate_random_expression, generate_random_expression_by_size,
        generate_random_expression_with_config, LiteralGenerationConfig, RandomGenerationConfig,
    };
    use crate::language::{Language, expression::VarFreeExpression, topology::expression_size};

    fn count_nodes(expr: &VarFreeExpression) -> usize {
        match expr {
            VarFreeExpression::Literal(_) => 1,
            VarFreeExpression::Symbol(symbol) => {
                1 + symbol.children.iter().map(count_nodes).sum::<usize>()
            }
        }
    }

    fn max_depth(expr: &VarFreeExpression) -> usize {
        match expr {
            VarFreeExpression::Literal(_) => 0,
            VarFreeExpression::Symbol(symbol) => {
                if symbol.children.is_empty() {
                    0
                } else {
                    1 + symbol.children.iter().map(max_depth).max().unwrap_or(0)
                }
            }
        }
    }

    #[test]
    fn test_generate_random_expression() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();

        for depth in 1..10 {
            let expr = generate_random_expression(&lang, depth, &mut rng);
            println!("Generated expression (depth {depth}): {expr:?}");
            // Basic assertion: ensure it doesn't panic and is a valid expression
            assert!(!format!("{expr:?}").is_empty());
            
            // Verify depth constraint
            let actual_depth = max_depth(&expr);
            assert!(
                actual_depth <= depth,
                "Expression depth {actual_depth} exceeds max depth {depth}"
            );
        }
    }

    #[test]
    fn test_generate_random_expression_by_size() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();

        // Maximum acceptable deviation from target size (as a fraction)
        const MAX_SIZE_DEVIATION_FRACTION: usize = 3; // 1/3 = 33%

        for target_size in 1..20 {
            let expr = generate_random_expression_by_size(&lang, target_size, &mut rng);
            let actual_size = count_nodes(&expr);
            
            println!(
                "Generated expression (target size {target_size}, actual size {actual_size}): {expr:?}"
            );
            
            // The actual size should be reasonably close to the target
            // Allow some deviation due to randomness and arity constraints
            let tolerance = if target_size < 5 { 1 } else { target_size / MAX_SIZE_DEVIATION_FRACTION };
            assert!(
                actual_size <= target_size + tolerance,
                "Expression size {actual_size} significantly exceeds target size {target_size}"
            );
            assert!(
                actual_size >= target_size.saturating_sub(tolerance).max(1),
                "Expression size {actual_size} is too far below target size {target_size}, tolerance {tolerance}"
            );
        }
    }

    #[test]
    fn test_custom_arity_configuration() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();
        
        // Create a configuration where + only accepts 3 children
        let plus_id = lang.get_id("+");
        let config = RandomGenerationConfig::from_language(&lang)
            .with_symbol_arities(plus_id, vec![3]);
        
        // Generate several expressions and check
        for _ in 0..10 {
            let expr = generate_random_expression_with_config(&lang, 5, &mut rng, &config);
            println!("Generated with custom arity: {expr:?}");
            
            // Verify that any + symbols have exactly 3 children
            verify_symbol_arity(&expr, plus_id, &[3]);
        }
    }

    fn verify_symbol_arity(expr: &VarFreeExpression, symbol_id: usize, allowed_arities: &[usize]) {
        match expr {
            VarFreeExpression::Literal(_) => {}
            VarFreeExpression::Symbol(symbol) => {
                if symbol.id == symbol_id {
                    assert!(
                        allowed_arities.contains(&symbol.children.len()),
                        "Symbol {} has arity {} which is not in allowed arities {:?}",
                        symbol_id,
                        symbol.children.len(),
                        allowed_arities
                    );
                }
                // Recursively check children
                for child in &symbol.children {
                    verify_symbol_arity(child, symbol_id, allowed_arities);
                }
            }
        }
    }

    #[test]
    fn test_common_literal_generation() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();
        
        // Create a configuration with common values
        let literal_config = LiteralGenerationConfig {
            common_int_values: vec![0, 1],
            common_value_probability: 0.9, // Very high probability
            int_range: (-1000, 1000),
            uint_range: (0, 1000),
            int_vs_uint_probability: 1.0, // Always use int
        };
        
        let config = RandomGenerationConfig::from_language(&lang)
            .with_literal_config(literal_config)
            .with_literal_probability(0.8); // High probability of literals
        
        // Generate expressions and count common values
        let mut common_count = 0;
        let total_expressions = 100;
        
        for _ in 0..total_expressions {
            let expr = generate_random_expression_with_config(&lang, 2, &mut rng, &config);
            if contains_common_literal(&expr, &[0, 1]) {
                common_count += 1;
            }
        }
        
        // With 90% probability and high literal generation, we should see many common values
        assert!(
            common_count > total_expressions / 2,
            "Expected more common literals, got {common_count}/{total_expressions}"
        );
    }

    fn contains_common_literal(expr: &VarFreeExpression, common_values: &[i64]) -> bool {
        match expr {
            VarFreeExpression::Literal(crate::language::expression::Literal::Int(val)) => {
                common_values.contains(val)
            }
            VarFreeExpression::Symbol(symbol) => symbol
                .children
                .iter()
                .any(|child| contains_common_literal(child, common_values)),
            _ => false,
        }
    }

    #[test]
    fn test_size_based_vs_depth_based() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();
        
        // Generate expressions with same parameter value using both methods
        let depth_expr = generate_random_expression(&lang, 5, &mut rng);
        let size_expr = generate_random_expression_by_size(&lang, 5, &mut rng);
        
        let depth_size = count_nodes(&depth_expr);
        let size_size = count_nodes(&size_expr);
        
        println!("Depth-based (depth=5) actual size: {depth_size}");
        println!("Size-based (size=5) actual size: {size_size}");
        
        // Size-based should be closer to 5 than depth-based
        assert!(
            (size_size as i32 - 5).abs() <= (depth_size as i32 - 5).abs() + 3,
            "Size-based generation should produce expressions closer to target size"
        );
    }

    #[test]
    fn test_expression_size_consistency() {
        // Test that our count_nodes matches the topology module's expression_size
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();
        
        for size in 1..15 {
            let expr = generate_random_expression_by_size(&lang, size, &mut rng);
            let counted = count_nodes(&expr);
            let topology_size = expression_size(&expr.to_expression());
            
            assert_eq!(
                counted, topology_size as usize,
                "Size counting mismatch for expression: {expr:?}"
            );
        }
    }
}
