use std::panic::{catch_unwind, AssertUnwindSafe};

use symbolix_core::lexer::constant::{Constant, Fraction, Number};

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

#[test]
fn number_and_constant_operator_impls_cover_owned_and_borrowed_variants() {
    let a = Number::integer(3);
    let b = Number::fraction(1, 2);

    assert_eq!((a.clone() + b.clone()).to_string(), "7/2");
    assert_eq!((a.clone() + &b).to_string(), "7/2");
    assert_eq!((&a + b.clone()).to_string(), "7/2");
    assert_eq!((&a + &b).to_string(), "7/2");
    assert_eq!((a.clone() - b.clone()).to_string(), "5/2");
    assert_eq!((a.clone() * b.clone()).to_string(), "3/2");
    assert_eq!((a.clone() / b.clone()).to_string(), "6");
    assert_eq!((-&a).to_string(), "-3");

    let c1 = Constant::integer(3);
    let c2 = Constant::fraction(1, 2);
    assert_eq!((c1.clone() + c2.clone()).to_string(), "7/2");
    assert_eq!((c1.clone() + &c2).to_string(), "7/2");
    assert_eq!((&c1 + c2.clone()).to_string(), "7/2");
    assert_eq!((&c1 + &c2).to_string(), "7/2");
    assert_eq!((c1.clone() - c2.clone()).to_string(), "5/2");
    assert_eq!((c1.clone() * c2.clone()).to_string(), "3/2");
    assert_eq!((c1.clone() / c2.clone()).to_string(), "6");
    assert_eq!((-&c1).to_string(), "-3");
}

#[test]
fn fraction_helpers_and_number_comparisons_cover_remaining_paths() {
    let mut negative_denom = Fraction::new(2, -4);
    negative_denom.simplify();
    assert_eq!(negative_denom.numerator, -2);
    assert_eq!(negative_denom.denominator, -4);

    let mut zero = Fraction::new(0, 7);
    zero.simplify();
    assert_eq!(zero.denominator, 1);
    assert_eq!(zero.to_latex(), "\\frac{0}{1}");

    let mut reducible = Fraction::new(6, 8);
    reducible.simplify();
    assert_eq!(reducible.numerator, 3);
    assert_eq!(reducible.denominator, 4);
    assert_eq!(reducible.to_latex(), "\\frac{3}{4}");

    let sum: Number = vec![Number::integer(1), Number::integer(2), Number::integer(3)]
        .into_iter()
        .sum();
    let product: Number = vec![Number::integer(2), Number::integer(3), Number::integer(4)]
        .into_iter()
        .product();
    assert_eq!(sum.to_string(), "6");
    assert_eq!(product.to_string(), "24");

    assert_eq!(Number::integer(3), 3_i64);
    assert!(Number::float(3.5) > 3_i64);
    assert!(Number::fraction(7, 2) > 3_i64);
}

#[test]
fn constant_boolean_and_invalid_numeric_operations_cover_panics() {
    assert_eq!(Constant::boolean(true).to_string(), "true");
    assert_eq!(Constant::number(Number::integer(4)).to_string(), "4");
    assert_eq!(Constant::float(1.25).to_string(), "1.25");

    for panic in [
        catch_unwind(AssertUnwindSafe(|| Constant::boolean(true).negation())),
        catch_unwind(AssertUnwindSafe(|| Constant::boolean(true).addition(&Constant::integer(1)))),
        catch_unwind(AssertUnwindSafe(|| Constant::integer(1).subtraction(&Constant::boolean(false)))),
        catch_unwind(AssertUnwindSafe(|| Constant::boolean(true).multiplication(&Constant::boolean(false)))),
        catch_unwind(AssertUnwindSafe(|| Constant::boolean(true).division(&Constant::integer(1)))),
    ] {
        assert!(panic.is_err());
    }
}
