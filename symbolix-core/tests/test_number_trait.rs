use symbolix_core::lexer::constant::Number;

#[test]
fn common_build_accepts_all_supported_numeric_primitives() {
    assert_eq!(Number::common_build(3_i32).to_string(), "3");
    assert_eq!(Number::common_build(4_i64).to_string(), "4");
    assert_eq!(Number::common_build(5_u32).to_string(), "5");
    assert_eq!(Number::common_build(6_u64).to_string(), "6");
    assert_eq!(Number::common_build(1.5_f32).to_string(), "1.5");
    assert_eq!(Number::common_build(2.5_f64).to_string(), "2.5");
}

#[test]
fn zero_and_one_checks_work_for_all_number_shapes() {
    assert!(Number::integer(0).is_zero());
    assert!(Number::float(0.0).is_zero());
    assert!(Number::fraction(0, 3).is_zero());

    assert!(Number::integer(1).is_one());
    assert!(Number::float(1.0).is_one());
    assert!(Number::fraction(1, 1).is_one());
}
