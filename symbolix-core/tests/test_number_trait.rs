use symbolix_core::lexer::{constant::Number, NumberTrait};

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

#[test]
fn number_trait_reports_integer_float_and_conversion_across_primitives() {
    assert!(3.0_f64.is_integer());
    assert!(!3.5_f64.is_integer());
    assert!(3.5_f64.is_float());
    assert_eq!(3.5_f64.to_integer(), 3);
    assert_eq!(3.5_f64.to_float(), 3.5);

    assert!(2.0_f32.is_integer());
    assert!(!2.25_f32.is_integer());
    assert!(2.25_f32.is_float());
    assert_eq!(2.25_f32.to_integer(), 2);

    assert!(5_i64.is_integer());
    assert!(!5_i64.is_float());
    assert_eq!(5_i64.to_integer(), 5);
    assert_eq!(5_i64.to_float(), 5.0);

    assert!(6_i32.is_integer());
    assert!(!6_i32.is_float());
    assert_eq!(6_i32.to_integer(), 6);

    assert!(7_u64.is_integer());
    assert!(!7_u64.is_float());
    assert_eq!(7_u64.to_integer(), 7);

    assert!(8_u32.is_integer());
    assert!(!8_u32.is_float());
    assert_eq!(8_u32.to_integer(), 8);
}
