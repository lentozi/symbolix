use symbolix_compile::symbolix;

#[test]
fn solve_accepts_explicit_target_variable() {
    let compiled = symbolix! {
        let x = var!("x", f64);
        let a = expr!("a");
        let lhs = x + a;
        let rhs = expr!("0");
        let equation = lhs.equal_to(rhs);
        solve!(equation, x)
    };

    let result = compiled.calculate(4.0);
    assert_eq!(result, -4.0);
}

#[test]
fn symbolix_supports_if_blocks_and_tuple_returns() {
    let compiled = symbolix! {
        let x = var!("x", f64);
        let y = var!("y", f64);
        let expr = if x.greater_than(y) {
            x + y
        } else {
            x - y
        };
        (expr, x.greater_equal(y))
    };

    assert_eq!(compiled.calculate(3.0, 1.0), (4.0, true));
    assert_eq!(compiled.calculate(1.0, 3.0), (-2.0, false));
}

#[test]
fn symbolix_supports_boolean_variable_conditions() {
    let compiled = symbolix! {
        let cond = var!("cond", bool);
        let x = var!("x", f64);
        let expr = if cond {
            x + 1
        } else {
            x - 1
        };
        expr
    };

    assert_eq!(compiled.calculate(true, 3.0), 4.0);
    assert_eq!(compiled.calculate(false, 3.0), 2.0);
}

#[test]
fn symbolix_supports_expr_macro_and_relation_methods() {
    let compiled = symbolix! {
        let x = var!("x", f64);
        let rhs = expr!("x + 1");
        x.less_than(rhs)
    };

    assert!(compiled.calculate(10.0));
}

#[test]
fn solution_set_does_not_duplicate_branch_constraints() {
    let compiled = symbolix! {
        let cond = var!("cond", bool);
        let y = var!("z", f64);

        let expr = if cond {
            expr!("z - 20")
        } else {
            expr!("2 * z")
        };

        let expr = expr + y;
        let equation = expr.equal_to(y);
        solve!(equation, y)
    };

    assert_eq!(compiled.calculate(true), 20.0);
    assert_eq!(compiled.calculate(false), 0.0);
}

#[test]
fn solve_returns_vector_for_multiple_roots() {
    let compiled = symbolix! {
        let x = var!("x", f64);
        let lhs = expr!("x ^ 2 - 1");
        let rhs = expr!("0");
        let equation = lhs.equal_to(rhs);
        solve!(equation, x)
    };

    let mut result = compiled.calculate();
    result.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(result, vec![-1.0, 1.0]);
}

#[test]
fn symbolix_can_return_boolean_and_numeric_tuple_without_free_vars() {
    let compiled = symbolix! {
        let x = var!("x", f64);
        let zero = expr!("0");
        let check = x.greater_than(zero);
        let shifted = x + 2;
        (check, shifted)
    };

    assert_eq!(compiled.calculate(3.0), (true, 5.0));
    assert_eq!(compiled.calculate(-1.0), (false, 1.0));
}

#[test]
fn symbolix_solve_without_explicit_target_infers_single_variable() {
    let compiled = symbolix! {
        let x = var!("x", f64);
        let lhs = expr!("2 * x + 4");
        let rhs = expr!("0");
        let equation = lhs.equal_to(rhs);
        solve!(equation)
    };

    assert_eq!(compiled.calculate(), -2.0);
}
