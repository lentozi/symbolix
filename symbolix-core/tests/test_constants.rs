use symbolix_core::lexer::constant::{Constant, Number};

#[test]
fn fraction_numbers_normalize_and_convert() {
    let fraction = Number::fraction(2, 4);
    assert_eq!(fraction.to_string(), "2/4");
    assert_eq!(fraction.to_float(), 0.5);
    assert_eq!(fraction.to_integer(), None);

    let whole = Number::fraction(8, 4);
    assert_eq!(whole.to_integer(), None);
    assert_eq!(whole.to_float(), 2.0);
    assert!(!whole.is_zero());
    assert!(!whole.is_one());
}

#[test]
fn number_arithmetic_covers_integer_float_and_fraction_paths() {
    assert_eq!(
        Number::addition(&Number::integer(2), &Number::fraction(1, 2)).to_string(),
        "5/2"
    );
    assert_eq!(
        Number::subtraction(&Number::float(3.5), &Number::integer(1)).to_string(),
        "2.5"
    );
    assert_eq!(
        Number::multiplication(&Number::fraction(2, 3), &Number::fraction(9, 4)).to_string(),
        "3/2"
    );
    assert_eq!(
        Number::division(&Number::integer(3), &Number::fraction(3, 2)).to_string(),
        "2"
    );
    assert_eq!(Number::negation(&Number::integer(5)).to_string(), "-5");
}

#[test]
fn constants_delegate_to_number_operations() {
    let left = Constant::fraction(1, 2);
    let right = Constant::integer(2);

    assert_eq!(left.addition(&right).to_string(), "5/2");
    assert_eq!(left.subtraction(&right).to_string(), "-3/2");
    assert_eq!(left.multiplication(&right).to_string(), "1");
    assert_eq!(right.division(&left).to_string(), "4");
    assert_eq!(left.negation().to_string(), "-1/2");
}
