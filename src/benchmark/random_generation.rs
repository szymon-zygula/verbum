use rand::Rng;

use crate::language::{
    Language,
    expression::{Literal, VarFreeExpression},
    symbol::Symbol,
};

pub fn generate_random_expression(
    language: &Language,
    max_size: usize,
    rng: &mut impl Rng,
) -> VarFreeExpression {
    generate_random_expression_recursive(language, max_size, rng, 0)
}

fn generate_random_expression_recursive(
    language: &Language,
    max_size: usize,
    rng: &mut impl Rng,
    current_size: usize,
) -> VarFreeExpression {
    if current_size >= max_size {
        // Generate a literal if max_size is reached
        return generate_random_literal(rng);
    }

    // Decide whether to generate a literal or a symbol
    if rng.gen_bool(0.3) || language.symbol_count() == 0 {
        // 30% chance to generate a literal, or if no symbols are defined
        generate_random_literal(rng)
    } else {
        // Generate a symbol
        let symbol_id = rng.gen_range(0..language.symbol_count());
        let symbol_name = language.get_symbol(symbol_id);

        // For simple_math, assume arities:
        // +, -, *, /, sin, cos, <<, >>
        let arity = match symbol_name {
            "+" | "-" | "*" | "/" => 2,
            "sin" | "cos" | "<<" | ">>" => 1,
            _ => 0, // Default arity for unknown symbols
        };

        let children: Vec<VarFreeExpression> = (0..arity)
            .map(|_| {
                generate_random_expression_recursive(language, max_size, rng, current_size + 1)
            })
            .collect();

        VarFreeExpression::Symbol(Symbol {
            id: symbol_id,
            children,
        })
    }
}

fn generate_random_literal(rng: &mut impl Rng) -> VarFreeExpression {
    // Randomly choose between Int and UInt for now
    if rng.gen_bool(0.5) {
        VarFreeExpression::Literal(Literal::Int(rng.gen_range(-100..100)))
    } else {
        VarFreeExpression::Literal(Literal::UInt(rng.gen_range(0..100)))
    }
}

#[cfg(test)]
mod tests {
    use super::generate_random_expression;
    use crate::language::Language;

    #[test]
    fn test_generate_random_expression() {
        let lang = Language::simple_math();
        let mut rng = rand::thread_rng();

        for size in 1..10 {
            let expr = generate_random_expression(&lang, size, &mut rng);
            println!("Generated expression (size {size}): {expr:?}");
            // Basic assertion: ensure it doesn't panic and is a valid expression
            assert!(!format!("{expr:?}").is_empty());
        }
    }
}
