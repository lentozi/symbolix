use std::panic::{catch_unwind, AssertUnwindSafe};

use symbolix_core::{
    lexer::constant::{Constant, Number},
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

fn boolean_var(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Boolean,
        value: None,
    }
}

#[test]
fn value_access_and_substitute_behave_as_expected() {
    let mut variable = numeric_var("x");
    assert_eq!(variable.get_value(), None);

    variable.set_value(Constant::integer(7));
    assert_eq!(variable.get_value(), Some(Constant::integer(7)));
    assert_eq!(variable.to_string(), "x");

    let replacement = SemanticExpression::numeric(NumericExpression::constant(Number::integer(3)));
    assert_eq!(variable.substitute(&variable, Some(&replacement)), replacement);
    assert_eq!(
        variable.substitute(&numeric_var("y"), None),
        variable.to_expression()
    );
}

#[test]
fn numeric_and_boolean_variables_convert_to_expected_expression_types() {
    assert!(matches!(
        numeric_var("x").to_expression(),
        SemanticExpression::Numeric(NumericExpression::Variable(_))
    ));
    assert!(matches!(
        Variable {
            name: "r".to_string(),
            var_type: VariableType::Fraction,
            value: None,
        }
        .as_expression(),
        SemanticExpression::Numeric(NumericExpression::Variable(_))
    ));
    assert!(matches!(
        boolean_var("flag").to_expression(),
        SemanticExpression::Logical(LogicalExpression::Variable(_))
    ));
}

#[test]
fn unknown_variable_cannot_convert_to_expression() {
    let panic = catch_unwind(AssertUnwindSafe(|| {
        Variable {
            name: "mystery".to_string(),
            var_type: VariableType::Unknown,
            value: None,
        }
        .to_expression();
    }));
    assert!(panic.is_err());
}

#[test]
fn numeric_operations_and_operator_impls_produce_numeric_expressions() {
    let x = numeric_var("x");
    let y = numeric_var("y");
    let frac = Variable {
        name: "q".to_string(),
        var_type: VariableType::Fraction,
        value: None,
    };

    assert_eq!(format!("{}", x.clone().addition(y.clone())), "(x + y)");
    assert_eq!(format!("{}", x.clone().subtraction(y.clone())), "(x + -(y))");
    assert_eq!(format!("{}", x.clone().multiplication(y.clone())), "(x * y)");
    assert_eq!(format!("{}", x.clone().division(y.clone())), "(x * (y)^(-1))");
    assert_eq!(format!("{}", x.clone().pow(y.clone())), "(x)^(y)");
    assert_eq!(format!("{}", x.clone().negation()), "-(x)");

    assert_eq!(format!("{}", (&x + &y)), "(x + y)");
    assert_eq!(format!("{}", (&x - 2)), "(-2 + x)");
    assert_eq!(format!("{}", (2 + &x)), "(2 + x)");
    assert_eq!(format!("{}", (&x * 3.0)), "(3 * x)");
    assert_eq!(format!("{}", (&x / 4_u32)), "(0.25 * x)");
    assert_eq!(
        format!(
            "{}",
            x.clone().add_expr(SemanticExpression::numeric(NumericExpression::constant(
                Number::integer(1)
            )))
        ),
        "(1 + x)"
    );
    assert!(matches!(
        frac.to_expression(),
        SemanticExpression::Numeric(NumericExpression::Variable(_))
    ));
}

#[test]
fn logical_operations_and_operator_impls_produce_logical_expressions() {
    let left = boolean_var("left");
    let right = boolean_var("right");

    assert_eq!(format!("{}", left.clone().and(right.clone())), "(left AND right)");
    assert_eq!(format!("{}", left.clone().or(right.clone())), "(left OR right)");
    assert_eq!(format!("{}", left.clone().not()), "NOT (left)");
    assert_eq!(format!("{}", (&left & &right)), "(left AND right)");
    assert_eq!(format!("{}", (&left | true)), "true");
    assert_eq!(
        format!(
            "{}",
            left.clone().and_expr(SemanticExpression::logical(LogicalExpression::constant(true)))
        ),
        "left"
    );
    assert_eq!(
        format!(
            "{}",
            left.clone().or_expr(SemanticExpression::logical(LogicalExpression::constant(false)))
        ),
        "left"
    );
}

#[test]
fn invalid_variable_operations_panic() {
    let bool_var = boolean_var("flag");
    let num_var = numeric_var("x");
    let unknown = Variable {
        name: "u".to_string(),
        var_type: VariableType::Unknown,
        value: None,
    };

    for operation in [
        catch_unwind(AssertUnwindSafe(|| bool_var.clone().addition(num_var.clone()))),
        catch_unwind(AssertUnwindSafe(|| unknown.clone().subtraction(num_var.clone()))),
        catch_unwind(AssertUnwindSafe(|| num_var.clone().and(bool_var.clone()))),
        catch_unwind(AssertUnwindSafe(|| bool_var.clone().mul_expr(SemanticExpression::numeric(
            NumericExpression::constant(Number::integer(1)),
        )))),
        catch_unwind(AssertUnwindSafe(|| num_var.clone().or_expr(SemanticExpression::logical(
            LogicalExpression::constant(true),
        )))),
        catch_unwind(AssertUnwindSafe(|| num_var.clone().pow_expr(SemanticExpression::logical(
            LogicalExpression::constant(true),
        )))),
        catch_unwind(AssertUnwindSafe(|| num_var.clone().not())),
        catch_unwind(AssertUnwindSafe(|| bool_var.clone().sub_expr(SemanticExpression::numeric(
            NumericExpression::constant(Number::integer(1)),
        )))),
        catch_unwind(AssertUnwindSafe(|| bool_var.clone().div_expr(SemanticExpression::numeric(
            NumericExpression::constant(Number::integer(1)),
        )))),
        catch_unwind(AssertUnwindSafe(|| bool_var.clone().pow(bool_var.clone()))),
        catch_unwind(AssertUnwindSafe(|| unknown.clone().mul_expr(SemanticExpression::numeric(
            NumericExpression::constant(Number::integer(1)),
        )))),
        catch_unwind(AssertUnwindSafe(|| unknown.clone().div_expr(SemanticExpression::numeric(
            NumericExpression::constant(Number::integer(1)),
        )))),
        catch_unwind(AssertUnwindSafe(|| unknown.clone().pow_expr(SemanticExpression::numeric(
            NumericExpression::constant(Number::integer(1)),
        )))),
        catch_unwind(AssertUnwindSafe(|| unknown.clone().negation())),
    ] {
        assert!(operation.is_err());
    }
}
