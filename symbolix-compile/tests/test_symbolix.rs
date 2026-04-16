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
    let rendered = format!("{}", result);
    assert!(rendered.contains("a"));
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
fn symbolix_supports_expr_macro_and_relation_methods() {
    let compiled = symbolix! {
        let x = var!("x", f64);
        let rhs = expr!("x + 1");
        x.less_than(rhs)
    };

    assert!(compiled.calculate(10.0));
}
