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
            "d", // derivative
            "∫", // integral
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
            // For bidirectionality, add the reverse directions explicitly
            "(+ x1 x0)" => "(+ x0 x1)",
            "(* x1 x0)" => "(* x0 x1)",
            "(+ (+ x0 x1) x2)" => "(+ x0 (+ x1 x2))",
            "(* (* x0 x1) x2)" => "(* x0 (* x1 x2))",
            // Derivative rules
            "(d (+ x0 x1))" => "(+ (d x0) (d x1))",
            "(d (* x0 x1))" => "(+ (* x0 (d x1)) (* x1 (d x0)))",
            "(d (^ x0 x1))" => "(* (* (^ x0 x1) (ln x0)) (d x1))",
            "(d (exp x0))" => "(* (exp x0) (d x0))",
            "(d (ln x0))" => "(/ (d x0) x0)",
            "(d (sin x0))" => "(* (cos x0) (d x0))",
            "(d (cos x0))" => "(* (* -1 (sin x0)) (d x0))",
            "(d (tan x0))" => "(* (^ (sec x0) 2) (d x0))",
            // Integration rules
            "(∫ (+ x0 x1))" => "(+ (∫ x0) (∫ x1))",
            "(∫ (* c x0))" => "(* c (∫ x0))",
            "(∫ (^ x0 -1))" => "(ln x0)",
            "(∫ (sin x0))" => "(* -1 (cos x0))",
            "(∫ (cos x0))" => "(sin x0)",
            // Trigonometric identities
            "(+ (^ (sin x0) 2) (^ (cos x0) 2))" => "1",
            "(sin (+ x0 x1))" => "(+ (* (sin x0) (cos x1)) (* (cos x0) (sin x1)))",
            "(cos (+ x0 x1))" => "(- (* (cos x0) (cos x1)) (* (sin x0) (sin x1)))",
            "(sin (- x0 x1))" => "(- (* (sin x0) (cos x1)) (* (cos x0) (sin x1)))",
            "(cos (- x0 x1))" => "(+ (* (cos x0) (cos x1)) (* (sin x0) (sin x1)))",
            "(tan x0)" => "(/ (sin x0) (cos x0))",
    )
}
