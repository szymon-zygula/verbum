// Macros to simplify rule and TRS declarations

#[macro_export]
macro_rules! rules {
    ($lang:expr; ) => {
        Vec::<$crate::rewriting::rule::Rule>::new()
    };
    ($lang:expr; $from:expr => $to:expr $(, $($rest:tt)*)? ) => {{
        let mut v = vec![$crate::rewriting::rule::Rule::from_strings($from, $to, &$lang)];
        $( v.extend(crate::macros::rules!($lang; $($rest)*)); )?
        v
    }};
    ($lang:expr; $from:tt <=> $to:tt $(, $($rest:tt)*)? ) => {{
        let mut v = vec![
            $crate::rewriting::rule::Rule::from_strings($from, $to, &$lang)
            $crate::rewriting::rule::Rule::from_strings($to, $from, &$lang)
        ];
        $( v.extend(crate::macros::rules!($lang; $($rest)*)); )?
        v
    }};
}

#[macro_export]
macro_rules! trs {
    (symbols: [ $( $sym:expr ),* $(,)? ], rules: $($rules:tt)* ) => {{
        let mut lang = $crate::language::Language::default();
        $( lang = lang.add_symbol($sym); )*
        let rules = crate::macros::rules!(lang; $($rules)* );
        crate::rewriting::system::TermRewritingSystem::new(lang, rules)
    }};
}

pub use rules;
pub use trs;
