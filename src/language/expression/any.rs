use std::borrow::Cow;
use super::{Expression, Path, VarFreeExpression};
use crate::language::Language;

pub trait AnyExpression: Clone + PartialEq + Eq + 'static {
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
    pub fn owned(expression: E, language: &'l Language) -> Self {
        LangExpression::<'static, 'l, E> {
            expression: Cow::Owned(expression),
            language,
        }
    }

    pub fn borrowed(expression: &'e E, language: &'l Language) -> Self {
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
                super::Literal::UInt(uint) => write!(f, "{uint}u"),
                super::Literal::Int(int) => write!(f, "{int}"),
            },
        }
    }
}

impl<'e, 'l> std::fmt::Display for LangExpression<'e, 'l, VarFreeExpression> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.expression.as_ref() {
            VarFreeExpression::Symbol(symbol) => symbol.fmt(f, self.language),
            VarFreeExpression::Literal(literal) => match literal {
                super::Literal::UInt(uint) => write!(f, "{uint}u"),
                super::Literal::Int(int) => write!(f, "{int}"),
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

#[cfg(test)]
mod tests {
    use super::LangExpression;

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
}
