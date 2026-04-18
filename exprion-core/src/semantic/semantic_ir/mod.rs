pub mod logic;
mod macros;
pub mod numeric;

use crate::lexer::constant::Number;
use crate::lexer::symbol::{Relation, Symbol};
use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression::Constant;
use crate::{
    impl_expr_binary_operation, impl_expr_logic_operation, impl_expr_numeric_operation,
    impl_expr_unary_operation,
};
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, BitAnd, BitOr, Div, Mul, Neg, Not, Sub};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum SemanticExpression {
    Numeric(NumericExpression),
    Logical(LogicalExpression),
}

impl fmt::Display for SemanticExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SemanticExpression::Numeric(num_expr) => write!(f, "{}", num_expr),
            SemanticExpression::Logical(log_expr) => write!(f, "{}", log_expr),
        }
    }
}

impl SemanticExpression {
    pub fn numeric(expr: NumericExpression) -> SemanticExpression {
        SemanticExpression::Numeric(expr)
    }

    pub fn logical(expr: LogicalExpression) -> SemanticExpression {
        SemanticExpression::Logical(expr)
    }

    pub fn as_numeric(&self) -> &NumericExpression {
        match self {
            SemanticExpression::Numeric(expr) => expr,
            SemanticExpression::Logical(_) => {
                panic!("expected numeric expression")
            }
        }
    }

    pub fn as_logical(&self) -> &LogicalExpression {
        match self {
            SemanticExpression::Logical(expr) => expr,
            SemanticExpression::Numeric(_) => {
                panic!("expected logical expression")
            }
        }
    }

    pub fn negation(expr: &SemanticExpression) -> SemanticExpression {
        SemanticExpression::numeric(-expr.as_numeric())
    }

    pub fn addition(term1: &SemanticExpression, term2: &SemanticExpression) -> SemanticExpression {
        SemanticExpression::numeric(term1.as_numeric() + term2.as_numeric())
    }

    pub fn subtraction(
        minuend: &SemanticExpression,
        subtrahend: &SemanticExpression,
    ) -> SemanticExpression {
        SemanticExpression::numeric(minuend.as_numeric() - subtrahend.as_numeric())
    }

    pub fn multiplication(
        factor1: &SemanticExpression,
        factor2: &SemanticExpression,
    ) -> SemanticExpression {
        SemanticExpression::numeric(factor1.as_numeric() * factor2.as_numeric())
    }

    pub fn division(
        dividend: &SemanticExpression,
        divisor: &SemanticExpression,
    ) -> SemanticExpression {
        SemanticExpression::numeric(dividend.as_numeric() / divisor.as_numeric())
    }

    pub fn power(base: &SemanticExpression, exponent: &SemanticExpression) -> SemanticExpression {
        SemanticExpression::numeric(NumericExpression::power(
            base.as_numeric(),
            exponent.as_numeric(),
        ))
    }

    pub fn pow(&self, exponent: &SemanticExpression) -> SemanticExpression {
        SemanticExpression::numeric(NumericExpression::power(
            self.as_numeric(),
            exponent.as_numeric(),
        ))
    }

    pub fn not(expr: &SemanticExpression) -> SemanticExpression {
        SemanticExpression::logical(!expr.as_logical())
    }

    pub fn and(expr1: &SemanticExpression, expr2: &SemanticExpression) -> SemanticExpression {
        SemanticExpression::logical(expr1.as_logical() & expr2.as_logical())
    }

    pub fn or(expr1: &SemanticExpression, expr2: &SemanticExpression) -> SemanticExpression {
        SemanticExpression::logical(expr1.as_logical() | expr2.as_logical())
    }

    pub fn one() -> SemanticExpression {
        SemanticExpression::Numeric(Constant(Number::Integer(1)))
    }

    pub fn substitute(
        &self,
        target: &crate::semantic::variable::Variable,
        replacement: Option<&SemanticExpression>,
    ) -> SemanticExpression {
        match self {
            SemanticExpression::Numeric(expr) => {
                let numeric_replacement = match replacement {
                    Some(SemanticExpression::Numeric(numeric)) => Some(numeric),
                    Some(SemanticExpression::Logical(_))
                        if target.var_type != crate::semantic::variable::VariableType::Boolean =>
                    {
                        panic!("cannot substitute a numeric variable with a logical expression")
                    }
                    _ => None,
                };
                SemanticExpression::numeric(expr.substitute(target, numeric_replacement))
            }
            SemanticExpression::Logical(expr) => {
                SemanticExpression::logical(expr.substitute(target, replacement))
            }
        }
    }
}

impl SemanticExpression {
    pub fn is_numeric(&self) -> bool {
        match self {
            SemanticExpression::Numeric(_) => true,
            _ => false,
        }
    }

    pub fn is_logical(&self) -> bool {
        match self {
            SemanticExpression::Logical(_) => true,
            _ => false,
        }
    }

    pub fn is_equation(&self) -> bool {
        match self {
            SemanticExpression::Logical(LogicalExpression::Relation {
                operator: Symbol::Relation(Relation::Equal),
                ..
            }) => true,
            _ => false,
        }
    }
}

impl_expr_binary_operation!(Add, add, addition);
impl_expr_binary_operation!(Sub, sub, subtraction);
impl_expr_binary_operation!(Mul, mul, multiplication);
impl_expr_binary_operation!(Div, div, division);
impl_expr_binary_operation!(BitAnd, bitand, and);
impl_expr_binary_operation!(BitOr, bitor, or);

impl_expr_unary_operation!(Neg, neg, negation);
impl_expr_unary_operation!(Not, not, not);

impl_expr_numeric_operation!(Add, add, addition, i32, i64, f32, f64, u32, u64);
impl_expr_numeric_operation!(Sub, sub, subtraction, i32, i64, f32, f64, u32, u64);
impl_expr_numeric_operation!(Mul, mul, multiplication, i32, i64, f32, f64, u32, u64);
impl_expr_numeric_operation!(Div, div, division, i32, i64, f32, f64, u32, u64);

impl_expr_logic_operation!(BitAnd, bitand, and, bool);
impl_expr_logic_operation!(BitOr, bitor, or, bool);

