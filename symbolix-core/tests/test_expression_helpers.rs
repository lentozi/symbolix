use symbolix_core::{
    lexer::{
        constant::Constant,
        symbol::{Binary, Relation, Symbol, Ternary, Unary},
    },
    parser::expression::Expression,
};

#[test]
fn expression_constructors_build_expected_shapes_and_display() {
    let constant = Expression::constant(Constant::integer(7));
    let variable = Expression::variable("x".to_string());
    let unary = Expression::unary(Symbol::Unary(Unary::Minus), variable.clone());
    let binary = Expression::binary(
        variable.clone(),
        Symbol::Binary(Binary::Add),
        constant.clone(),
    );
    let relation = Expression::relation(
        variable.clone(),
        Symbol::Relation(Relation::GreaterEqual),
        constant.clone(),
    );
    let ternary = Expression::ternary(
        relation.clone(),
        Symbol::Ternary(Ternary::Conditional),
        binary.clone(),
        Symbol::Ternary(Ternary::ConditionalElse),
        unary.clone(),
    );

    assert_eq!(constant.to_string(), "7");
    assert_eq!(variable.to_string(), "x");
    assert_eq!(unary.to_string(), "(- x)");
    assert_eq!(binary.to_string(), "(x + 7)");
    assert_eq!(relation.to_string(), "(x >= 7)");
    assert_eq!(ternary.to_string(), "((x >= 7) ? (x + 7) : (- x))");
}
