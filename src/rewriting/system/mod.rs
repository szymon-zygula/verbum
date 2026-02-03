use crate::language::expression::AnyExpression;
use crate::language::{Language, expression::VarFreeExpression};
use crate::rewriting::egraph::saturation::{SaturationConfig, Saturator, SimpleSaturator};
use crate::rewriting::egraph::{Analysis, EGraph, matching::bottom_up::BottomUpMatcher};
use crate::rewriting::rule::Rule;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor, ser::SerializeStruct};
use std::error::Error;
use std::fmt;
use std::path::Path;

pub mod calculus;
pub mod dependency_graph;

// Helper struct for serializing/deserializing rules
#[derive(Serialize, Deserialize)]
struct SerializableRule {
    from: String,
    to: String,
}

// Helper struct for loading rules from separate JSON file
#[derive(Deserialize)]
struct RulesFile {
    rules: Vec<SerializableRule>,
}

pub struct TermRewritingSystem {
    language: Language,
    rules: Vec<Rule>,
}

impl TermRewritingSystem {
    pub fn new(language: Language, rules: Vec<Rule>) -> Self {
        Self { language, rules }
    }

    /// Load a TermRewritingSystem from a directory containing language.json and trs.json
    pub fn from_directory<P: AsRef<Path>>(dir_path: P) -> Result<Self, Box<dyn Error>> {
        let dir_path = dir_path.as_ref();

        // Load language
        let lang_path = dir_path.join("language.json");
        let language: Language = crate::utils::json::load_json(lang_path)?;

        // Load rules
        let rules_path = dir_path.join("trs.json");
        let rules_file: RulesFile = crate::utils::json::load_json(rules_path)?;

        // Parse rules using the language
        let rules: Vec<Rule> = rules_file
            .rules
            .into_iter()
            .map(|sr| Rule::from_strings(&sr.from, &sr.to, &language))
            .collect();

        Ok(Self::new(language, rules))
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn rules(&self) -> &Vec<Rule> {
        &self.rules
    }

    /// Build an e-graph from the provided expression and saturate it using the system's rules.
    /// Returns the saturated e-graph.
    pub fn rewrite<A: Analysis>(&self, expression: VarFreeExpression) -> EGraph<A> {
        let mut egraph = EGraph::<A>::from_expression(expression);
        let saturator = SimpleSaturator::new(Box::new(BottomUpMatcher));
        let _ = saturator.saturate(&mut egraph, &self.rules, &SaturationConfig::default());
        egraph
    }
}

impl Serialize for TermRewritingSystem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("TermRewritingSystem", 2)?;
        state.serialize_field("language", &self.language)?;

        let serializable_rules: Vec<SerializableRule> = self
            .rules
            .iter()
            .map(|rule| SerializableRule {
                from: format!("{}", rule.from().with_language(&self.language)),
                to: format!("{}", rule.to().with_language(&self.language)),
            })
            .collect();
        state.serialize_field("rules", &serializable_rules)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for TermRewritingSystem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Language,
            Rules,
        }

        struct TermRewritingSystemVisitor;

        impl<'de> Visitor<'de> for TermRewritingSystemVisitor {
            type Value = TermRewritingSystem;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct TermRewritingSystem")
            }

            fn visit_map<V>(self, mut map: V) -> Result<TermRewritingSystem, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut language: Option<Language> = None;
                let mut serializable_rules: Option<Vec<SerializableRule>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Language => {
                            if language.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language = Some(map.next_value()?);
                        }
                        Field::Rules => {
                            if serializable_rules.is_some() {
                                return Err(serde::de::Error::duplicate_field("rules"));
                            }
                            serializable_rules = Some(map.next_value()?);
                        }
                    }
                }

                let language =
                    language.ok_or_else(|| serde::de::Error::missing_field("language"))?;
                let serializable_rules =
                    serializable_rules.ok_or_else(|| serde::de::Error::missing_field("rules"))?;

                let rules: Vec<Rule> = serializable_rules
                    .into_iter()
                    .map(|sr| Rule::from_strings(&sr.from, &sr.to, &language))
                    .collect();

                Ok(TermRewritingSystem::new(language, rules))
            }
        }

        deserializer.deserialize_struct(
            "TermRewritingSystem",
            &["language", "rules"],
            TermRewritingSystemVisitor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::TermRewritingSystem;
    use crate::language::Language;
    use crate::language::expression::AnyExpression;
    use crate::macros::rules;
    use crate::rewriting::egraph::{DynEGraph, EGraph};

    #[test]
    fn trs_rewrite_classical() {
        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(* $0 2)" => "(<< $0 1)",
            "(* $0 1)" => "$0",
            "(/ (* $0 $1) $2)" => "(* $0 (/ $1 $2))",
            "(/ $0 $0)" => "1",
        );

        let expr = lang.parse_no_vars("(/ (* (sin 5) 2) 2)").unwrap();
        let trs = TermRewritingSystem::new(lang, rules);

        let egraph: EGraph<()> = trs.rewrite(expr);

        assert_eq!(egraph.class_count(), 5);
        assert_eq!(egraph.actual_node_count(), 9);
    }

    #[test]
    fn test_term_rewriting_system_serialization() {
        use crate::macros::rules;
        use serde_json;

        let lang = Language::simple_math();
        let rules = rules!(lang;
            "(* $0 2)" => "(<< $0 1)",
            "(* $0 1)" => "$0",
        );
        let trs = TermRewritingSystem::new(lang.clone(), rules);

        let serialized = serde_json::to_string(&trs).unwrap();
        let deserialized: TermRewritingSystem = serde_json::from_str(&serialized).unwrap();

        // Compare languages
        assert_eq!(trs.language(), deserialized.language());

        // Compare rules
        assert_eq!(trs.rules().len(), deserialized.rules().len());
        for i in 0..trs.rules().len() {
            assert_eq!(
                format!("{}", trs.rules()[i].from().with_language(trs.language())),
                format!(
                    "{}",
                    deserialized.rules()[i]
                        .from()
                        .with_language(deserialized.language())
                )
            );
            assert_eq!(
                format!("{}", trs.rules()[i].to().with_language(trs.language())),
                format!(
                    "{}",
                    deserialized.rules()[i]
                        .to()
                        .with_language(deserialized.language())
                )
            );
        }
    }
}
