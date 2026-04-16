use symbolix_compile::formula;

#[test]
fn formula_generates_numeric_calculator() {
    let compiled = formula!("x == 50 ? 4 : 5");

    assert_eq!(compiled.calculate(50.0), 4.0);
    assert_eq!(compiled.calculate(12.0), 5.0);
}

#[test]
fn formula_closure_keeps_argument_order_stable() {
    let compiled = formula!("a + b * 2");
    let closure = compiled.to_closure();

    assert_eq!(closure(3.0, 4.0), 11.0);
}
