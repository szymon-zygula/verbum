use std::collections::HashMap;

use itertools::Itertools;

use crate::language::expression::{Expression, Symbol, VarFreeExpression, VariableId};

#[derive(Clone, Debug, Default)]
pub struct Matches<'e> {
    substitutions: HashMap<VariableId, &'e VarFreeExpression>,
}

impl<'e> Matches<'e> {
    fn try_merge(&self, other: &Self) -> Option<Self> {
        let new_match = self.clone();
        new_match.try_add_match(other)
    }

    fn try_add_match(mut self, other: &Matches<'e>) -> Option<Self> {
        for (key, value) in &self.substitutions {
            if let Some(other_value) = other.substitutions.get(key)
                && other_value != value
            {
                return None;
            }
        }

        self.substitutions.extend(other.substitutions.iter());
        Some(self)
    }

    fn substitutions(&self) -> &HashMap<VariableId, &'e VarFreeExpression> {
        &self.substitutions
    }

    pub fn at(&self, variable: VariableId) -> Option<&'e VarFreeExpression> {
        self.substitutions.get(&variable).map(|x| *x)
    }

    pub fn set(&mut self, variable: VariableId, expression: &'e VarFreeExpression) {
        self.substitutions.insert(variable, expression);
    }
}

impl Expression {
    /// `self` is treated as a pattern which may match `expression`
    pub fn try_match<'e>(&self, expression: &'e VarFreeExpression) -> Option<Matches<'e>> {
        match (self, expression) {
            (
                Expression::Symbol(Symbol {
                    id: id_1,
                    children: children_1,
                }),
                VarFreeExpression::Symbol(Symbol {
                    id: id_2,
                    children: children_2,
                }),
            ) => {
                if id_1 == id_2 && children_1.len() == children_2.len() {
                    let mut entire_match = Matches::default();

                    for (child_1, child_2) in children_1.iter().zip_eq(children_2.iter()) {
                        let child_match = child_1.try_match(child_2)?;
                        entire_match = entire_match.try_add_match(&child_match)?;
                    }

                    Some(entire_match)
                } else {
                    None
                }
            }
            (Expression::Variable(variable), expression @ _) => Some(Matches::<'e> {
                substitutions: HashMap::from([(*variable, expression)]),
            }),
            (Expression::Literal(lit_1), VarFreeExpression::Literal(lit_2)) => {
                (lit_1 == lit_2).then_some(Matches::<'e>::default())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Matches;
    use crate::language::{
        Language,
        expression::{Literal, VarFreeExpression},
    };

    #[test]
    fn match_merge() {
        let mut matches_1 = Matches::default();
        let mut matches_2 = Matches::default();

        let expr_1 = VarFreeExpression::Literal(Literal::Int(0));
        let expr_2 = VarFreeExpression::Literal(Literal::Int(1));

        matches_1.set(0, &expr_1);
        matches_1.set(1, &expr_2);

        matches_2.set(0, &expr_1);
        matches_2.set(2, &expr_2);

        let merged = matches_1.try_merge(&matches_2).unwrap();

        assert_eq!(merged.substitutions().len(), 3);
        assert_eq!(merged.at(0).unwrap(), &expr_1);
        assert_eq!(merged.at(1).unwrap(), &expr_2);
        assert_eq!(merged.at(2).unwrap(), &expr_2);

        matches_1.set(3, &expr_1);
        matches_2.set(3, &expr_2);
        assert!(matches_1.try_merge(&matches_2).is_none());
    }

    #[test]
    fn simple_match() {
        let lang = Language::math();
        let pattern = lang.parse("(+ 3 7u)").unwrap();

        let expr = lang.parse_no_vars("(+ 3 7u)").unwrap();
        assert_eq!(pattern.try_match(&expr).unwrap().substitutions().len(), 0);

        let expr = lang.parse_no_vars("(+ 3 7)").unwrap();
        assert!(pattern.try_match(&expr).is_none());

        let expr = lang.parse_no_vars("(+ 3 3u)").unwrap();
        assert!(pattern.try_match(&expr).is_none());

        let expr = lang.parse_no_vars("(- 3 7u)").unwrap();
        assert!(pattern.try_match(&expr).is_none());
    }

    #[test]
    fn nested_match() {
        let lang = Language::math();
        let string = "(+ (sin 1) (cos 3) (- 8 12u))";
        let pattern = lang.parse(string).unwrap();

        let expr = lang.parse_no_vars(string).unwrap();
        assert_eq!(pattern.try_match(&expr).unwrap().substitutions().len(), 0);

        let expr = lang.parse_no_vars("(+ (cos 1) (sin 3) (- 8 12u))").unwrap();
        assert!(pattern.try_match(&expr).is_none());
    }

    #[test]
    fn singular_variables_match() {
        let lang = Language::math();
        let pattern = lang.parse("(* x0 (sin x1))").unwrap();

        let expr = lang.parse_no_vars("(* 12 (sin (cos 5u)))").unwrap();
        let matches = pattern.try_match(&expr).unwrap();
        assert_eq!(matches.substitutions().len(), 2);

        let mul_args = expr.expect_symbol("*", &lang);
        assert!(std::ptr::eq(&mul_args[0], matches.substitutions()[&0]));

        let cos = &mul_args[1].expect_symbol("sin", &lang)[0];
        assert!(std::ptr::eq(cos, matches.substitutions()[&1]));
    }

    #[test]
    fn repeated_variable_match() {
        let lang = Language::math();
        let pattern = lang.parse("(+ (* x0 x1) (* x0 x2))").unwrap();

        let expr = lang
            .parse_no_vars("(+ (* (sin 1) 5) (* (sin 1) 3u))")
            .unwrap();

        let matches = pattern.try_match(&expr).unwrap();
        assert_eq!(matches.substitutions().len(), 3);

        let add_args = expr.expect_symbol("+", &lang);
        let mul_1_args = add_args[0].expect_symbol("*", &lang);
        let mul_2_args = add_args[1].expect_symbol("*", &lang);

        // Any of these two is semantically correct
        assert!(
            std::ptr::eq(matches.at(0).unwrap(), &mul_1_args[0])
                || std::ptr::eq(matches.at(0).unwrap(), &mul_2_args[0])
        );
        assert!(std::ptr::eq(matches.at(1).unwrap(), &mul_1_args[1]));
        assert!(std::ptr::eq(matches.at(2).unwrap(), &mul_2_args[1]));

        let expr = lang
            .parse_no_vars("(+ (* (sin 1) 5) (* (sin 2) 3u))")
            .unwrap();
        assert!(pattern.try_match(&expr).is_none());
    }
}
