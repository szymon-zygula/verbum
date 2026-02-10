//! Language definition for symbolic expressions.
//!
//! This module provides the core [`Language`] struct that defines a symbolic language
//! for term rewriting. A language consists of a set of symbols (operators and functions)
//! that can be used to construct expressions.
//!
//! # Examples
//!
//! ```no_run
//! # use verbum::language::Language;
//! let lang = Language::default()
//!     .add_symbol("+")
//!     .add_symbol("*");
//! ```

use serde::{Deserialize, Serialize};
use symbol::SymbolId;

pub mod arities;
pub mod expression;
pub mod parsing;
pub mod symbol;
pub mod topology;

/// A symbolic language definition.
///
/// The `Language` struct represents a collection of symbols that can be used to
/// build expressions. Symbols are identified by their unique IDs, which are
/// assigned based on the order they are added to the language. Symbols do not
/// have a fixed arity and can have any number of children.
#[derive(Default, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Language {
    // Runtime constraints specifying number of inputs/outputs?
    symbols: Vec<String>,
    // constraints: Vec<Vec<Constraint>>
}

impl Language {
    /// Adds a new symbol to the language.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol to add
    ///
    /// # Returns
    ///
    /// Returns the language with the new symbol added
    pub fn add_symbol(mut self, name: &str) -> Self {
        self.symbols.push(String::from(name));
        self
    }

    /// Gets the name of a symbol by its ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the symbol
    ///
    /// # Returns
    ///
    /// Returns the string name of the symbol
    pub fn get_symbol(&self, id: SymbolId) -> &str {
        &self.symbols[id]
    }

    /// Gets the ID of a symbol by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol to look up
    ///
    /// # Returns
    ///
    /// Returns the unique identifier of the symbol
    ///
    /// # Panics
    ///
    /// Panics if the symbol is not present in the language
    pub fn get_id(&self, name: &str) -> SymbolId {
        self.try_get_id(name)
            .unwrap_or_else(|| panic!("Symbol not present in the language: {name}"))
    }

    /// Tries to get the ID of a symbol by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol to look up
    ///
    /// # Returns
    ///
    /// Returns `Some(id)` if the symbol exists, `None` otherwise
    pub fn try_get_id(&self, name: &str) -> Option<SymbolId> {
        self.symbols.iter().position(|x| name == x)
    }

    /// Creates a simple mathematical language with common operators.
    ///
    /// The language includes: `+`, `-`, `*`, `/`, `sin`, `cos`, `<<`, `>>`
    ///
    /// # Returns
    ///
    /// Returns a language with standard mathematical symbols
    pub fn simple_math() -> Self {
        Self::default()
            .add_symbol("+")
            .add_symbol("-")
            .add_symbol("*")
            .add_symbol("/")
            .add_symbol("sin")
            .add_symbol("cos")
            .add_symbol("<<")
            .add_symbol(">>")
    }

    /// Returns the total number of symbols in the language.
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }
}

#[cfg(test)]
mod tests {
    use super::Language;

    #[test]
    fn symbols() {
        let lang = Language::default().add_symbol("f").add_symbol("g");

        assert_eq!("f", lang.get_symbol(0));
        assert_eq!("g", lang.get_symbol(1));
        assert!(lang.try_get_id("h").is_none());
    }

    #[test]
    fn test_language_serialization() {
        let lang = Language::simple_math();
        let serialized = serde_json::to_string(&lang).unwrap();
        let deserialized: Language = serde_json::from_str(&serialized).unwrap();

        assert_eq!(lang.symbols, deserialized.symbols);
    }
}
