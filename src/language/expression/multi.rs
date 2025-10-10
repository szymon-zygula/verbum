use super::{AnyExpression, Expression};
use crate::language::Language;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor, ser::SerializeStruct};
use std::fmt;

pub struct LangMultiExpression {
    language: Language,
    expressions: Vec<Expression>,
}

impl LangMultiExpression {
    pub fn new(language: Language, expressions: Vec<Expression>) -> Self {
        Self {
            language,
            expressions,
        }
    }

    pub fn language(&self) -> &Language {
        &self.language
    }

    pub fn expressions(&self) -> &Vec<Expression> {
        &self.expressions
    }
}

// Custom Serialize implementation for MultiExpressionContainer
impl Serialize for LangMultiExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("MultiExpressionContainer", 2)?;
        state.serialize_field("language", &self.language)?;

        let serializable_expressions: Vec<String> = self
            .expressions
            .iter()
            .map(|expr| format!("{}", expr.with_language(&self.language)))
            .collect();
        state.serialize_field("expressions", &serializable_expressions)?;
        state.end()
    }
}

// Custom Deserialize implementation for MultiExpressionContainer
impl<'de> Deserialize<'de> for LangMultiExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Language,
            Expressions,
        }

        struct MultiExpressionContainerVisitor;

        impl<'de> Visitor<'de> for MultiExpressionContainerVisitor {
            type Value = LangMultiExpression;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct MultiExpressionContainer")
            }

            fn visit_map<V>(self, mut map: V) -> Result<LangMultiExpression, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut language: Option<Language> = None;
                let mut serializable_expressions: Option<Vec<String>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Language => {
                            if language.is_some() {
                                return Err(serde::de::Error::duplicate_field("language"));
                            }
                            language = Some(map.next_value()?);
                        }
                        Field::Expressions => {
                            if serializable_expressions.is_some() {
                                return Err(serde::de::Error::duplicate_field("expressions"));
                            }
                            serializable_expressions = Some(map.next_value()?);
                        }
                    }
                }

                let language =
                    language.ok_or_else(|| serde::de::Error::missing_field("language"))?;
                let serializable_expressions = serializable_expressions
                    .ok_or_else(|| serde::de::Error::missing_field("expressions"))?;

                let expressions: Vec<Expression> = serializable_expressions
                    .into_iter()
                    .map(|s| language.parse(&s).map_err(serde::de::Error::custom))
                    .collect::<Result<Vec<Expression>, _>>()?;

                Ok(LangMultiExpression::new(language, expressions))
            }
        }

        deserializer.deserialize_struct(
            "MultiExpressionContainer",
            &["language", "expressions"],
            MultiExpressionContainerVisitor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::LangMultiExpression;
    use crate::language::expression::any::AnyExpression;
    use serde_json;

    #[test]
    fn test_multi_expression_container_serialization() {
        let lang = crate::language::Language::simple_math();
        let expressions = vec![
            lang.parse("(* $0 (+ $1 4))").unwrap(),
            lang.parse("(+ 1 1)").unwrap(),
        ];
        let container = LangMultiExpression::new(lang.clone(), expressions);
        let serialized = serde_json::to_string(&container).unwrap();
        let deserialized: LangMultiExpression = serde_json::from_str(&serialized).unwrap();

        assert_eq!(container.language(), deserialized.language());
        assert_eq!(
            container.expressions().len(),
            deserialized.expressions().len()
        );
        for i in 0..container.expressions().len() {
            assert_eq!(
                format!(
                    "{}",
                    container.expressions()[i].with_language(container.language())
                ),
                format!(
                    "{}",
                    deserialized.expressions()[i].with_language(deserialized.language())
                )
            );
        }
    }
}
