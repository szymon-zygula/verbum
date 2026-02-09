#[cfg(test)]
mod debug_tests {
    use crate::language::{Language, expression::{Literal, VarFreeExpression}};
    use crate::rewriting::egraph::{DynEGraph, EGraph, Node};

    #[test]
    fn debug_deferred_rebuilding() {
        let lang = Language::simple_math();
        let mut egraph = EGraph::<()>::default();

        let expr_1: VarFreeExpression = lang.parse_no_vars("(+ 1 5)").unwrap();
        let expr_2: VarFreeExpression = lang.parse_no_vars("(+ 1 2)").unwrap();

        let expr_1_id = egraph.add_expression(expr_1);
        let expr_2_id = egraph.add_expression(expr_2);

        println!("expr_1_id (+ 1 5): {}", expr_1_id);
        println!("expr_2_id (+ 1 2): {}", expr_2_id);

        let class_5 = egraph.node(expr_1_id).as_symbol().children[1];
        let class_2 = egraph.node(expr_2_id).as_symbol().children[1];

        println!("class_5: {}, class_2: {}", class_5, class_2);
        
        // Check parents before merge
        println!("Parents of class_5: {:?}", egraph.parents(class_5));
        println!("Parents of class_2: {:?}", egraph.parents(class_2));

        println!("Before merge:");
        println!("  Classes: {}", egraph.class_count());
        println!("  Actual nodes: {}", egraph.actual_node_count());

        egraph.merge_classes(class_5, class_2);

        println!("\nAfter merge, before rebuild:");
        println!("  Classes: {}", egraph.class_count());
        println!("  Actual nodes: {}", egraph.actual_node_count());

        egraph.rebuild();

        println!("\nAfter rebuild:");
        println!("  Classes: {}", egraph.class_count());
        println!("  Actual nodes: {}", egraph.actual_node_count());
    }
}
