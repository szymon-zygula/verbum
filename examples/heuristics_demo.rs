//! Example demonstrating the usage of heuristics and helper functions.
//!
//! This example shows how to:
//! 1. Create a SinglyCompact value
//! 2. Get abelianized vectors for paths to variables
//! 3. Use the Heuristic trait to estimate rewrite distances

use verbum::{
    compact::SinglyCompact,
    language::{Language, arities::Arities},
    macros::rules,
    rewriting::{
        heuristic::{Heuristic, HeuristicConstructor, ILPHeuristic, ILPHeuristicConstructor},
        strings::get_path_abelian_vectors_to_variables,
        system::TermRewritingSystem,
    },
};
use std::collections::HashMap;

fn main() {
    println!("=== Verbum Heuristics Demo ===\n");

    // 1. Demonstrate SinglyCompact
    println!("1. SinglyCompact Examples:");
    let finite: SinglyCompact<u32> = SinglyCompact::Finite(42);
    let infinite: SinglyCompact<u32> = SinglyCompact::Infinite;
    
    println!("  Finite value: {}", finite);
    println!("  Infinite value: {}", infinite);
    println!("  Finite + Finite: {}", SinglyCompact::Finite(10) + SinglyCompact::Finite(20));
    println!("  Finite + Infinite: {}", SinglyCompact::Finite(10) + infinite);
    println!();

    // 2. Setup a simple language and TRS
    println!("2. Setting up Term Rewriting System:");
    let lang = Language::default()
        .add_symbol("+")  // id: 0
        .add_symbol("*"); // id: 1
    
    let mut arities_map = HashMap::new();
    arities_map.insert(0, 2); // + has arity 2
    arities_map.insert(1, 2); // * has arity 2
    let arities = Arities::from(arities_map);

    let rules = rules!(lang;
        "(+ $0 $1)" => "(* $0 $1)",
        "(* $0 $0)" => "(+ $0 $0)"
    );
    let trs = TermRewritingSystem::new(lang.clone(), rules);
    println!("  Language has {} symbols", lang.symbol_count());
    println!("  TRS has {} rules", trs.rules().len());
    println!();

    // 3. Get path abelian vectors
    println!("3. Path Abelianized Vectors:");
    let expr = lang.parse("(+ $0 (* $1 $0))").unwrap();
    println!("  Expression: (+ $0 (* $1 $0))");
    
    let string_lang = verbum::rewriting::strings::to_string_language(&lang, &arities);
    let path_vectors = get_path_abelian_vectors_to_variables(
        &expr,
        &lang,
        &string_lang,
        &arities,
    );
    
    println!("  Found {} paths to variables:", path_vectors.len());
    for pv in &path_vectors {
        println!("    Variable ${}: vector = {:?}", pv.variable_id, pv.vector.as_slice());
    }
    println!();

    // 4. Use ILP Heuristic
    println!("4. ILP Heuristic Example:");
    let target = lang.parse("(* $0 $1)").unwrap();
    let current = lang.parse("(+ $0 $1)").unwrap();
    
    println!("  Target expression: (* $0 $1)");
    println!("  Current expression: (+ $0 $1)");
    
    let heuristic = ILPHeuristic::new(&target, &trs, &arities);
    let distance = heuristic.lower_bound_dist(&current);
    
    match distance {
        SinglyCompact::Finite(d) => {
            println!("  Lower bound distance: {} rule applications", d);
        }
        SinglyCompact::Infinite => {
            println!("  Lower bound distance: ∞ (target unreachable)");
        }
    }
    println!();

    // 5. Use HeuristicConstructor
    println!("5. HeuristicConstructor Example:");
    let constructor = ILPHeuristicConstructor {
        arities: arities.clone(),
    };
    
    let heuristic_boxed = constructor.construct(&target, &trs);
    let distance2 = heuristic_boxed.lower_bound_dist(&current);
    
    println!("  Distance from constructor: {}", distance2);
    println!();

    println!("=== Demo Complete ===");
}
