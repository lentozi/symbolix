use symbolix_core::{
    lexer::{
        constant::Number,
        symbol::{Relation, Symbol},
    },
    optimizer::{flatten_numeric, normalize_logic, normalize_numeric, optimize},
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
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
fn flatten_numeric_merges_nested_additions_and_multiplications() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));

    let flattened_add = flatten_numeric(
        NumericExpression::constant(Number::integer(1))
            + (x.clone() + NumericExpression::constant(Number::integer(2)))
            + y.clone(),
    );
    let rendered_add = format!("{}", flattened_add);
    assert!(rendered_add.contains("1"));
    assert!(rendered_add.contains("2"));
    assert!(rendered_add.contains("x"));
    assert!(rendered_add.contains("y"));

    let flattened_mul = flatten_numeric(
        NumericExpression::constant(Number::integer(2))
            * (x.clone() * NumericExpression::constant(Number::integer(3)))
            * y,
    );
    let rendered_mul = format!("{}", flattened_mul);
    assert!(rendered_mul.contains("2"));
    assert!(rendered_mul.contains("3"));
    assert!(rendered_mul.contains("x"));
    assert!(rendered_mul.contains("y"));
}

#[test]
fn normalize_numeric_simplifies_piecewise_and_negation_shapes() {
    let x = NumericExpression::variable(numeric_var("x"));
    let mut expr = NumericExpression::Piecewise {
        cases: vec![(
            LogicalExpression::constant(true),
            -(-x.clone() + NumericExpression::constant(Number::integer(2))),
        )],
        otherwise: Some(Box::new(NumericExpression::constant(Number::integer(0)))),
    };

    normalize_numeric(&mut expr);

    let rendered = format!("{}", expr);
    assert!(rendered.contains("x"));
    assert!(!rendered.contains("-(-"));
}

#[test]
fn normalize_logic_and_optimize_fold_constant_relations() {
    let mut logic = LogicalExpression::relation(
        &NumericExpression::constant(Number::integer(2)),
        &Symbol::Relation(Relation::LessThan),
        &NumericExpression::constant(Number::integer(3)),
    ) & LogicalExpression::constant(true);
    normalize_logic(&mut logic);
    assert_eq!(format!("{}", logic), "true");

    let mut semantic = SemanticExpression::logical(LogicalExpression::relation(
        &NumericExpression::constant(Number::integer(5)),
        &Symbol::Relation(Relation::GreaterEqual),
        &NumericExpression::constant(Number::integer(5)),
    ));
    optimize(&mut semantic);
    assert_eq!(format!("{}", semantic), "true");
}
