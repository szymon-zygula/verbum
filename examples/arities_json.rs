/// Example demonstrating JSON serialization of Arities.
///
/// This example shows how to serialize and deserialize the Arities struct to/from JSON.
use std::collections::HashMap;
use verbum::language::arities::Arities;

fn main() {
    println!("=== Arities JSON Serialization Example ===\n");

    // Create an Arities structure
    let mut arities_map = HashMap::new();
    arities_map.insert(0, vec![2]); // Symbol 0 (e.g., "+") has arity 2
    arities_map.insert(1, vec![1]); // Symbol 1 (e.g., "sin") has arity 1
    arities_map.insert(2, vec![0]); // Symbol 2 (e.g., "x") has arity 0
    arities_map.insert(3, vec![2, 3]); // Symbol 3 can have arity 2 or 3

    let arities = Arities::from(arities_map);

    println!("Original Arities structure:");
    for symbol_id in 0..=3 {
        if let Some(arity_vec) = arities.get(symbol_id) {
            println!("  Symbol {}: arities {:?}", symbol_id, arity_vec);
        }
    }
    println!();

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&arities).unwrap();
    println!("Serialized to JSON:");
    println!("{}", json);
    println!();

    // Deserialize from JSON
    let deserialized: Arities = serde_json::from_str(&json).unwrap();
    println!("Deserialized Arities structure:");
    for symbol_id in 0..=3 {
        if let Some(arity_vec) = deserialized.get(symbol_id) {
            println!("  Symbol {}: arities {:?}", symbol_id, arity_vec);
        }
    }
    println!();

    // Verify they are equal
    if arities == deserialized {
        println!("✓ Serialization and deserialization successful!");
    } else {
        println!("✗ Serialization or deserialization failed!");
    }
    println!();

    // Example with conversion from single-arity HashMap
    println!("=== Converting from single-arity HashMap ===");
    let mut single_arity_map = HashMap::new();
    single_arity_map.insert(0, 2);
    single_arity_map.insert(1, 1);
    single_arity_map.insert(2, 0);

    let arities_from_single = Arities::from(single_arity_map);
    println!("Arities created from single-arity HashMap:");
    let json_from_single = serde_json::to_string_pretty(&arities_from_single).unwrap();
    println!("{}", json_from_single);
}
