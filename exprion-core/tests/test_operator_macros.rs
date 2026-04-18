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
fn semantic_expression_operator_impls_cover_owned_and_borrowed_variants() {
    let x = SemanticExpression::numeric(NumericExpression::variable(numeric_var("x")));
    let y = SemanticExpression::numeric(NumericExpression::variable(numeric_var("y")));
    let flag = SemanticExpression::logical(LogicalExpression::variable(bool_var("flag")));

    assert_eq!((x.clone() + y.clone()).to_string(), "(x + y)");
    assert_eq!((x.clone() + &y).to_string(), "(x + y)");
    assert_eq!((&x + y.clone()).to_string(), "(x + y)");
    assert_eq!((&x + &y).to_string(), "(x + y)");

    assert_eq!((x.clone() + 2_i32).to_string(), "(2 + x)");
    assert_eq!((&x + 2_i64).to_string(), "(2 + x)");
    assert_eq!((2_u32 + x.clone()).to_string(), "(2 + x)");
    assert_eq!((2_f64 + &x).to_string(), "(2 + x)");

    assert_eq!((flag.clone() & true).to_string(), "flag");
    assert_eq!((&flag & true).to_string(), "flag");
    assert_eq!((true | flag.clone()).to_string(), "(true OR flag)");
    assert_eq!((false | &flag).to_string(), "(false OR flag)");

    assert_eq!((!flag.clone()).to_string(), "NOT (flag)");
    assert_eq!((-&x).to_string(), "-(x)");
}

#[test]
fn numeric_and_logical_ir_operator_impls_cover_owned_and_borrowed_variants() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let flag = LogicalExpression::variable(bool_var("flag"));

    assert_eq!((x.clone() + y.clone()).to_string(), "(x + y)");
    assert_eq!((x.clone() + &y).to_string(), "(x + y)");
    assert_eq!((&x + y.clone()).to_string(), "(x + y)");
    assert_eq!((&x + &y).to_string(), "(x + y)");

    assert_eq!((x.clone() + 3_i32).to_string(), "(3 + x)");
    assert_eq!((&x * 2_i64).to_string(), "(2 * x)");
    assert_eq!((4_u32 / x.clone()).to_string(), "(4 * (x)^(-1))");
    assert_eq!((5_f64 - &x).to_string(), "(5 + -(x))");
    assert_eq!((-(x.clone())).to_string(), "-(x)");

    assert_eq!((flag.clone() & LogicalExpression::constant(true)).to_string(), "flag");
    assert_eq!((flag.clone() & &LogicalExpression::constant(true)).to_string(), "flag");
    assert_eq!((&flag | LogicalExpression::constant(false)).to_string(), "flag");
    assert_eq!((&flag | &LogicalExpression::constant(false)).to_string(), "flag");
    assert_eq!((flag.clone() & true).to_string(), "flag");
    assert_eq!((false | &flag).to_string(), "(false OR flag)");
    assert_eq!((!flag).to_string(), "NOT (flag)");
}
