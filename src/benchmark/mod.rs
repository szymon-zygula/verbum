//! Benchmarking utilities for term rewriting systems.
//!
//! This module provides tools for benchmarking the performance of term rewriting
//! systems, including:
//! - Saturation benchmarks
//! - Reachability analysis
//! - Result formatting (CSV, pretty tables)
//! - Random expression generation

pub mod csv_output;
pub mod formatter;
pub mod pretty_printing;
pub mod random_generation;
pub mod reachability;
pub mod saturation;

pub use saturation::{BenchmarkConfig, Outcome, OutcomeFormatter, benchmark};

pub use reachability::{
    ReachabilityOutcome,
    benchmark_pairs_with_scheduler as reachability_benchmark_pairs_with_scheduler,
};

pub use random_generation::{
    GenerationError, LiteralGenerationConfig, RandomGenerationConfig, VariableGenerationConfig,
    generate_random_expression_by_size_with_config,
    generate_random_expression_by_size_with_variables, generate_random_expression_with_config,
    generate_random_expression_with_variables,
};
