use exprion_core::{
    lexer::{
        constant::Number,
        symbol::{Relation, Symbol},
    },
    optimizer::testing::{
        extract_addition_term, extract_logical_term, extract_multiply_term, optimize_d1,
        optimize_logic_d1, optimize_numeric_d1, rebuild_addition_term, rebuild_logical_term,
        rebuild_multiply_term,
    },
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

fn bool_var(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Boolean,
        value: None,
    }
}

#[test]
fn optimize_term_extract_and_rebuild_cover_numeric_cases() {
    let x = NumericExpression::variable(numeric_var("x"));

    let addition_term =
        extract_addition_term(NumericExpression::constant(Number::integer(3)) * x.clone());
    assert_eq!(addition_term.coef, Number::integer(3));
    assert_eq!(addition_term.core.to_string(), "x");

    let neg_term = extract_addition_term(-x.clone());
    assert_eq!(neg_term.coef, Number::integer(-1));
    assert_eq!(neg_term.core.to_string(), "x");

    assert_eq!(
        rebuild_addition_term(addition_term).to_string(),
        "(3 * x)"
    );
    assert_eq!(
        rebuild_addition_term(extract_addition_term(NumericExpression::constant(
            Number::integer(4)
        )))
        .to_string(),
        "4"
    );

    let mult_term = extract_multiply_term(x.clone());
    assert_eq!(mult_term.exponent, Number::integer(1));
    assert_eq!(rebuild_multiply_term(mult_term).to_string(), "x");
    assert_eq!(
        rebuild_multiply_term(extract_multiply_term(NumericExpression::constant(
            Number::integer(2)
        )))
        .to_string(),
        "2"
    );
}

#[test]
fn optimize_term_extract_and_rebuild_cover_logical_cases() {
    let flag = LogicalExpression::variable(bool_var("flag"));
    let term = extract_logical_term(!flag.clone());
    assert!(term.is_not);
    assert_eq!(rebuild_logical_term(term).to_string(), "NOT (flag)");

    let plain = extract_logical_term(flag.clone());
    assert!(!plain.is_not);
    assert_eq!(rebuild_logical_term(plain).to_string(), "flag");
}

#[test]
fn optimize_numeric_d1_merges_like_terms_and_piecewise_values() {
    let x = NumericExpression::variable(numeric_var("x"));
    let expr = x.clone()
        + (NumericExpression::constant(Number::integer(2)) * x.clone())
        + NumericExpression::constant(Number::integer(5));
    let optimized = optimize_numeric_d1(expr);
    let rendered = optimized.to_string();
    assert!(rendered.contains("3"));
    assert!(rendered.contains("x"));
    assert!(rendered.contains("5"));

    let piecewise = NumericExpression::piecewise(
        vec![(
            LogicalExpression::relation(
                &x.clone(),
                &Symbol::Relation(Relation::GreaterThan),
                &NumericExpression::constant(Number::integer(0)),
            ),
            x.clone() + x.clone(),
        )],
        Some(x.clone() * x.clone()),
    );
    let optimized_piecewise = optimize_numeric_d1(piecewise);
    let rendered_piecewise = optimized_piecewise.to_string();
    assert!(rendered_piecewise.contains("2"));
    assert!(rendered_piecewise.contains("x"));
}

#[test]
fn optimize_logic_d1_handles_contradictions_and_nested_relations() {
    let flag = LogicalExpression::variable(bool_var("flag"));
    let contradiction = optimize_logic_d1(flag.clone() & !flag.clone());
    assert_eq!(contradiction.to_string(), "false");

    let tautology = optimize_logic_d1(flag.clone() | !flag.clone());
    assert_eq!(tautology.to_string(), "true");

    let x_rel = LogicalExpression::relation(
        &NumericExpression::variable(numeric_var("x")),
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    );
    let y_rel = LogicalExpression::relation(
        &NumericExpression::variable(numeric_var("y")),
        &Symbol::Relation(Relation::LessThan),
        &NumericExpression::constant(Number::integer(10)),
    );
    let preserved = optimize_logic_d1(x_rel.clone() & y_rel.clone());
    let rendered_preserved = preserved.to_string();
    assert!(rendered_preserved.contains("x"));
    assert!(rendered_preserved.contains("y"));

    let x = NumericExpression::variable(numeric_var("x"));
    let relation = LogicalExpression::relation(
        &(x.clone() + x.clone()),
        &Symbol::Relation(Relation::Equal),
        &(NumericExpression::constant(Number::integer(1)) + NumericExpression::constant(Number::integer(2))),
    );
    let optimized = optimize_logic_d1(relation);
    let rendered = optimized.to_string();
    assert!(rendered.contains("2"));
    assert!(rendered.contains("3"));
}

#[test]
fn optimize_d1_dispatches_for_semantic_expression() {
    let mut numeric = SemanticExpression::numeric(
        NumericExpression::variable(numeric_var("x")) + NumericExpression::variable(numeric_var("x")),
    );
    optimize_d1(&mut numeric);
    assert!(numeric.to_string().contains("2"));

    let mut logical = SemanticExpression::logical(
        LogicalExpression::variable(bool_var("flag")) | !LogicalExpression::variable(bool_var("flag")),
    );
    optimize_d1(&mut logical);
    assert_eq!(logical.to_string(), "true");
}
