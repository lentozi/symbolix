use std::panic::{catch_unwind, AssertUnwindSafe};

use exprion_core::{
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
fn numeric_expression_covers_piecewise_pairwise_composition_and_factor_hooks() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let positive_x = LogicalExpression::relation(
        &x,
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    );
    let positive_y = LogicalExpression::relation(
        &y,
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    );

    let left = NumericExpression::piecewise(
        vec![(positive_x.clone(), x.clone())],
        Some(NumericExpression::constant(Number::integer(1))),
    );
    let right = NumericExpression::piecewise(
        vec![(positive_y.clone(), y.clone())],
        Some(NumericExpression::constant(Number::integer(2))),
    );

    let added = NumericExpression::addition(&left, &right);
    let multiplied = NumericExpression::multiplication(&left, &right);
    let added_rendered = added.to_string();
    let multiplied_rendered = multiplied.to_string();
    assert!(added_rendered.contains("x"));
    assert!(added_rendered.contains("y"));
    assert!(added_rendered.contains("other"));
    assert!(multiplied_rendered.contains("x"));
    assert!(multiplied_rendered.contains("y"));
    assert!(multiplied_rendered.contains("other"));

    let nested_power = NumericExpression::power(
        &NumericExpression::power(&x, &NumericExpression::constant(Number::integer(2))),
        &NumericExpression::constant(Number::integer(3)),
    );
    assert!(nested_power.to_string().contains("6"));

    let distributed_power = NumericExpression::power(
        &(x.clone() * y.clone()),
        &NumericExpression::constant(Number::integer(2)),
    );
    assert!(distributed_power.to_string().contains("x"));
    assert!(distributed_power.to_string().contains("y"));

    let flattened = (x.clone() + NumericExpression::constant(Number::integer(1))).flatten();
    assert!(flattened.to_string().contains("x"));

    let mut factor_target = left.clone();
    factor_target.factor();
    assert_eq!(factor_target.to_string(), left.to_string());

    let piecewise_sub = left.substitute(&numeric_var("y"), Some(&NumericExpression::constant(Number::integer(9))));
    assert_eq!(piecewise_sub.to_string(), left.to_string());
}

#[test]
fn numeric_expression_covers_none_otherwise_and_empty_bucket_substitution_defaults() {
    let x = NumericExpression::variable(numeric_var("x"));
    let piecewise = NumericExpression::piecewise(
        vec![(
            LogicalExpression::constant(true),
            NumericExpression::piecewise(
                vec![(LogicalExpression::constant(true), x.clone())],
                Some(NumericExpression::constant(Number::integer(3))),
            ),
        )],
        None,
    );
    let rendered = piecewise.to_string();
    assert!(rendered.contains("x"));
    assert!(!rendered.contains("other;"));

    let empty_add = NumericExpression::Addition(exprion_core::numeric_bucket![]);
    let empty_mul = NumericExpression::Multiplication(exprion_core::numeric_bucket![]);
    assert_eq!(
        empty_add
            .substitute(&numeric_var("x"), Some(&NumericExpression::constant(Number::integer(1))))
            .to_string(),
        "0"
    );
    assert_eq!(
        empty_mul
            .substitute(&numeric_var("x"), Some(&NumericExpression::constant(Number::integer(1))))
            .to_string(),
        "1"
    );
}

#[test]
fn numeric_expression_covers_one_sided_piecewise_and_bucket_mixing_paths() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let cond = LogicalExpression::relation(
        &x,
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    );
    let piecewise = NumericExpression::piecewise(
        vec![(cond.clone(), x.clone())],
        Some(NumericExpression::constant(Number::integer(2))),
    );

    let add_left = NumericExpression::addition(&piecewise, &y);
    let add_right = NumericExpression::addition(&y, &piecewise);
    let mul_left = NumericExpression::multiplication(&piecewise, &y);
    let mul_right = NumericExpression::multiplication(&y, &piecewise);
    assert!(add_left.to_string().contains("y"));
    assert!(add_right.to_string().contains("y"));
    assert!(mul_left.to_string().contains("y"));
    assert!(mul_right.to_string().contains("y"));

    let add_addition = NumericExpression::addition(
        &NumericExpression::Addition(exprion_core::numeric_bucket![x.clone()]),
        &NumericExpression::constant(Number::integer(3)),
    );
    assert!(add_addition.to_string().contains("3"));

    let mul_mult = NumericExpression::multiplication(
        &NumericExpression::Multiplication(exprion_core::numeric_bucket![x.clone()]),
        &NumericExpression::constant(Number::integer(3)),
    );
    assert!(mul_mult.to_string().contains("3"));

    let neg_piecewise = NumericExpression::negation(&piecewise);
    assert!(neg_piecewise.to_string().contains("-("));
    let neg_power = NumericExpression::negation(&NumericExpression::power(
        &x,
        &NumericExpression::constant(Number::integer(2)),
    ));
    assert!(neg_power.to_string().contains("-("));
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
fn logical_expression_covers_remaining_not_or_and_substitute_paths() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let flag = LogicalExpression::variable(bool_var("flag"));
    let other_flag = LogicalExpression::variable(bool_var("other"));

    assert_eq!(LogicalExpression::not(&LogicalExpression::constant(true)).to_string(), "false");
    assert_eq!(LogicalExpression::not(&LogicalExpression::not(&flag)).to_string(), "flag");
    assert!(LogicalExpression::not(&(flag.clone() & other_flag.clone()))
        .to_string()
        .contains("OR"));
    assert!(LogicalExpression::not(&(flag.clone() | other_flag.clone()))
        .to_string()
        .contains("AND"));

    for (op, expected) in [
        (Relation::Equal, "!="),
        (Relation::NotEqual, "=="),
        (Relation::LessThan, ">="),
        (Relation::GreaterThan, "<="),
        (Relation::LessEqual, ">"),
        (Relation::GreaterEqual, "<"),
    ] {
        let relation = LogicalExpression::Relation {
            left: Box::new(x.clone()),
            operator: Symbol::Relation(op),
            right: Box::new(y.clone()),
        };
        assert!(LogicalExpression::not(&relation).to_string().contains(expected));
    }

    assert_eq!(
        LogicalExpression::and(&LogicalExpression::constant(false), &flag).to_string(),
        "(false AND flag)"
    );
    assert_eq!(
        LogicalExpression::or(&LogicalExpression::constant(true), &flag).to_string(),
        "(true OR flag)"
    );
    assert!(LogicalExpression::and(&(flag.clone() & flag.clone()), &flag)
        .to_string()
        .contains("AND"));
    assert!(LogicalExpression::or(&(flag.clone() | flag.clone()), &flag)
        .to_string()
        .contains("OR"));

    let untouched = flag.substitute(&numeric_var("x"), None);
    assert_eq!(untouched.to_string(), "flag");
}

#[test]
fn logical_expression_covers_error_and_grouping_edge_cases() {
    let x = NumericExpression::variable(numeric_var("x"));
    let y = NumericExpression::variable(numeric_var("y"));
    let flag = LogicalExpression::variable(bool_var("flag"));
    let other = LogicalExpression::variable(bool_var("other"));

    let and_group = LogicalExpression::and(&(flag.clone() & other.clone()), &(flag.clone() & other.clone()));
    assert!(and_group.to_string().contains("AND"));

    let or_group = LogicalExpression::or(&(flag.clone() | other.clone()), &(flag.clone() | other.clone()));
    assert!(or_group.to_string().contains("OR"));

    let none_sub = LogicalExpression::relation(&x, &Symbol::Relation(Relation::Equal), &y)
        .substitute(&bool_var("missing"), None);
    assert!(none_sub.to_string().contains("x"));

    assert!(catch_unwind(AssertUnwindSafe(|| {
        LogicalExpression::not(&LogicalExpression::Relation {
            left: Box::new(x.clone()),
            operator: Symbol::Binary(exprion_core::lexer::symbol::Binary::Add),
            right: Box::new(y.clone()),
        })
    }))
    .is_err());

    assert!(catch_unwind(AssertUnwindSafe(|| {
        flag.substitute(
            &bool_var("flag"),
            Some(&SemanticExpression::numeric(NumericExpression::constant(Number::integer(1)))),
        )
    }))
    .is_err());

    let empty_and = LogicalExpression::And(exprion_core::logical_bucket![]);
    let empty_or = LogicalExpression::Or(exprion_core::logical_bucket![]);
    assert_eq!(
        empty_and
            .substitute(&bool_var("flag"), Some(&SemanticExpression::logical(LogicalExpression::constant(true))))
            .to_string(),
        "true"
    );
    assert_eq!(
        empty_or
            .substitute(&bool_var("flag"), Some(&SemanticExpression::logical(LogicalExpression::constant(true))))
            .to_string(),
        "false"
    );
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
