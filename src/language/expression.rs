use std::borrow::Cow;

use super::{Language, SymbolId};

pub type VariableId = usize;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expression {
    Variable(VariableId),
    Symbol {
        id: SymbolId,
        children: Vec<Expression>,
    },
}

impl Expression {
    pub const NICE_VARIABLES: [&str; 8] = ["x", "y", "z", "w", "α", "β", "γ", "δ"];
    const USE_NICE_VARIABLES: bool = true;

    pub fn variable_name(id: VariableId) -> String {
        if Self::USE_NICE_VARIABLES && id < Self::NICE_VARIABLES.len() {
            String::from(Self::NICE_VARIABLES[id])
        } else {
            format!("x{}", id)
        }
    }

    pub fn nice_variable_id(name: &str) -> VariableId {
        Self::NICE_VARIABLES
            .iter()
            .position(|x| name == *x)
            .unwrap()
    }

    pub fn with_language<'e, 'l>(&'e self, language: &'l Language) -> LangExpression<'e, 'l> {
        LangExpression {
            expression: Cow::Borrowed(self),
            language,
        }
    }
}

pub struct LangExpression<'e, 'l> {
    expression: Cow<'e, Expression>,
    language: &'l Language,
}

impl<'e, 'l> std::fmt::Display for LangExpression<'e, 'l> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.expression.as_ref() {
            Expression::Variable(id) => write!(f, "{}", Expression::variable_name(*id)),
            Expression::Symbol { id, children } => {
                write!(f, "({}", self.language.get_symbol(*id))?;
                for child in children {
                    write!(f, "{}", child.with_language(self.language))?;
                }
                write!(f, ")")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Expression;

    #[test]
    fn display() {
        let lang = crate::language::Language::math();
    }
}
