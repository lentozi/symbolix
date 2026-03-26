pub mod logic;
mod macros;
pub mod numeric;

use crate::lexer::constant::Number;
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
use tree_drawer::tree::OwnedTree;
use crate::lexer::symbol::{Relation, Symbol};
use crate::equation::linear::LinearEquation;

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

    pub fn negation(expr: SemanticExpression) -> SemanticExpression {
        match expr {
            SemanticExpression::Numeric(n) => {
                SemanticExpression::numeric(NumericExpression::negation(n))
            }
            _ => panic!("negation is only defined for numeric expressions"),
        }
    }

    pub fn addition(term1: SemanticExpression, term2: SemanticExpression) -> SemanticExpression {
        match (term1, term2) {
            (SemanticExpression::Numeric(n1), SemanticExpression::Numeric(n2)) => {
                SemanticExpression::numeric(NumericExpression::addition(n1, n2))
            }
            _ => panic!("addition is only defined for numeric expressions"),
        }
    }

    pub fn subtraction(
        minuend: SemanticExpression,
        subtrahend: SemanticExpression,
    ) -> SemanticExpression {
        match (minuend, subtrahend) {
            (SemanticExpression::Numeric(n1), SemanticExpression::Numeric(n2)) => {
                SemanticExpression::numeric(NumericExpression::subtraction(n1, n2))
            }
            _ => panic!("subtraction is only defined for numeric expressions"),
        }
    }

    pub fn multiplication(
        factor1: SemanticExpression,
        factor2: SemanticExpression,
    ) -> SemanticExpression {
        match (factor1, factor2) {
            (SemanticExpression::Numeric(n1), SemanticExpression::Numeric(n2)) => {
                SemanticExpression::numeric(NumericExpression::multiplication(n1, n2))
            }
            _ => panic!("multiplication is only defined for numeric expressions"),
        }
    }

    pub fn division(
        dividend: SemanticExpression,
        divisor: SemanticExpression,
    ) -> SemanticExpression {
        match (dividend, divisor) {
            (SemanticExpression::Numeric(n1), SemanticExpression::Numeric(n2)) => {
                SemanticExpression::numeric(NumericExpression::division(n1, n2))
            }
            _ => panic!("division is only defined for numeric expressions"),
        }
    }

    pub fn power(base: SemanticExpression, exponent: SemanticExpression) -> SemanticExpression {
        match (base, exponent) {
            (SemanticExpression::Numeric(n1), SemanticExpression::Numeric(n2)) => {
                SemanticExpression::numeric(NumericExpression::power(n1, n2))
            }
            _ => panic!("power is only defined for numeric expressions"),
        }
    }

    pub fn pow(&self, exponent: SemanticExpression) -> SemanticExpression {
        match (self, exponent) {
            (SemanticExpression::Numeric(n1), SemanticExpression::Numeric(n2)) => {
                SemanticExpression::numeric(NumericExpression::power(n1.clone(), n2))
            }
            _ => panic!("power is only defined for numeric expressions"),
        }
    }

    pub fn not(expr: SemanticExpression) -> SemanticExpression {
        match expr {
            SemanticExpression::Logical(l) => {
                SemanticExpression::logical(LogicalExpression::not(l))
            }
            _ => panic!("not is only defined for logical expressions"),
        }
    }

    pub fn and(expr1: SemanticExpression, expr2: SemanticExpression) -> SemanticExpression {
        match (expr1, expr2) {
            (SemanticExpression::Logical(l1), SemanticExpression::Logical(l2)) => {
                SemanticExpression::logical(LogicalExpression::and(l1, l2))
            }
            _ => panic!("and is only defined for logical expressions"),
        }
    }

    pub fn or(expr1: SemanticExpression, expr2: SemanticExpression) -> SemanticExpression {
        match (expr1, expr2) {
            (SemanticExpression::Logical(l1), SemanticExpression::Logical(l2)) => {
                SemanticExpression::logical(LogicalExpression::or(l1, l2))
            }
            _ => panic!("or is only defined for logical expressions"),
        }
    }

    pub fn one() -> SemanticExpression {
        SemanticExpression::Numeric(Constant(Number::Integer(1)))
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
            SemanticExpression::Logical(
                LogicalExpression::Relation {operator: Symbol::Relation(Relation::Equal), .. }
            ) => true,
            _ => false,
        }
    }

    pub fn to_linear(&self) -> Option<LinearEquation> {
        match self {
            SemanticExpression::Numeric(numeric) => {
                match numeric {
                    Constant(c) => {}
                    NumericExpression::Variable(_) => {}
                    NumericExpression::Negation(_) => {}
                    NumericExpression::Addition(_) => {}
                    NumericExpression::Multiplication(_) => {}
                    NumericExpression::Power { .. } => {}
                    NumericExpression::Piecewise { .. } => {}
                }
            }
            _ => None
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


impl SemanticExpression {
    pub fn to_owned_tree(&self) -> OwnedTree {
        match self {
            SemanticExpression::Numeric(expr) => expr.to_owned_tree(),
            SemanticExpression::Logical(expr) => expr.to_owned_tree(),
        }
    }
}