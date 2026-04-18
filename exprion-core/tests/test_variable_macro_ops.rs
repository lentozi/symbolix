use exprion_core::semantic::{
    semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
    variable::{Variable, VariableType},
};

fn numeric_var(name: &str) -> Variable {
    Variable {
        name_id: 0,
        name: name.to_string(),
        var_type: VariableType::Float,
        value: None,
    }
}

fn bool_var(name: &str) -> Variable {
    Variable {
        name_id: 0,
        name: name.to_string(),
        var_type: VariableType::Boolean,
        value: None,
    }
}

#[test]
fn variable_macro_impls_cover_expr_owned_and_borrowed_variants() {
    let x = numeric_var("x");
    let y = numeric_var("y");
    let numeric_expr = SemanticExpression::numeric(NumericExpression::variable(y.clone()));
    let logic_var = bool_var("flag");
    let logic_expr = SemanticExpression::logical(LogicalExpression::constant(true));

    assert_eq!((x.clone() + numeric_expr.clone()).to_string(), "(x + y)");
    assert_eq!((x.clone() + &numeric_expr).to_string(), "(x + y)");
    assert_eq!((&x + numeric_expr.clone()).to_string(), "(x + y)");
    assert_eq!((&x + &numeric_expr).to_string(), "(x + y)");

    assert_eq!((numeric_expr.clone() + x.clone()).to_string(), "(x + y)");
    assert_eq!((numeric_expr.clone() + &x).to_string(), "(x + y)");
    assert_eq!((&numeric_expr + x.clone()).to_string(), "(x + y)");
    assert_eq!((&numeric_expr + &x).to_string(), "(x + y)");

    assert_eq!((logic_var.clone() & logic_expr.clone()).to_string(), "flag");
    assert_eq!((logic_var.clone() & &logic_expr).to_string(), "flag");
    assert_eq!((&logic_var & logic_expr.clone()).to_string(), "flag");
    assert_eq!((&logic_var & &logic_expr).to_string(), "flag");

    assert_eq!((logic_expr.clone() | logic_var.clone()).to_string(), "true");
    assert_eq!((logic_expr.clone() | &logic_var).to_string(), "true");
    assert_eq!((&logic_expr | logic_var.clone()).to_string(), "true");
    assert_eq!((&logic_expr | &logic_var).to_string(), "true");
}

#[test]
fn variable_macro_impls_cover_numeric_and_logic_literals_from_both_sides() {
    let x = numeric_var("x");
    let flag = bool_var("flag");

    assert_eq!((x.clone() + 2_i32).to_string(), "(2 + x)");
    assert_eq!((&x + 2_i64).to_string(), "(2 + x)");
    assert_eq!((2_u32 + x.clone()).to_string(), "(2 + x)");
    assert_eq!((2_f64 + &x).to_string(), "(2 + x)");

    assert_eq!((x.clone() * 3_i32).to_string(), "(3 * x)");
    assert_eq!((&x / 4_i64).to_string(), "(0.25 * x)");
    assert_eq!((5_u32 - x.clone()).to_string(), "(-5 + x)");

    assert_eq!((flag.clone() & true).to_string(), "flag");
    assert_eq!((&flag & true).to_string(), "flag");
    assert_eq!((true & flag.clone()).to_string(), "flag");
    assert_eq!((true & &flag).to_string(), "flag");

    assert_eq!((flag.clone() | false).to_string(), "flag");
    assert_eq!((&flag | false).to_string(), "flag");
    assert_eq!((false | flag.clone()).to_string(), "flag");
    assert_eq!((false | &flag).to_string(), "flag");
}
