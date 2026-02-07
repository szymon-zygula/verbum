//! Random expression generation for testing and benchmarking.
//!
//! This module provides utilities for generating random expressions,
//! useful for benchmarking term rewriting systems.

use rand::Rng;
use std::fmt;

use crate::language::{
    Language,
    expression::{Expression, Literal, VarFreeExpression, VariableId},
    symbol::Symbol,
};

/// Error type for random expression generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationError {
    /// Requested size is zero
    ZeroSize,
    /// No viable symbol-arity combinations for the requested size
    NoViableSymbolArities { remaining_size: usize },
    /// Remaining size cannot be zero during generation
    ZeroRemainingSize,
}

impl fmt::Display for GenerationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GenerationError::ZeroSize => {
                write!(f, "Cannot generate expression of size 0")
            }
            GenerationError::NoViableSymbolArities { remaining_size } => {
                write!(
                    f,
                    "No viable symbol-arity combinations for remaining size {}",
                    remaining_size
                )
            }
            GenerationError::ZeroRemainingSize => {
                write!(f, "Remaining size cannot be 0 during generation")
            }
        }
    }
}

impl std::error::Error for GenerationError {}

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

    /// Configuration for generating random variables.
    pub variable_config: Option<VariableGenerationConfig>,
}

/// Configuration for generating random variables.
#[derive(Clone, Debug)]
pub struct VariableGenerationConfig {
    /// Range of variable IDs that can be generated (min, max inclusive).
    pub variable_range: (VariableId, VariableId),

    /// Probability of generating a variable vs a literal at leaf nodes (0.0 to 1.0).
    /// 0.0 = always literal, 1.0 = always variable.
    pub variable_probability: f64,
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
    /// Creates a configuration from a language.
    ///
    /// Note: You must set symbol_arities explicitly for your language.
    /// By default, all symbols have no allowed arities.
    pub fn from_language(language: &Language) -> Self {
        let symbol_arities = vec![vec![]; language.symbol_count()];

        Self {
            symbol_arities,
            literal_config: LiteralGenerationConfig::default(),
            literal_probability: 0.3,
            variable_config: None,
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

    /// Sets the variable generation configuration.
    pub fn with_variable_config(mut self, variable_config: VariableGenerationConfig) -> Self {
        self.variable_config = Some(variable_config);
        self
    }
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
/// Returns a randomly generated variable-free expression
pub fn generate_random_expression_with_config(
    language: &Language,
    max_depth: usize,
    rng: &mut impl Rng,
    config: &RandomGenerationConfig,
) -> VarFreeExpression {
    // Create a config without variables
    let mut no_var_config = config.clone();
    no_var_config.variable_config = None;

    // Call the variable version and unwrap (safe since no variables)
    let expr = generate_random_expression_with_variables(language, max_depth, rng, &no_var_config);
    expr.without_variables()
        .expect("Expression should not contain variables when variable_config is None")
}

fn generate_random_literal(rng: &mut impl Rng, config: &LiteralGenerationConfig) -> Literal {
    // Check if we should use a common value
    if rng.gen_bool(config.common_value_probability) && !config.common_int_values.is_empty() {
        let value = config.common_int_values[rng.gen_range(0..config.common_int_values.len())];
        return Literal::Int(value);
    }

    // Generate a random value
    if rng.gen_bool(config.int_vs_uint_probability) {
        Literal::Int(rng.gen_range(config.int_range.0..=config.int_range.1))
    } else {
        Literal::UInt(rng.gen_range(config.uint_range.0..=config.uint_range.1))
    }
}

/// Generates a random expression with variables up to a maximum depth.
///
/// # Arguments
///
/// * `language` - The language defining available symbols
/// * `max_depth` - The maximum depth of the expression tree (number of nested levels)
/// * `rng` - The random number generator to use
/// * `config` - Configuration for random generation (must include variable_config)
///
/// # Returns
///
/// Returns a randomly generated expression that may contain variables
pub fn generate_random_expression_with_variables(
    language: &Language,
    max_depth: usize,
    rng: &mut impl Rng,
    config: &RandomGenerationConfig,
) -> Expression {
    generate_random_expression_with_variables_recursive(language, max_depth, rng, 0, config)
}

fn generate_random_expression_with_variables_recursive(
    language: &Language,
    max_depth: usize,
    rng: &mut impl Rng,
    current_depth: usize,
    config: &RandomGenerationConfig,
) -> Expression {
    if current_depth >= max_depth {
        // Generate a leaf node (literal or variable)
        return generate_random_leaf(rng, config);
    }

    // Decide whether to generate a leaf or a symbol
    if rng.gen_bool(config.literal_probability) || language.symbol_count() == 0 {
        // Generate a leaf based on configured probability, or if no symbols are defined
        generate_random_leaf(rng, config)
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

        let children: Vec<Expression> = (0..arity)
            .map(|_| {
                generate_random_expression_with_variables_recursive(
                    language,
                    max_depth,
                    rng,
                    current_depth + 1,
                    config,
                )
            })
            .collect();

        Expression::Symbol(Symbol {
            id: symbol_id,
            children,
        })
    }
}

fn generate_random_leaf(rng: &mut impl Rng, config: &RandomGenerationConfig) -> Expression {
    // Check if we should generate a variable
    if let Some(var_config) = &config.variable_config
        && rng.gen_bool(var_config.variable_probability) {
            let variable_id =
                rng.gen_range(var_config.variable_range.0..=var_config.variable_range.1);
            return Expression::Variable(variable_id);
        }

    // Generate a literal
    Expression::Literal(generate_random_literal(rng, &config.literal_config))
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
/// Returns a randomly generated variable-free expression with the target size, or an error
/// if it's impossible to generate an expression of that size.
///
/// # Errors
///
/// Returns `GenerationError` if the target size is 0 or if
/// no expression of the target size can be generated with the given configuration.
pub fn generate_random_expression_by_size_with_config(
    language: &Language,
    target_size: usize,
    rng: &mut impl Rng,
    config: &RandomGenerationConfig,
) -> Result<VarFreeExpression, GenerationError> {
    // Create a config without variables
    let mut no_var_config = config.clone();
    no_var_config.variable_config = None;

    // Call the variable version and unwrap (safe since no variables)
    let expr = generate_random_expression_by_size_with_variables(
        language,
        target_size,
        rng,
        &no_var_config,
    )?;
    Ok(expr
        .without_variables()
        .expect("Expression should not contain variables when variable_config is None"))
}

/// Generates a random expression with variables and an exact total size.
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
/// Returns a randomly generated expression with the target size, or an error
/// if it's impossible to generate an expression of that size.
///
/// # Errors
///
/// Returns `GenerationError` if the target size is 0 or if
/// no expression of the target size can be generated with the given configuration.
pub fn generate_random_expression_by_size_with_variables(
    language: &Language,
    target_size: usize,
    rng: &mut impl Rng,
    config: &RandomGenerationConfig,
) -> Result<Expression, GenerationError> {
    if target_size == 0 {
        return Err(GenerationError::ZeroSize);
    }

    if target_size == 1 {
        return Ok(generate_random_leaf(rng, config));
    }

    generate_random_expression_by_size_with_variables_recursive(language, target_size, rng, config)
}

fn generate_random_expression_by_size_with_variables_recursive(
    language: &Language,
    remaining_size: usize,
    rng: &mut impl Rng,
    config: &RandomGenerationConfig,
) -> Result<Expression, GenerationError> {
    if remaining_size == 0 {
        return Err(GenerationError::ZeroRemainingSize);
    }

    if remaining_size == 1 || language.symbol_count() == 0 {
        // Must generate a leaf node (literal or variable)
        return Ok(generate_random_leaf(rng, config));
    }

    // For size-based generation, try to find a symbol that can work
    // Collect all viable (symbol_id, arity) pairs
    let mut viable_choices: Vec<(usize, usize)> = Vec::new();

    for symbol_id in 0..language.symbol_count() {
        let allowed_arities = &config.symbol_arities[symbol_id];
        for &arity in allowed_arities {
            if arity > 0 && arity < remaining_size {
                viable_choices.push((symbol_id, arity));
            }
        }
    }

    if viable_choices.is_empty() {
        // Can't create any symbol with children
        return Err(GenerationError::NoViableSymbolArities { remaining_size });
    }

    // Randomly select a viable choice
    let (symbol_id, arity) = viable_choices[rng.gen_range(0..viable_choices.len())];

    // Distribute remaining size (minus 1 for current node) among children
    let size_for_children = remaining_size - 1;
    let child_sizes = distribute_size(size_for_children, arity, rng);

    let children: Result<Vec<Expression>, GenerationError> = child_sizes
        .iter()
        .map(|&child_size| {
            generate_random_expression_by_size_with_variables_recursive(
                language, child_size, rng, config,
            )
        })
        .collect();

    Ok(Expression::Symbol(Symbol {
        id: symbol_id,
        children: children?,
    }))
}

/// Distributes a total size among a number of children.
/// Returns a vector of sizes for each child.
fn distribute_size(total: usize, num_children: usize, rng: &mut impl Rng) -> Vec<usize> {
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
        LiteralGenerationConfig, RandomGenerationConfig,
        generate_random_expression_by_size_with_config, generate_random_expression_with_config,
    };
    use crate::language::{
        Language,
        expression::{Expression, VarFreeExpression},
        topology::expression_size,
    };

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

    fn default_math_config(lang: &Language) -> RandomGenerationConfig {
        let mut config = RandomGenerationConfig::from_language(lang);
        // Set up arities for simple_math symbols
        config.symbol_arities[lang.get_id("+")] = vec![2];
        config.symbol_arities[lang.get_id("-")] = vec![2];
        config.symbol_arities[lang.get_id("*")] = vec![2];
        config.symbol_arities[lang.get_id("/")] = vec![2];
        config.symbol_arities[lang.get_id("sin")] = vec![1];
        config.symbol_arities[lang.get_id("cos")] = vec![1];
        config.symbol_arities[lang.get_id("<<")] = vec![2];
        config.symbol_arities[lang.get_id(">>")] = vec![2];
        config
    }

    #[test]
    fn test_generate_random_expression() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();
        let config = default_math_config(&lang);

        for depth in 1..10 {
            let expr = generate_random_expression_with_config(&lang, depth, &mut rng, &config);
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
        let config = default_math_config(&lang);

        // Maximum acceptable deviation from target size (as a fraction)
        const MAX_SIZE_DEVIATION_FRACTION: usize = 3; // 1/3 = 33%

        for target_size in 1..20 {
            let expr = generate_random_expression_by_size_with_config(
                &lang,
                target_size,
                &mut rng,
                &config,
            )
            .unwrap();
            let actual_size = count_nodes(&expr);

            println!(
                "Generated expression (target size {target_size}, actual size {actual_size}): {expr:?}"
            );

            // The actual size should be reasonably close to the target
            // Allow some deviation due to randomness and arity constraints
            let tolerance = if target_size < 5 {
                1
            } else {
                target_size / MAX_SIZE_DEVIATION_FRACTION
            };
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
    fn test_generate_by_size_error_on_zero() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();
        let config = default_math_config(&lang);

        let result = generate_random_expression_by_size_with_config(&lang, 0, &mut rng, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_arity_configuration() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();

        // Create a configuration where + only accepts 3 children
        let plus_id = lang.get_id("+");
        let config = default_math_config(&lang).with_symbol_arities(plus_id, vec![3]);

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

        let config = default_math_config(&lang)
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
        let config = default_math_config(&lang);

        // Generate expressions with same parameter value using both methods
        let depth_expr = generate_random_expression_with_config(&lang, 5, &mut rng, &config);
        let size_expr =
            generate_random_expression_by_size_with_config(&lang, 5, &mut rng, &config).unwrap();

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
        let config = default_math_config(&lang);

        for size in 1..15 {
            let expr =
                generate_random_expression_by_size_with_config(&lang, size, &mut rng, &config)
                    .unwrap();
            let counted = count_nodes(&expr);
            let topology_size = expression_size(&expr.to_expression());

            assert_eq!(
                counted, topology_size as usize,
                "Size counting mismatch for expression: {expr:?}"
            );
        }
    }

    #[test]
    fn test_generate_expression_with_variables() {
        use super::{VariableGenerationConfig, generate_random_expression_with_variables};

        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();

        let variable_config = VariableGenerationConfig {
            variable_range: (0, 3),
            variable_probability: 0.5,
        };

        let config = default_math_config(&lang).with_variable_config(variable_config);

        let mut variable_count = 0;
        let total_generations = 50;

        for _ in 0..total_generations {
            let expr = generate_random_expression_with_variables(&lang, 4, &mut rng, &config);
            println!("Generated expression with variables: {expr:?}");

            // Check if expression contains variables
            if contains_variables(&expr) {
                variable_count += 1;
            }

            // Verify all variables are in the allowed range
            verify_variable_range(&expr, 0, 3);
        }

        // With 50% probability, we should see some expressions with variables
        assert!(
            variable_count > 5,
            "Expected more expressions with variables, got {variable_count}/{total_generations}"
        );
    }

    fn contains_variables(expr: &Expression) -> bool {
        match expr {
            Expression::Variable(_) => true,
            Expression::Symbol(symbol) => symbol.children.iter().any(contains_variables),
            Expression::Literal(_) => false,
        }
    }

    fn verify_variable_range(expr: &Expression, min: usize, max: usize) {
        match expr {
            Expression::Variable(id) => {
                assert!(
                    *id >= min && *id <= max,
                    "Variable {} is outside allowed range [{}, {}]",
                    id,
                    min,
                    max
                );
            }
            Expression::Symbol(symbol) => {
                for child in &symbol.children {
                    verify_variable_range(child, min, max);
                }
            }
            Expression::Literal(_) => {}
        }
    }

    #[test]
    fn test_no_variables_when_config_not_set() {
        use super::generate_random_expression_with_variables;

        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();
        let config = default_math_config(&lang); // No variable config

        for _ in 0..20 {
            let expr = generate_random_expression_with_variables(&lang, 3, &mut rng, &config);
            assert!(
                !contains_variables(&expr),
                "Should not generate variables when variable_config is None"
            );
        }
    }
}
