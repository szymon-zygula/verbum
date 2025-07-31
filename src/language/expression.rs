use std::{borrow::Cow, collections::HashSet};

use crate::rewriting::{egraph::ClassId, egraph_matching::EGraphMatch};

use super::{Language, symbol::Symbol};

pub type VariableId = usize;

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Literal {
    UInt(u64),
    Int(i64),
}

/// An expression which does not admit variables
#[derive(Clone, PartialEq, Eq, Debug)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MixedExpression {
    Literal(Literal),
    Symbol(Symbol<MixedExpression>),
    Class(ClassId),
}

/// An expression with variables
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Literal(Literal),
    Symbol(Symbol<Expression>),
    Variable(VariableId),
}

impl Expression {
    pub const NICE_VARIABLES: [&str; 8] = ["x", "y", "z", "w", "α", "β", "γ", "δ"];
    const USE_NICE_VARIABLES: bool = true;

    pub fn variable_name(id: VariableId) -> String {
        if Self::USE_NICE_VARIABLES && id < Self::NICE_VARIABLES.len() {
            String::from(Self::NICE_VARIABLES[id])
        } else {
            format!("x{id}")
        }
    }

    pub fn nice_variable_id(name: &str) -> VariableId {
        Self::NICE_VARIABLES
            .iter()
            .position(|x| name == *x)
            .unwrap()
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
                .map(|child| child.variables())
                .flatten()
                .collect(),
            Expression::Variable(id) => HashSet::from([*id]),
        }
    }

    pub fn common_variables(&self, other: &Expression) -> HashSet<VariableId> {
        self.variables()
            .intersection(&other.variables())
            .copied()
            .collect()
    }

    pub fn to_mixed_expression(self, matching: &EGraphMatch) -> MixedExpression {
        match self {
            Expression::Literal(literal) => MixedExpression::Literal(literal),
            Expression::Symbol(symbol) => MixedExpression::Symbol(Symbol {
                id: symbol.id,
                children: symbol
                    .children
                    .into_iter()
                    .map(|child| child.to_mixed_expression(matching))
                    .collect(),
            }),
            Expression::Variable(variable_id) => {
                MixedExpression::Class(matching.class_variable(variable_id))
            }
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
}

impl AnyExpression for Expression {}
impl AnyExpression for VarFreeExpression {}

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

#[cfg(test)]
mod tests {
    use crate::language::expression::LangExpression;

    fn test_display(expression_str: &str) {
        let lang = crate::language::Language::math();
        let expr = lang.parse(expression_str).unwrap();
        let formatted = format!("{}", LangExpression::owned(expr, &lang));
        assert_eq!(expression_str, &formatted);
    }

    #[test]
    fn display_1() {
        test_display("x")
    }

    #[test]
    fn display_2() {
        test_display("(+ x y (sin y))");
    }

    #[test]
    fn display_3() {
        test_display("(+ x (- (- (* x y z w α β γ))) (sin (cos y)))");
    }
}
