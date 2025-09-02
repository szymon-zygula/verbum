use super::{Expression, Path, VarFreeExpression, path::SubexpressionPathIterator};
use crate::language::Language;
use std::borrow::Cow;

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

    fn iter_paths(&self) -> SubexpressionPathIterator<Self> {
        SubexpressionPathIterator::new(self)
    }

    fn iter_subexpressions(&self) -> SubexpressionIterator<Self> {
        SubexpressionIterator::new(self)
    }
}

#[derive(Clone, Debug)]
pub struct SubexpressionIterator<'e, E: AnyExpression> {
    expression: &'e E,
    path_iterator: SubexpressionPathIterator<'e, E>,
}

impl<'e, E: AnyExpression> SubexpressionIterator<'e, E> {
    pub fn new(expression: &'e E) -> Self {
        Self {
            expression,
            path_iterator: expression.iter_paths(),
        }
    }
}

impl<'e, E: AnyExpression> Iterator for SubexpressionIterator<'e, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        self.expression
            .subexpression(self.path_iterator.next()?.as_path())
            .cloned()
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

#[cfg(test)]
mod tests_subexpression_iterator {
    use super::*;
    use crate::language::Language;

    fn parse_expr(lang: &Language, s: &str) -> VarFreeExpression {
        lang.parse_no_vars(s).unwrap()
    }

    #[test]
    fn iterate_over_leaf() {
        let lang = Language::simple_math();
        let expr = parse_expr(&lang, "1");
        let mut iterator = SubexpressionIterator::new(&expr);
        assert_eq!(iterator.next(), Some(parse_expr(&lang, "1")));
        assert_eq!(iterator.next(), None);
    }

    #[test]
    fn iterate_over_simple_expression() {
        let lang = Language::simple_math();
        let expr = parse_expr(&lang, "(+ 1 2)");
        let subexpressions: Vec<_> = SubexpressionIterator::new(&expr).collect();
        assert_eq!(
            subexpressions,
            vec![
                parse_expr(&lang, "(+ 1 2)"),
                parse_expr(&lang, "1"),
                parse_expr(&lang, "2"),
            ]
        );
    }

    #[test]
    fn iterate_over_nested_expression() {
        let lang = Language::simple_math();
        let expr = parse_expr(&lang, "(+ 1 (* 2 3))");
        let subexpressions: Vec<_> = SubexpressionIterator::new(&expr).collect();
        assert_eq!(
            subexpressions,
            vec![
                parse_expr(&lang, "(+ 1 (* 2 3))"),
                parse_expr(&lang, "1"),
                parse_expr(&lang, "(* 2 3)"),
                parse_expr(&lang, "2"),
                parse_expr(&lang, "3"),
            ]
        );
    }
}
