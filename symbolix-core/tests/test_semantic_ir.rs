use std::panic::{catch_unwind, AssertUnwindSafe};

use symbolix_core::{
    lexer::{
        constant::Number,
        symbol::{Relation, Symbol},
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
fn numeric_expression_operations_cover_piecewise_power_and_substitute() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let cond = LogicalExpression::relation(
        &x,
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    );

    let piecewise = NumericExpression::piecewise(
        vec![(cond.clone(), x.clone() + NumericExpression::constant(Number::integer(1)))],
        Some(y.clone()),
    );
    let added = NumericExpression::addition(&piecewise, &NumericExpression::constant(Number::integer(2)));
    let multiplied = NumericExpression::multiplication(&piecewise, &NumericExpression::constant(Number::integer(3)));
    let powered = NumericExpression::power(
        &(x.clone() * y.clone()),
        &NumericExpression::constant(Number::integer(2)),
    );

    assert!(added.to_string().contains("2"));
    assert!(multiplied.to_string().contains("3"));
    assert!(powered.to_string().contains("x"));
    assert!(powered.to_string().contains("y"));

    let substituted = piecewise.substitute(
        &numeric_var("x"),
        Some(&NumericExpression::constant(Number::integer(4))),
    );
    assert!(substituted.to_string().contains("5"));
    assert!(substituted.to_string().contains("y"));
}

#[test]
fn logical_expression_operations_cover_not_and_or_relation_and_substitute() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let relation = LogicalExpression::relation(&x, &Symbol::Relation(Relation::LessThan), &y);
    let inverted = LogicalExpression::not(&relation);
    assert!(inverted.to_string().contains(">="));

    let flag = LogicalExpression::variable(bool_var("flag"));
    assert_eq!((flag.clone() & true).to_string(), "flag");
    assert_eq!((flag.clone() | false).to_string(), "flag");

    let substituted = relation.substitute(
        &numeric_var("x"),
        Some(&SemanticExpression::numeric(NumericExpression::constant(
            Number::integer(5),
        ))),
    );
    assert!(substituted.to_string().contains("5"));
}

#[test]
fn semantic_expression_operations_cover_kinds_equations_and_substitution() {
    let x = SemanticExpression::numeric(NumericExpression::variable(numeric_var("x")));
    let y = SemanticExpression::numeric(NumericExpression::variable(numeric_var("y")));
    let flag = SemanticExpression::logical(LogicalExpression::variable(bool_var("flag")));

    assert_eq!((x.clone() + y.clone()).to_string(), "(x + y)");
    assert_eq!((x.clone() - y.clone()).to_string(), "(x + -(y))");
    assert_eq!((x.clone() * y.clone()).to_string(), "(x * y)");
    assert_eq!((x.clone() / y.clone()).to_string(), "(x * (y)^(-1))");
    assert_eq!(SemanticExpression::power(&x, &y).to_string(), "(x)^(y)");
    assert_eq!((flag.clone() & true).to_string(), "flag");
    assert_eq!((flag.clone() | false).to_string(), "flag");
    assert_eq!((!flag.clone()).to_string(), "NOT (flag)");
    assert_eq!(SemanticExpression::one().to_string(), "1");
    assert!(x.is_numeric());
    assert!(flag.is_logical());

    let equation = SemanticExpression::logical(LogicalExpression::relation(
        &NumericExpression::variable(numeric_var("x")),
        &Symbol::Relation(Relation::Equal),
        &NumericExpression::constant(Number::integer(0)),
    ));
    assert!(equation.is_equation());

    let substituted = x.substitute(
        &numeric_var("x"),
        Some(&SemanticExpression::numeric(NumericExpression::constant(
            Number::integer(10),
        ))),
    );
    assert_eq!(substituted.to_string(), "10");
}

#[test]
fn semantic_expression_panics_on_invalid_cross_type_operations() {
    let numeric = SemanticExpression::numeric(NumericExpression::constant(Number::integer(1)));
    let logical = SemanticExpression::logical(LogicalExpression::variable(bool_var("flag")));

    for panic in [
        catch_unwind(AssertUnwindSafe(|| SemanticExpression::addition(&numeric, &logical))),
        catch_unwind(AssertUnwindSafe(|| SemanticExpression::not(&numeric))),
        catch_unwind(AssertUnwindSafe(|| SemanticExpression::power(&numeric, &logical))),
        catch_unwind(AssertUnwindSafe(|| logical.substitute(
            &bool_var("flag"),
            Some(&SemanticExpression::numeric(NumericExpression::constant(Number::integer(1)))),
        ))),
        catch_unwind(AssertUnwindSafe(|| numeric.substitute(
            &numeric_var("x"),
            Some(&SemanticExpression::logical(LogicalExpression::constant(true))),
        ))),
    ] {
        assert!(panic.is_err());
    }
}
