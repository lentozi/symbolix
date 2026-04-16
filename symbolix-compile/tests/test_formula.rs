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

#[test]
fn formula_supports_boolean_results_and_zero_argument_expressions() {
    let compiled = formula!("x > 100");
    assert!(!compiled.calculate(99.0));
    assert!(compiled.calculate(120.0));

    let constant = formula!("1 + 2 * 3");
    assert_eq!(constant.calculate(), 7.0);
}

#[test]
fn formula_argument_order_is_alphabetical_not_appearance_order() {
    let compiled = formula!("z + x * 10 + y");
    assert_eq!(compiled.calculate(2.0, 3.0, 5.0), 28.0);
}
