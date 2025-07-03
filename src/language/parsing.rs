use super::{
    Language, SymbolId,
    expression::{Expression, VariableId},
};

impl Language {
    fn parse_variable(&self, s: &str) -> Option<VariableId> {
        Expression::NICE_VARIABLES
            .iter()
            .position(|x| *x == s)
            .or(s.strip_prefix("x").map(|n| n.parse().ok()).flatten())
    }

    fn parse_symbol(&self, s: &str) -> Option<(SymbolId, Vec<Expression>)> {
        let mut insides = s.strip_prefix("(")?.strip_suffix(")")?.split(" ");
        let id = self.try_get_id(insides.next()?)?;
        let mut children = Vec::with_capacity(insides.size_hint().0);

        for inside in insides {
            children.push(self.parse_expr(inside)?);
        }

        Some((id, children))
    }

    pub fn parse_expr(&self, s: &str) -> Option<Expression> {
        self.parse_variable(s)
            .map(|id| Expression::Variable(id))
            .or(self
                .parse_symbol(s)
                .map(|(id, children)| Expression::Symbol { id, children }))
    }
}

#[cfg(test)]
mod tests {
    use crate::language::{
        Language,
        expression::{Expression, VariableId},
    };

    #[test]
    fn parse_variable() {
        let lang = Language::default();

        assert_eq!(Some(0), lang.parse_variable("x0"));
        assert_eq!(Some(5), lang.parse_variable("x5"));
        assert_eq!(Some(123), lang.parse_variable("x123"));
        for var in Expression::NICE_VARIABLES {
            assert_ne!(None, lang.parse_variable(var));
        }
    }

    fn expect_variable(expr: &Expression) -> VariableId {
        let Expression::Variable(id) = expr else {
            panic!("Expected variable but did not find it")
        };

        *id
    }

    #[test]
    fn parse_symbol() {
        let lang = Language::math();
        let (id, children) = lang.parse_symbol("(+ x y z)").unwrap();
        let x = expect_variable(&children[0]);
        let y = expect_variable(&children[1]);
        let z = expect_variable(&children[2]);

        assert_eq!(id, lang.get_id("+"));
        assert_eq!(x, Expression::variable_id("x"));
        assert_eq!(y, Expression::variable_id("y"));
        assert_eq!(z, Expression::variable_id("z"));
    }
}
