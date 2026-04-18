use exprion_core::{
    lexer::{
        constant::Number,
        symbol::{Relation, Symbol},
    },
    optimizer::optimize,
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
        variable::{Variable, VariableType},
    },
};

fn numeric_var(name: &str) -> Variable {
    Variable {
        name_id: 0,
        name: name.to_string(),
        var_type: VariableType::Float,
        value: None,
    }
}

fn boolean_var(name: &str) -> Variable {
    Variable {
        name_id: 0,
        name: name.to_string(),
        var_type: VariableType::Boolean,
        value: None,
    }
}

#[test]
fn substitutes_numeric_variable_with_expression() {
    let x = numeric_var("x");
    let y = numeric_var("y");
    let expr =
        NumericExpression::variable(x.clone()) + NumericExpression::constant(Number::integer(2));

    let substituted = expr.substitute(
        &x,
        Some(
            &(NumericExpression::variable(y.clone())
                * NumericExpression::constant(Number::integer(3))),
        ),
    );

    assert_eq!(format!("{}", substituted), "(2 + (3 * y))");
}

#[test]
fn substitutes_semantic_expression_with_optional_none() {
    let x = numeric_var("x");
    let expr = SemanticExpression::numeric(
        NumericExpression::variable(x.clone()) + NumericExpression::constant(Number::integer(1)),
    );

    let substituted = expr.substitute(&x, None);

    assert_eq!(format!("{}", substituted), "(1 + x)");
}

#[test]
fn substitutes_logical_relation_numeric_side() {
    let x = numeric_var("x");
    let relation = LogicalExpression::relation(
        &NumericExpression::variable(x.clone()),
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    );

    let substituted = relation.substitute(
        &x,
        Some(&SemanticExpression::numeric(NumericExpression::constant(
            Number::integer(5),
        ))),
    );

    assert_eq!(substituted, LogicalExpression::Constant(true));
}

#[test]
fn substitutes_logical_variable_with_logical_expression() {
    let flag = boolean_var("flag");
    let condition = LogicalExpression::variable(flag.clone());

    let substituted = condition.substitute(
        &flag,
        Some(&SemanticExpression::logical(LogicalExpression::constant(
            true,
        ))),
    );

    assert_eq!(format!("{}", substituted), "true");
}

#[test]
fn folds_constant_relations_during_optimization() {
    let mut expr = SemanticExpression::logical(LogicalExpression::relation(
        &NumericExpression::constant(Number::integer(5)),
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    ));

    optimize(&mut expr);

    assert_eq!(
        expr,
        SemanticExpression::logical(LogicalExpression::constant(true))
    );
}
