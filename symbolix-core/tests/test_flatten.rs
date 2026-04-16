use symbolix_core::{
    lexer::{
        constant::Number,
        symbol::{Relation, Symbol},
    },
    optimizer::flatten_numeric,
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression},
        variable::{Variable, VariableType},
    },
};

fn numeric_var(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Float,
        value: None,
    }
}

#[test]
fn flatten_numeric_covers_multiplication_with_many_expression_terms() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let z = NumericExpression::variable(numeric_var("z"));

    let expr = NumericExpression::Multiplication(symbolix_core::numeric_bucket![
        NumericExpression::constant(Number::integer(2)),
        x.clone() + NumericExpression::constant(Number::integer(1)),
        y.clone() + NumericExpression::constant(Number::integer(2)),
        z.clone()
    ]);

    let flattened = flatten_numeric(expr);
    let rendered = flattened.to_string();
    assert!(rendered.contains("2"));
    assert!(rendered.contains("z"));
    assert!(rendered.contains("x"));
    assert!(rendered.contains("y"));
}

#[test]
fn flatten_numeric_covers_single_and_zero_expression_multiplication_paths() {
    let x = NumericExpression::variable(numeric_var("x"));

    let single_expr = NumericExpression::Multiplication(symbolix_core::numeric_bucket![
        NumericExpression::constant(Number::integer(3)),
        x.clone() + NumericExpression::constant(Number::integer(1))
    ]);
    let flattened_single = flatten_numeric(single_expr);
    assert!(flattened_single.to_string().contains("3"));

    let zero_expr = NumericExpression::Multiplication(symbolix_core::numeric_bucket![
        NumericExpression::constant(Number::integer(5)),
        x.clone()
    ]);
    let flattened_zero = flatten_numeric(zero_expr);
    assert_eq!(flattened_zero.to_string(), "(5 * x)");
}

#[test]
fn flatten_numeric_covers_power_piecewise_and_negation_paths() {
    let x = NumericExpression::variable(numeric_var("x"));
    let condition = LogicalExpression::relation(
        &x,
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    );

    let expr = NumericExpression::Negation(Box::new(NumericExpression::Power {
        base: Box::new(NumericExpression::Piecewise {
            cases: vec![(condition, x.clone() + NumericExpression::constant(Number::integer(1)))],
            otherwise: Some(Box::new(NumericExpression::constant(Number::integer(0)))),
        }),
        exponent: Box::new(NumericExpression::constant(Number::integer(2))),
    }));

    let flattened = flatten_numeric(expr);
    let rendered = flattened.to_string();
    assert!(rendered.contains("^"));
    assert!(rendered.contains("other"));
    assert!(rendered.contains("-("));
}
