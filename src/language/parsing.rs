use super::{Language, expression::Expression};
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "language/grammar.pest"]
struct LanguageParser;

impl Language {
    fn parse_expression(&self, pair: Pair<Rule>) -> Expression {
        match pair.as_rule() {
            Rule::standalone_expression | Rule::expression => {
                self.parse_expression(pair.into_inner().next().unwrap())
            }
            Rule::nice_variable => {
                Expression::Variable(Expression::nice_variable_id(pair.as_str()))
            }
            Rule::numbered_variable => {
                Expression::Variable(pair.into_inner().next().unwrap().as_str().parse().unwrap())
            }
            Rule::variable => self.parse_expression(pair.into_inner().next().unwrap()),
            Rule::symbol_call => {
                let mut inner = pair.into_inner();
                let id = self.get_id(inner.next().unwrap().as_str());
                let children = inner.map(|e| self.parse_expression(e)).collect();

                Expression::Symbol { id, children }
            }
            Rule::symbol_char | Rule::WHITESPACE | Rule::EOI | Rule::symbol_name | Rule::number => {
                unreachable!()
            }
        }
    }

    pub fn parse(&self, string: &str) -> Result<Expression, pest::error::Error<Rule>> {
        let expr = LanguageParser::parse(Rule::standalone_expression, string)?
            .next()
            .unwrap();

        Ok(self.parse_expression(expr))
    }
}

#[cfg(test)]
mod tests {
    use crate::language::{
        Language,
        expression::{Expression, VariableId},
    };

    #[test]
    fn parse_variable() -> Result<(), pest::error::Error<super::Rule>> {
        let lang = Language::default();

        assert_eq!(Ok(Expression::Variable(0)), lang.parse("x0"));
        assert_eq!(Ok(Expression::Variable(5)), lang.parse("x5"));
        assert_eq!(Ok(Expression::Variable(123)), lang.parse("x123"));
        for var in Expression::NICE_VARIABLES {
            lang.parse(var)?;
        }

        Ok(())
    }

    fn expect_variable(expr: &Expression) -> VariableId {
        let Expression::Variable(id) = expr else {
            panic!("Expected variable but did not find it")
        };

        *id
    }

    fn expect_symbol<'a>(name: &str, lang: &Language, expr: &'a Expression) -> &'a Vec<Expression> {
        let Expression::Symbol { id, children } = expr else {
            panic!("Expected symbol but did not find a symbol")
        };

        assert_eq!(lang.get_id(name), *id);

        children
    }

    #[test]
    fn parse_symbol() {
        let lang = Language::math();
        let expr = lang.parse("(+ x y z)").unwrap();
        let plus_children = expect_symbol("+", &lang, &expr);
        let x = expect_variable(&plus_children[0]);
        let y = expect_variable(&plus_children[1]);
        let z = expect_variable(&plus_children[2]);

        assert_eq!(plus_children.len(), 3);

        assert_eq!(x, Expression::nice_variable_id("x"));
        assert_eq!(y, Expression::nice_variable_id("y"));
        assert_eq!(z, Expression::nice_variable_id("z"));
    }

    #[test]
    fn parse_expression() {
        let lang = Language::math();
        let expr = lang.parse("(+ (sin x) y (- z y))").unwrap();

        let plus_args = expect_symbol("+", &lang, &expr);
        assert_eq!(plus_args.len(), 3);

        let sin_arg = expect_symbol("sin", &lang, &plus_args[0]);
        assert_eq!(sin_arg.len(), 1);

        let x = expect_variable(&sin_arg[0]);
        assert_eq!(x, Expression::nice_variable_id("x"));

        let y = expect_variable(&plus_args[1]);
        assert_eq!(y, Expression::nice_variable_id("y"));

        let minus_args = expect_symbol("-", &lang, &plus_args[2]);
        assert_eq!(minus_args.len(), 2);

        let z = expect_variable(&minus_args[0]);
        assert_eq!(z, Expression::nice_variable_id("z"));

        let y = expect_variable(&minus_args[1]);
        assert_eq!(y, Expression::nice_variable_id("y"));
    }
}
