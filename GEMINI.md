# Verbum: A Term Rewriting System in Rust

## Project Overview

Verbum is a powerful and flexible term rewriting system implemented in Rust. It provides a framework for defining symbolic languages, applying rewrite rules, and simplifying expressions. At its core, Verbum utilizes an e-graph data structure for efficient equality saturation, enabling sophisticated program optimizations and symbolic computations.

The project is structured into several key modules:

*   **`language`**: This module provides the building blocks for defining custom symbolic languages. It includes components for parsing expressions, defining symbols, and specifying the topology of the language.
*   **`rewriting`**: This module contains the core term rewriting engine. It implements the e-graph data structure, the saturation algorithm, and the logic for applying rewrite rules.
*   **`benchmark`**: This module offers tools for benchmarking the performance of the term rewriting system, allowing users to measure the efficiency of their rewrite rules and configurations.

## Building and Running

As a standard Rust project, Verbum can be built and run using Cargo:

*   **Build the project:**
    ```bash
    cargo build
    ```

*   **Run the main application:**
    ```bash
    cargo run
    ```

*   **Run tests:**
    ```bash
    cargo test
    ```

These commands should be used after completing any task to make sure everything works.

## Development Conventions

The Verbum codebase follows standard Rust conventions and is formatted using `rustfmt`. The project emphasizes a clean and modular architecture, with clear separation of concerns between the language definition, rewriting engine, and benchmarking components.

### Key Concepts

*   **Term Rewriting System (TRS):** A TRS consists of a set of rewrite rules that are used to transform expressions into simpler or more canonical forms.
*   **E-Graph:** An e-graph is a data structure that efficiently represents a set of equivalent expressions. It is used to avoid redundant computations and to explore the search space of possible rewrites more effectively.
*   **Equality Saturation:** This is a process of repeatedly applying rewrite rules to an e-graph until no new equivalent expressions can be found. This process allows Verbum to find the most optimal representation of an expression according to a given cost function.

### Example Usage

The `main.rs` file provides a simple example of how to use Verbum. It defines a basic mathematical language, a set of rewrite rules for simplifying arithmetic expressions, and then runs the term rewriting system on a sample expression.

```rust
use language::Language;
use rewriting::{
    egraph::{
        extraction::{SimpleExtractor, children_cost_sum},
        saturation::SaturationConfig,
    },
    system::TermRewritingSystem,
};

fn main() {
    let lang = Language::simple_math();
    let rules = rules!(lang;
        "(* $0 2)" => "(<< $0 1)",
        "(* $0 1)" => "$0",
        "(/ (* $0 $1) $2)" => "(* $0 (/ $1 $2))",
        "(/ $0 $0)" => "1",
    );

    let trs = TermRewritingSystem::new(lang.clone(), rules);
    let expressions = vec![
        lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap(),
        lang.parse_no_vars("(+ 1 1)").unwrap(),
    ];

    // ... benchmarking and output ...
}
```

### Other

Tests should be implemented in the same file as the thing they are testing, wrapped in `mod tests`.
