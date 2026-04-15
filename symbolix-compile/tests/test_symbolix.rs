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
