pub mod bucket;
mod macros;
pub mod semantic_ir;
pub mod variable;

use std::panic;

use crate::lexer::constant::Constant;
use crate::lexer::symbol::Symbol;
use crate::lexer::symbol::{Binary, Ternary, Unary};
use crate::parser::expression::Expression;
use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::semantic_ir::SemanticExpression;
use crate::semantic::variable::{Variable, VariableType};

pub fn semantic_without_ctx(expr: &Expression, is_numeric: bool) -> SemanticExpression {
    match expr {
        Expression::BinaryExpression(left, operation, right) => match operation {
            Symbol::Binary(Binary::Add) => {
                let left = semantic_without_ctx(left, true);
                let right = semantic_without_ctx(right, true);
                left + right
            }
            Symbol::Binary(Binary::Subtract) => {
                let left = semantic_without_ctx(left, true);
                let right = semantic_without_ctx(right, true);
                left - right
            }
            Symbol::Binary(Binary::Multiply) => {
                let left = semantic_without_ctx(left, true);
                let right = semantic_without_ctx(right, true);
                left * right
            }
            Symbol::Binary(Binary::Divide) => {
                let left = semantic_without_ctx(left, true);
                let right = semantic_without_ctx(right, true);
                left / right
            }
            Symbol::Binary(Binary::Power) => {
                let left = semantic_without_ctx(left, true);
                let right = semantic_without_ctx(right, true);
                SemanticExpression::power(left, right)
            }
            Symbol::Binary(Binary::LogicAnd) => {
                let left = semantic_without_ctx(left, false);
                let right = semantic_without_ctx(right, false);
                SemanticExpression::and(left, right)
            }
            Symbol::Binary(Binary::LogicOr) => {
                let left = semantic_without_ctx(left, false);
                let right = semantic_without_ctx(right, false);
                SemanticExpression::or(left, right)
            }
            Symbol::Relation(_) => {
                let left = semantic_without_ctx(left, true);
                let right = semantic_without_ctx(right, true);
                match (left, right) {
                    (SemanticExpression::Numeric(left), SemanticExpression::Numeric(right)) => {
                        SemanticExpression::Logical(LogicalExpression::relation(
                            left, *operation, right,
                        ))
                    }
                    _ => panic!("relation operator applied to non-numeric expressions"),
                }
            }
            _ => panic!("invalid symbol"),
        },
        Expression::Constant(Constant::Number(ref n)) => {
            let n = (*n).clone();
            SemanticExpression::Numeric(NumericExpression::constant(n))
        }
        Expression::Constant(Constant::Boolean(ref b)) => {
            let b = (*b).clone();
            SemanticExpression::Logical(LogicalExpression::constant(b))
        }
        Expression::Variable(v) => {
            let var_type: VariableType = if is_numeric {
                VariableType::Float
            } else {
                VariableType::Boolean
            };
            let variable = Variable::new(v.as_str(), var_type, None);
            if is_numeric {
                SemanticExpression::Numeric(NumericExpression::variable(variable))
            } else {
                SemanticExpression::Logical(LogicalExpression::variable(variable))
            }
        }
        Expression::TernaryExpression(cond, symbol1, then, symbol2, otherwise) => {
            if symbol1 == &Symbol::Ternary(Ternary::Conditional)
                && symbol2 == &Symbol::Ternary(Ternary::ConditionalElse)
            {
                let otherwise_semantic = semantic_without_ctx(&*otherwise, true);
                let then_semantic = semantic_without_ctx(&*then, true);
                let cond_semantic = semantic_without_ctx(&*cond, false);

                match (cond_semantic, then_semantic, otherwise_semantic) {
                    (
                        SemanticExpression::Logical(cond),
                        SemanticExpression::Numeric(then),
                        SemanticExpression::Numeric(otherwise),
                    ) => SemanticExpression::Numeric(NumericExpression::piecewise(
                        vec![(cond, then)],
                        Some(otherwise),
                    )),
                    _ => panic!("invalid ternary expression"),
                }
            } else {
                panic!(
                    "unsupported symbols in ternary expression: {}, {}",
                    symbol1, symbol2
                );
            }
        }
        Expression::UnaryExpression(symbol, expression) => match symbol {
            Symbol::Unary(Unary::Plus) => semantic_without_ctx(expression, true),
            Symbol::Unary(Unary::Minus) => {
                let expr_semantic = semantic_without_ctx(expression, true);
                match expr_semantic {
                    SemanticExpression::Numeric(n) => {
                        SemanticExpression::Numeric(NumericExpression::negation(n))
                    }
                    _ => panic!("invalid unary expression"),
                }
            }
            Symbol::Unary(Unary::LogicNot) => {
                let expr_semantic = semantic_without_ctx(expression, false);
                match expr_semantic {
                    SemanticExpression::Logical(b) => {
                        SemanticExpression::Logical(LogicalExpression::not(b))
                    }
                    _ => panic!("invalid unary expression"),
                }
            }
            _ => panic!("unexpected unary operator: {}", symbol),
        },
        Expression::Relation(left, relation, right) => {
            let left = semantic_without_ctx(left, true);
            let right = semantic_without_ctx(right, true);
            match (left, right) {
                (SemanticExpression::Numeric(left), SemanticExpression::Numeric(right)) => {
                    SemanticExpression::Logical(LogicalExpression::relation(left, *relation, right))
                }
                _ => panic!("relation operator applied to non-numeric expressions"),
            }
        }
    }
}
