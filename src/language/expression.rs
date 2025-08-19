use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde::{Deserializer, Serializer, de::Visitor, ser::SerializeStruct};
use std::fmt;

use crate::rewriting::egraph::{ClassId, matching::EGraphMatch};

use super::{Language, symbol::Symbol};

pub type VariableId = usize;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct OwnedPath(Vec<usize>);

impl OwnedPath {
    pub fn as_ref(&self) -> Path<'_> {
        Path(&self.0)
    }

    pub fn child(&self) -> Path<'_> {
        self.as_ref().child()
    }

    pub fn head(&self) -> Option<usize> {
        self.0.first().copied()
    }

    pub fn push(&mut self, location: usize) {
        self.0.push(location)
    }

    pub fn pop(&mut self) -> Option<usize> {
        self.0.pop()
    }
}

#[derive(Clone, Debug)]
pub struct Path<'p>(&'p [usize]);

impl<'p> Path<'p> {
    pub fn child(&self) -> Self {
        Path(&self.0[1..])
    }

    pub fn head(&self) -> Option<usize> {
        self.0.first().copied()
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Literal {
    UInt(u64),
    Int(i64),
}

/// An expression which does not admit variables
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum VarFreeExpression {
    Literal(Literal),
    Symbol(Symbol<VarFreeExpression>),
}

impl VarFreeExpression {
    /// Panics if `self` is not a symbol represented by `name` in `lang`. Returns its children otherwise.
    /// Just for doing tests.
    pub fn expect_symbol<'a>(&'a self, name: &str, lang: &Language) -> &'a Vec<Self> {
        let Self::Symbol(Symbol { id, children }) = self else {
            panic!("Expected symbol but did not find a symbol")
        };

        assert_eq!(lang.get_id(name), *id);

        children
    }
}

/// An expression which consists of concrete elements as well as class IDs.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MixedExpression {
    Literal(Literal),
    Symbol(Symbol<MixedExpression>),
    Class(ClassId),
}

/// An expression with variables
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Symbol(Symbol<Expression>),
    Variable(VariableId),
}

impl Expression {
    pub fn variable_name(id: VariableId) -> String {
        format!("${id}")
    }

    /// Returns `None` if `self` contains variables, or a corresponding `VarFreeExpression` if it does not
    pub fn without_variables(&self) -> Option<VarFreeExpression> {
        match self {
            Expression::Variable(_) => None,
            Expression::Symbol(Symbol { id, children }) => {
                let mut children_no_vars = Vec::with_capacity(children.len());

                for child in children {
                    children_no_vars.push(child.without_variables()?);
                }

                Some(VarFreeExpression::Symbol(Symbol {
                    id: *id,
                    children: children_no_vars,
                }))
            }
            Expression::Literal(literal) => Some(VarFreeExpression::Literal(literal.clone())),
        }
    }

    /// Panics if `self` is not a variable, returns its ID otherwise. Just for doing tests.
    pub fn expect_variable(&self) -> VariableId {
        let Self::Variable(id) = self else {
            panic!("Expected variable but did not find it")
        };

        *id
    }

    /// Panics if `self` is not a symbol represented by `name` in `lang`. Returns its children otherwise.
    /// Just for doing tests.
    pub fn expect_symbol<'a>(&'a self, name: &str, lang: &Language) -> &'a Vec<Self> {
        let Self::Symbol(Symbol { id, children }) = self else {
            panic!("Expected symbol but did not find a symbol")
        };

        assert_eq!(lang.get_id(name), *id);

        children
    }

    pub fn variables(&self) -> HashSet<VariableId> {
        match self {
            Expression::Literal(_) => HashSet::new(),
            Expression::Symbol(symbol) => symbol
                .children
                .iter()
                .flat_map(|child| child.variables())
                .collect(),
            Expression::Variable(id) => HashSet::from([*id]),
        }
    }

    pub fn variables_vec(&self) -> Vec<VariableId> {
        self.variables().into_iter().collect_vec()
    }

    pub fn common_variables(&self, other: &Expression) -> HashSet<VariableId> {
        self.variables()
            .intersection(&other.variables())
            .copied()
            .collect()
    }

    pub fn mixed_expression(self, matching: &EGraphMatch) -> MixedExpression {
        match self {
            Expression::Literal(literal) => MixedExpression::Literal(literal),
            Expression::Symbol(symbol) => MixedExpression::Symbol(Symbol {
                id: symbol.id,
                children: symbol
                    .children
                    .into_iter()
                    .map(|child| child.mixed_expression(matching))
                    .collect(),
            }),
            Expression::Variable(variable_id) => {
                MixedExpression::Class(matching.class_variable(variable_id))
            }
        }
    }

    /// Finds all variables in the expression together with their paths
    pub fn find_all_variables(&self) -> HashMap<VariableId, Vec<OwnedPath>> {
        let mut vars = HashMap::new();
        self.find_all_variables_impl(&mut OwnedPath::default(), &mut vars);
        vars
    }

    fn find_all_variables_impl(
        &self,
        current_path: &mut OwnedPath,
        vars: &mut HashMap<VariableId, Vec<OwnedPath>>,
    ) {
        match self {
            Expression::Variable(variable_id) => {
                vars.entry(*variable_id)
                    .or_default()
                    .push(current_path.clone());
            }
            Expression::Symbol(symbol) => {
                for (i, child) in symbol.children.iter().enumerate() {
                    current_path.push(i);
                    child.find_all_variables_impl(current_path, vars);
                    current_path.pop();
                }
            }
            Expression::Literal(_) => {}
        }
    }
}

pub trait AnyExpression: Clone + 'static {
    fn with_language<'e, 'l>(&'e self, language: &'l Language) -> LangExpression<'e, 'l, Self> {
        LangExpression {
            expression: Cow::Borrowed(self),
            language,
        }
    }

    fn children(&self) -> Option<Vec<&Self>>;

    fn subexpression<'e>(&'e self, path: Path) -> Option<&'e Self> {
        if let Some(head) = path.head() {
            self.children()
                .and_then(|children| children.get(head)?.subexpression(path.child()))
        } else {
            Some(self)
        }
    }
}

impl AnyExpression for Expression {
    fn children(&self) -> Option<Vec<&Self>> {
        match self {
            Expression::Symbol(symbol) => Some(symbol.children.iter().collect()),
            _ => None,
        }
    }
}

impl AnyExpression for VarFreeExpression {
    fn children(&self) -> Option<Vec<&Self>> {
        match self {
            VarFreeExpression::Symbol(symbol) => Some(symbol.children.iter().collect()),
            _ => None,
        }
    }
}

pub struct LangExpression<'e, 'l, E: AnyExpression> {
    pub expression: Cow<'e, E>,
    pub language: &'l Language,
}

impl<'e, 'l, E: AnyExpression> LangExpression<'e, 'l, E> {
    fn owned(expression: E, language: &'l Language) -> Self {
        LangExpression::<'static, 'l, E> {
            expression: Cow::Owned(expression),
            language,
        }
    }

    fn borrowed(expression: &'e E, language: &'l Language) -> Self {
        Self {
            expression: Cow::Borrowed(expression),
            language,
        }
    }
}

impl<'e, 'l> std::fmt::Display for LangExpression<'e, 'l, Expression> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.expression.as_ref() {
            Expression::Variable(id) => write!(f, "{}", Expression::variable_name(*id)),
            Expression::Symbol(symbol) => symbol.fmt(f, self.language),
            Expression::Literal(literal) => match literal {
                Literal::UInt(uint) => write!(f, "{uint}u"),
                Literal::Int(int) => write!(f, "{int}"),
            },
        }
    }
}

impl<'e, 'l> std::fmt::Display for LangExpression<'e, 'l, VarFreeExpression> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.expression.as_ref() {
            VarFreeExpression::Symbol(symbol) => symbol.fmt(f, self.language),
            VarFreeExpression::Literal(literal) => match literal {
                Literal::UInt(uint) => write!(f, "{uint}u"),
                Literal::Int(int) => write!(f, "{int}"),
            },
        }
    }
}

impl std::fmt::Display for VarFreeExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lang = Language::simple_math(); // This is a hack, ideally language should be passed
        let lang_expr = LangExpression::borrowed(self, &lang);
        write!(f, "{lang_expr}")
    }
}

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
    use super::{Expression, LangMultiExpression, VarFreeExpression};
    use crate::language::expression::{AnyExpression, LangExpression};
    use serde_json;

    fn test_display(expression_str: &str) {
        let lang = crate::language::Language::simple_math();
        let expr = lang.parse(expression_str).unwrap();
        let formatted = format!("{}", LangExpression::owned(expr, &lang));
        assert_eq!(expression_str, &formatted);
    }

    #[test]
    fn display_1() {
        test_display("$0")
    }

    #[test]
    fn display_2() {
        test_display("(+ $0 $1 (sin $1))");
    }

    #[test]
    fn display_3() {
        test_display("(+ $0 (- (- (* $0 $1 $2 $3 $4 $5 $6))) (sin (cos $1)))");
    }

    #[test]
    fn test_var_free_expression_serialization() {
        let lang = crate::language::Language::simple_math();
        let expr = lang.parse_no_vars("(* 2 (+ 3 4))").unwrap();
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: VarFreeExpression = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }

    #[test]
    fn test_expression_serialization() {
        let lang = crate::language::Language::simple_math();
        let expr = lang.parse("(* $0 (+ $1 4))").unwrap();
        let serialized = serde_json::to_string(&expr).unwrap();
        let deserialized: Expression = serde_json::from_str(&serialized).unwrap();
        assert_eq!(expr, deserialized);
    }

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
