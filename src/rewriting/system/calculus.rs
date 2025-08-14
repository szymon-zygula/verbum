use crate::macros::trs;
use crate::rewriting::system::TermRewritingSystem;

pub fn calculus() -> TermRewritingSystem {
    trs!(
        symbols: [
            // Arithmetic operators
            "+", "-", "*", "/", "^",
            // Trigonometric functions
            "sin", "cos", "tan", "sec",
            // Calculus operations
            // Additional functions
            "ln", "exp", "arcsin", "arccos", "arctan",
        ],
        rules:
            // Arithmetic simplifications
            "(+ $0 0)" => "$0",
            "(* $0 1)" => "$0",
            "(* $0 0)" => "0",
            "(^ $0 1)" => "$0",
            // Arithmetic associativity and commutativity
            "(+ $0 $1)" => "(+ $1 $0)",
            "(* $0 $1)" => "(* $1 $0)",
            "(+ (+ $0 $1) $2)" => "(+ $0 (+ $1 $2))",
            "(* (* $0 $1) $2)" => "(* $0 (* $1 $2))",
            // Trigonometric identities
            "(+ (^ (sin $0) 2) (^ (cos $0) 2))" => "1",
            "(sin (+ $0 $1))" => "(+ (* (sin $0) (cos $1)) (* (cos $0) (sin $1)))",
            "(cos (+ $0 $1))" => "(- (* (cos $0) (cos $1)) (* (sin $0) (sin $1)))",
            "(sin (- $0 $1))" => "(- (* (sin $0) (cos $1)) (* (cos $0) (sin $1)))",
            "(cos (- $0 $1))" => "(+ (* (cos $0) (cos $1)) (* (sin $0) (sin $1)))",
            "(tan $0)" => "(/ (sin $0) (cos $0))",
    )
}

#[cfg(test)]
mod tests {
    use super::calculus;
    use crate::language::expression::Literal;
    use crate::rewriting::egraph::{
        EGraph,
        matching::bottom_up::BottomUpMatcher,
        saturation::{SaturationConfig, Saturator, SimpleSaturator},
    };

    #[test]
    fn trig_identity_rewrite_with_limits() {
        let trs = calculus();
        let lang = trs.language();

        // (+ (sin 3)^2 (cos 3)^2) => 1
        let expr = lang
            .parse_no_vars("(+ (^ (sin 3) 2) (^ (cos 3) 2))")
            .expect("valid expression");
        let mut egraph = EGraph::<()>::from_expression(expr);

        let saturator = SimpleSaturator::new(BottomUpMatcher);
        let _reason = saturator.saturate(
            &mut egraph,
            trs.rules(),
            &SaturationConfig {
                max_applications: Some(10),
                max_nodes: Some(200),
                ..Default::default()
            },
        );

        assert!(
            egraph.find_literal(Literal::Int(1)).is_some(),
            "Expected to derive literal 1 from trig identity"
        );
    }
}
