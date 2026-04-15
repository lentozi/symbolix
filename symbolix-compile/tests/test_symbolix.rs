use symbolix_compile::symbolix;

#[test]
fn solve_infers_target_variable() {
    let compiled = symbolix! {
        let lhs = expr!("2 * x + 2");
        let rhs = expr!("0");
        let equation = lhs.equal_to(rhs);
        equation.solve()
    };

    let result = compiled.calculate();
    let rendered = format!("{}", result);
    assert!(rendered.contains("-1"));
}

#[test]
fn solve_accepts_explicit_target_variable() {
    let compiled = symbolix! {
        let x = var!("x", f64);
        let a = expr!("a");
        let lhs = x + a;
        let rhs = expr!("0");
        let equation = lhs.equal_to(rhs);
        equation.solve(x)
    };

    let result = compiled.calculate(4.0);
    let rendered = format!("{}", result);
    assert!(rendered.contains("a"));
}
