use crate::lexer::constant::Number;
use crate::numeric_bucket;
use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression;

pub struct AdditionTerm {
    pub coef: Number,
    pub core: NumericExpression,
}

pub struct MultiplyTerm {
    pub base: NumericExpression,
    pub exponent: Number,
}

pub struct LogicalTerm {
    pub base: LogicalExpression,
    pub is_not: bool,
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

impl LogicalTerm {
    pub fn new(base: LogicalExpression, is_not: bool) -> Self {
        LogicalTerm { base, is_not }
    }
}

pub fn extract_addition_term(expr: NumericExpression) -> AdditionTerm {
    match expr {
        NumericExpression::Constant(c) => {
            AdditionTerm::new(Number::Integer(1), NumericExpression::Constant(c))
        }
        NumericExpression::Negation(inner) => AdditionTerm::new(Number::Integer(-1), *inner),
        NumericExpression::Multiplication(mut bucket) => {
            if bucket.contains_constant() {
                let coef: Number = bucket.drain_constants().sum();
                let bucket = bucket.without_constants_owned();
                if bucket.len() == 1 {
                    let expr = bucket
                        .into_iter()
                        .next()
                        .expect("single-item bucket must yield one expression");
                    AdditionTerm::new(coef, expr)
                } else {
                    AdditionTerm::new(coef, NumericExpression::Multiplication(bucket))
                }
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

pub fn extract_logical_term(expr: LogicalExpression) -> LogicalTerm {
    match expr {
        LogicalExpression::Not(n) => LogicalTerm::new(*n, true),
        _ => LogicalTerm::new(expr, false),
    }
}

pub fn rebuild_logical_term(term: LogicalTerm) -> LogicalExpression {
    if term.is_not {
        LogicalExpression::Not(Box::new(term.base))
    } else {
        term.base
    }
}
