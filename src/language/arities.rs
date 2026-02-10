//! Arities definition for language symbols.
//!
//! This module provides the [`Arities`] struct that maps symbols to their allowed arities.
//! An arity defines the number of children a symbol can have in an expression.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::symbol::SymbolId;

/// A mapping from symbol IDs to their allowed arities.
///
/// The `Arities` struct wraps a HashMap that maps each symbol to a list of integers
/// representing the valid number of children (arity) that symbol can have.
///
/// # Examples
///
/// ```
/// use verbum::language::arities::Arities;
///
/// let mut arities = Arities::default();
/// arities.set(0, vec![2]); // Symbol 0 (e.g., "+") has arity 2
/// arities.set(1, vec![1]); // Symbol 1 (e.g., "sin") has arity 1
/// arities.set(2, vec![0, 2]); // Symbol 2 can have arity 0 or 2
///
/// assert_eq!(arities.get(0), Some(&[2][..]));
/// ```
#[derive(Default, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arities {
    /// Maps symbol IDs to their allowed arities
    pub map: HashMap<SymbolId, Vec<usize>>,
}

impl Arities {
    /// Creates a new empty `Arities` mapping.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Sets the arities for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol_id` - The ID of the symbol
    /// * `arities` - A vector of allowed arities for this symbol
    pub fn set(&mut self, symbol_id: SymbolId, arities: Vec<usize>) {
        self.map.insert(symbol_id, arities);
    }

    /// Gets the arities for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol_id` - The ID of the symbol
    ///
    /// # Returns
    ///
    /// Returns `Some(&[usize])` if the symbol has defined arities, `None` otherwise
    #[must_use]
    pub fn get(&self, symbol_id: SymbolId) -> Option<&[usize]> {
        self.map.get(&symbol_id).map(|v| v.as_slice())
    }

    /// Gets the first (or only) arity for a symbol.
    ///
    /// This is a convenience method for cases where symbols have a single arity.
    ///
    /// # Arguments
    ///
    /// * `symbol_id` - The ID of the symbol
    ///
    /// # Returns
    ///
    /// Returns `Some(usize)` if the symbol has at least one arity defined, `None` otherwise
    #[must_use]
    pub fn get_first(&self, symbol_id: SymbolId) -> Option<usize> {
        self.map.get(&symbol_id).and_then(|v| v.first().copied())
    }

    /// Checks if a symbol has a specific arity.
    ///
    /// # Arguments
    ///
    /// * `symbol_id` - The ID of the symbol
    /// * `arity` - The arity to check
    ///
    /// # Returns
    ///
    /// Returns `true` if the symbol allows the given arity, `false` otherwise
    #[must_use]
    pub fn has_arity(&self, symbol_id: SymbolId, arity: usize) -> bool {
        self.map
            .get(&symbol_id)
            .map(|arities| arities.contains(&arity))
            .unwrap_or(false)
    }

    /// Returns the number of symbols with defined arities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Checks if the arities mapping is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl From<HashMap<SymbolId, Vec<usize>>> for Arities {
    fn from(map: HashMap<SymbolId, Vec<usize>>) -> Self {
        Self { map }
    }
}

impl From<HashMap<SymbolId, usize>> for Arities {
    /// Converts a HashMap with single arities to an Arities struct.
    ///
    /// Each symbol's single arity is wrapped in a Vec with one element.
    fn from(map: HashMap<SymbolId, usize>) -> Self {
        let arities_map = map
            .into_iter()
            .map(|(id, arity)| (id, vec![arity]))
            .collect();
        Self { map: arities_map }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arities_basic() {
        let mut arities = Arities::new();
        arities.set(0, vec![2]);
        arities.set(1, vec![1]);

        assert_eq!(arities.get(0), Some(&[2][..]));
        assert_eq!(arities.get(1), Some(&[1][..]));
        assert_eq!(arities.get(2), None);
    }

    #[test]
    fn test_arities_multiple_arities() {
        let mut arities = Arities::new();
        arities.set(0, vec![0, 2, 3]);

        assert_eq!(arities.get(0), Some(&[0, 2, 3][..]));
        assert!(arities.has_arity(0, 0));
        assert!(arities.has_arity(0, 2));
        assert!(arities.has_arity(0, 3));
        assert!(!arities.has_arity(0, 1));
    }

    #[test]
    fn test_arities_get_first() {
        let mut arities = Arities::new();
        arities.set(0, vec![2, 3]);
        arities.set(1, vec![1]);

        assert_eq!(arities.get_first(0), Some(2));
        assert_eq!(arities.get_first(1), Some(1));
        assert_eq!(arities.get_first(2), None);
    }

    #[test]
    fn test_arities_len() {
        let mut arities = Arities::new();
        assert_eq!(arities.len(), 0);
        assert!(arities.is_empty());

        arities.set(0, vec![2]);
        assert_eq!(arities.len(), 1);
        assert!(!arities.is_empty());

        arities.set(1, vec![1]);
        assert_eq!(arities.len(), 2);
    }

    #[test]
    fn test_arities_serialization() {
        let mut arities = Arities::new();
        arities.set(0, vec![2]);
        arities.set(1, vec![1]);
        arities.set(2, vec![0, 2]);

        // Serialize to JSON
        let serialized = serde_json::to_string(&arities).unwrap();
        
        // Deserialize from JSON
        let deserialized: Arities = serde_json::from_str(&serialized).unwrap();

        assert_eq!(arities, deserialized);
    }

    #[test]
    fn test_arities_from_hashmap_vec() {
        let mut map = HashMap::new();
        map.insert(0, vec![2]);
        map.insert(1, vec![1, 2]);

        let arities = Arities::from(map.clone());
        assert_eq!(arities.get(0), Some(&[2][..]));
        assert_eq!(arities.get(1), Some(&[1, 2][..]));
    }

    #[test]
    fn test_arities_from_hashmap_single() {
        let mut map = HashMap::new();
        map.insert(0, 2);
        map.insert(1, 1);

        let arities = Arities::from(map);
        assert_eq!(arities.get(0), Some(&[2][..]));
        assert_eq!(arities.get(1), Some(&[1][..]));
    }

    #[test]
    fn test_arities_json_roundtrip() {
        let mut arities = Arities::new();
        arities.set(0, vec![2]);
        arities.set(1, vec![1]);
        arities.set(5, vec![0, 1, 2]);

        let json = serde_json::to_string_pretty(&arities).unwrap();
        
        // Verify the JSON contains expected structure
        assert!(json.contains("\"map\""));
        assert!(json.contains("\"0\""));
        assert!(json.contains("\"1\""));
        assert!(json.contains("\"5\""));

        let parsed: Arities = serde_json::from_str(&json).unwrap();
        assert_eq!(arities, parsed);
    }
}
