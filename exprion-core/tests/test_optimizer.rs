use exprion_core::{
    lexer::{
        constant::Number,
        symbol::{Relation, Symbol},
    },
    optimizer::{flatten_numeric, normalize, normalize_logic, normalize_numeric, optimize},
    optimizer::testing::factor,
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
        variable::{Variable, VariableType},
    },
};
use std::panic::{catch_unwind, AssertUnwindSafe};

fn numeric_var(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Float,
        value: None,
    }
}

fn bool_var(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Boolean,
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

#[test]
fn normalize_numeric_and_logic_cover_singleton_and_zero_paths() {
    let x = NumericExpression::variable(numeric_var("x"));

    let mut add_single = NumericExpression::Addition(exprion_core::numeric_bucket![
        NumericExpression::constant(Number::integer(0)),
        x.clone()
    ]);
    normalize_numeric(&mut add_single);
    assert_eq!(add_single.to_string(), "x");

    let mut mul_zero = NumericExpression::Multiplication(exprion_core::numeric_bucket![
        NumericExpression::constant(Number::integer(0)),
        x.clone()
    ]);
    normalize_numeric(&mut mul_zero);
    assert_eq!(mul_zero.to_string(), "0");

    let mut mul_single = NumericExpression::Multiplication(exprion_core::numeric_bucket![
        NumericExpression::constant(Number::integer(1)),
        x.clone()
    ]);
    normalize_numeric(&mut mul_single);
    assert_eq!(mul_single.to_string(), "x");

    let mut and_single = LogicalExpression::And(exprion_core::logical_bucket![
        LogicalExpression::constant(true),
        LogicalExpression::variable(bool_var("flag"))
    ]);
    normalize_logic(&mut and_single);
    assert_eq!(and_single.to_string(), "flag");

    let mut relation = LogicalExpression::Relation {
        left: Box::new(NumericExpression::constant(Number::integer(2))),
        operator: Symbol::Relation(Relation::GreaterThan),
        right: Box::new(NumericExpression::constant(Number::integer(1))),
    };
    normalize_logic(&mut relation);
    assert_eq!(relation.to_string(), "true");
}

#[test]
fn normalize_and_factor_dispatch_over_semantic_expression() {
    let mut numeric = SemanticExpression::numeric(NumericExpression::Addition(
        exprion_core::numeric_bucket![
            NumericExpression::constant(Number::integer(0)),
            NumericExpression::variable(numeric_var("x"))
        ],
    ));
    normalize(&mut numeric);
    factor(&mut numeric);
    assert_eq!(numeric.to_string(), "x");

    let mut logical = SemanticExpression::logical(LogicalExpression::And(
        exprion_core::logical_bucket![
            LogicalExpression::constant(true),
            LogicalExpression::variable(bool_var("flag"))
        ],
    ));
    normalize(&mut logical);
    factor(&mut logical);
    assert_eq!(logical.to_string(), "flag");
}

#[test]
fn normalize_panics_on_empty_reducible_buckets_and_handles_invalid_relation_symbol() {
    let mut add_zero_only = NumericExpression::Addition(exprion_core::numeric_bucket![
        NumericExpression::constant(Number::integer(0))
    ]);
    assert!(catch_unwind(AssertUnwindSafe(|| normalize_numeric(&mut add_zero_only))).is_err());

    let mut or_false_only = LogicalExpression::Or(exprion_core::logical_bucket![
        LogicalExpression::constant(false)
    ]);
    assert!(catch_unwind(AssertUnwindSafe(|| normalize_logic(&mut or_false_only))).is_err());

    let mut invalid_relation = LogicalExpression::Relation {
        left: Box::new(NumericExpression::constant(Number::integer(1))),
        operator: Symbol::Binary(exprion_core::lexer::symbol::Binary::Add),
        right: Box::new(NumericExpression::constant(Number::integer(2))),
    };
    normalize_logic(&mut invalid_relation);
    assert_eq!(invalid_relation.to_string(), "false");
}
