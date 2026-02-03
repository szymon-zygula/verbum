//! Convenience macros for defining rewrite rules and term rewriting systems.
//!
//! This module provides macros that simplify the declaration of rewrite rules
//! and term rewriting systems with a more concise syntax.

// Macros to simplify rule and TRS declarations

/// Creates a vector of rewrite rules from string patterns.
///
/// # Syntax
///
/// - `rules!(lang; "pattern" => "replacement", ...)` - Creates unidirectional rules
/// - `rules!(lang; "pattern" <=> "replacement", ...)` - Creates bidirectional rules
///
/// # Examples
///
/// ```no_run
/// # use verbum::macros::rules;
/// # use verbum::language::Language;
/// let lang = Language::simple_math();
/// let rules = rules!(lang;
///     "(* $0 1)" => "$0",
///     "(+ $0 0)" => "$0"
/// );
/// ```
macro_rules! rules {
    ($lang:expr; ) => {
        Vec::<$crate::rewriting::rule::Rule>::new()
    };
    ($lang:expr; $from:expr => $to:expr $(, $($rest:tt)*)? ) => {{
        let mut v = Vec::new();
        v.push($crate::rewriting::rule::Rule::from_strings($from, $to, &$lang));
        $( v.extend(crate::macros::rules!($lang; $($rest)*)); )?
        v
    }};
    ($lang:expr; $from:tt <=> $to:tt $(, $($rest:tt)*)? ) => {{
        let mut v = Vec::new();
        v.push($crate::rewriting::rule::Rule::from_strings($from, $to, &$lang));
        v.push($crate::rewriting::rule::Rule::from_strings($to, $from, &$lang));
        $( v.extend(crate::macros::rules!($lang; $($rest)*)); )?
        v
    }};
}

/// Creates a complete term rewriting system with symbols and rules.
///
/// # Syntax
///
/// ```no_run
/// # use verbum::macros::trs;
/// let system = trs!(
///     symbols: ["+", "*", "0", "1"],
///     rules:
///         "(* $0 1)" => "$0",
///         "(+ $0 0)" => "$0"
/// );
/// ```
macro_rules! trs {
    (symbols: [ $( $sym:expr ),* $(,)? ], rules: $($rules:tt)* ) => {{
        let mut lang = $crate::language::Language::default();
        $( lang = lang.add_symbol($sym); )*
        let rules = crate::macros::rules!(lang; $($rules)* );
        crate::rewriting::system::TermRewritingSystem::new(lang, rules)
    }};
}

pub(crate) use rules;
pub(crate) use trs;
