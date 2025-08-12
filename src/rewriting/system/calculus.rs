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
            "(+ x0 0)" => "x0",
            "(* x0 1)" => "x0",
            "(* x0 0)" => "0",
            "(^ x0 1)" => "x0",
            // Arithmetic associativity and commutativity
            "(+ x0 x1)" => "(+ x1 x0)",
            "(* x0 x1)" => "(* x1 x0)",
            "(+ (+ x0 x1) x2)" => "(+ x0 (+ x1 x2))",
            "(* (* x0 x1) x2)" => "(* x0 (* x1 x2))",
            // Trigonometric identities
            "(+ (^ (sin x0) 2) (^ (cos x0) 2))" => "1",
            "(sin (+ x0 x1))" => "(+ (* (sin x0) (cos x1)) (* (cos x0) (sin x1)))",
            "(cos (+ x0 x1))" => "(- (* (cos x0) (cos x1)) (* (sin x0) (sin x1)))",
            "(sin (- x0 x1))" => "(- (* (sin x0) (cos x1)) (* (cos x0) (sin x1)))",
            "(cos (- x0 x1))" => "(+ (* (cos x0) (cos x1)) (* (sin x0) (sin x1)))",
            "(tan x0)" => "(/ (sin x0) (cos x0))",
    )
}

#[cfg(test)]
mod tests {
    use super::calculus;
    use crate::language::expression::Literal;
    use crate::rewriting::egraph::{
        EGraph,
        matching::bottom_up::BottomUpMatcher,
        saturation::{SaturationConfig, saturate},
    };

    #[test]
    fn trig_identity_rewrite_with_limits() {
        let trs = calculus();
        let lang = trs.language();

        // (+ (sin 3)^2 (cos 3)^2) => 1
        let expr = lang
            .parse_no_vars("(+ (^ (sin 3) 2) (^ (cos 3) 2))")
            .expect("valid expression");
        let mut egraph = EGraph::from_expression(expr);

        let _reason = saturate(
            &mut egraph,
            trs.rules(),
            BottomUpMatcher,
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
