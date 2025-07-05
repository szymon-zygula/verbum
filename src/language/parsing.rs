use super::{
    Language,
    expression::{Expression, Literal, VarFreeExpression},
    symbol::Symbol,
};
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

                Expression::Symbol(Symbol { id, children })
            }
            Rule::literal => self.parse_expression(pair.into_inner().next().unwrap()),
            Rule::integer => Expression::Literal(Literal::Int(pair.as_str().parse().unwrap())),
            Rule::unsigned_integer => Expression::Literal(Literal::UInt(
                pair.as_str().strip_suffix("u").unwrap().parse().unwrap(),
            )),
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

    pub fn parse_no_vars(&self, string: &str) -> anyhow::Result<VarFreeExpression> {
        self.parse(string)?
            .without_variables()
            .ok_or(anyhow::Error::msg(
                "Trying to parse an expression with variables as VarFreeExpression",
            ))
    }
}

#[cfg(test)]
mod tests {
    use crate::language::{
        Language,
        expression::{Expression, Literal},
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

    #[test]
    fn parse_symbol() {
        let lang = Language::math();
        let expr = lang.parse("(+ x y z)").unwrap();
        let plus_children = expr.expect_symbol("+", &lang);
        let x = plus_children[0].expect_variable();
        let y = plus_children[1].expect_variable();
        let z = plus_children[2].expect_variable();

        assert_eq!(plus_children.len(), 3);

        assert_eq!(x, Expression::nice_variable_id("x"));
        assert_eq!(y, Expression::nice_variable_id("y"));
        assert_eq!(z, Expression::nice_variable_id("z"));
    }

    #[test]
    fn parse_expression() {
        let lang = Language::math();
        let expr = lang.parse("(+ (sin x) y (- z y))").unwrap();

        let plus_args = expr.expect_symbol("+", &lang);
        assert_eq!(plus_args.len(), 3);

        let sin_arg = plus_args[0].expect_symbol("sin", &lang);
        assert_eq!(sin_arg.len(), 1);

        let x = sin_arg[0].expect_variable();
        assert_eq!(x, Expression::nice_variable_id("x"));

        let y = plus_args[1].expect_variable();
        assert_eq!(y, Expression::nice_variable_id("y"));

        let minus_args = plus_args[2].expect_symbol("-", &lang);
        assert_eq!(minus_args.len(), 2);

        let z = minus_args[0].expect_variable();
        assert_eq!(z, Expression::nice_variable_id("z"));

        let y = minus_args[1].expect_variable();
        assert_eq!(y, Expression::nice_variable_id("y"));
    }

    #[test]
    fn parse_integers_simple() {
        let lang = Language::math();

        let expr = lang.parse("12").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::Int(12))));

        let expr = lang.parse("0").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::Int(0))));

        let expr = lang.parse("-42").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::Int(-42))));
    }

    #[test]
    fn parse_unsigned_integers_simple() {
        let lang = Language::math();

        let expr = lang.parse("12u").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::UInt(12))));

        let expr = lang.parse("0u").unwrap();
        assert!(matches!(expr, Expression::Literal(Literal::UInt(0))));

        assert!(lang.parse("-42u").is_err());
    }

    #[test]
    fn parse_integers_many() {
        let lang = Language::math();

        let expr = lang.parse("(+ -12 0u 4u 128)").unwrap();
        let children = expr.expect_symbol("+", &lang);

        assert_eq!(children.len(), 4);
        assert!(matches!(
            children[0],
            Expression::Literal(Literal::Int(-12))
        ));
        assert!(matches!(children[1], Expression::Literal(Literal::UInt(0))));
        assert!(matches!(children[2], Expression::Literal(Literal::UInt(4))));
        assert!(matches!(
            children[3],
            Expression::Literal(Literal::Int(128))
        ));
    }
}
