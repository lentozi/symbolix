use crate::lexer::constant::Number;
use crate::numeric_bucket;
use crate::semantic::semantic_ir::numeric::NumericExpression;

pub enum Term {
    AdditionTerm(AdditionTerm),
    MultiplyTerm(MultiplyTerm),
}

pub struct AdditionTerm {
    pub coef: Number,
    pub core: NumericExpression,
}

pub struct MultiplyTerm {
    pub base: NumericExpression,
    pub exponent: Number,
}

impl AdditionTerm {
    pub fn new(coef: Number, core: NumericExpression) -> Self {
        AdditionTerm { coef, core }
    }
}

impl MultiplyTerm {
    pub fn new(base: NumericExpression, exponent: Number) -> Self {
        MultiplyTerm { base, exponent }
    }
}

pub fn extract_addition_term(expr: NumericExpression) -> AdditionTerm {
    match expr {
        NumericExpression::Constant(c) => {
            AdditionTerm::new(Number::Integer(1), NumericExpression::Constant(c))
        }
        NumericExpression::Negation(inner) => AdditionTerm::new(Number::Integer(-1), *inner),
        NumericExpression::Multiplication(bucket) => {
            if bucket.contains_constant() {
                let coef: Number = bucket.get_constants().into_iter().sum();
                let bucket = bucket.without_constants();
                AdditionTerm::new(
                    coef,
                    NumericExpression::Multiplication(bucket.without_constants()),
                )
            } else {
                AdditionTerm::new(
                    Number::Integer(1),
                    NumericExpression::Multiplication(bucket),
                )
            }
        }
        _ => AdditionTerm::new(Number::Integer(1), expr),
    }
}

pub fn rebuild_addition_term(term: AdditionTerm) -> NumericExpression {
    if term.coef == 1 {
        term.core
    } else if term.coef != 0 {
        let bucket = numeric_bucket![NumericExpression::Constant(term.coef), term.core];

        NumericExpression::Multiplication(bucket)
    } else {
        NumericExpression::Constant(Number::Integer(0))
    }
}

pub fn extract_multiply_term(expr: NumericExpression) -> MultiplyTerm {
    match expr {
        NumericExpression::Constant(c) => {
            MultiplyTerm::new(NumericExpression::Constant(c), Number::Integer(1))
        }
        _ => MultiplyTerm::new(expr, Number::Integer(1)),
    }
}

pub fn rebuild_multiply_term(term: MultiplyTerm) -> NumericExpression {
    if term.exponent == 1 {
        term.base
    } else if term.exponent != 0 {
        NumericExpression::Power {
            base: Box::new(term.base),
            exponent: Box::new(NumericExpression::Constant(term.exponent)),
        }
    } else {
        NumericExpression::Constant(Number::Integer(1))
    }
}
