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
use crate::semantic::variable::VariableType;
use crate::with_compile_context;

pub struct Analyzer {
    is_numeric: bool,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer { is_numeric: true }
    }

    pub fn analyze_with_ctx(&mut self, expr: &Expression) -> SemanticExpression {
        let semantic = semantic_with_ctx(expr, self.is_numeric);
        self.is_numeric = match semantic {
            SemanticExpression::Numeric(_) => true,
            SemanticExpression::Logical(_) => false,
        };

        semantic
    }

    pub fn is_numeric(&self) -> bool {
        self.is_numeric
    }
}

fn semantic_with_ctx(expr: &Expression, is_numeric: bool) -> SemanticExpression {
    match expr {
        Expression::BinaryExpression(left, operation, right) => match operation {
            Symbol::Binary(Binary::Add) => numeric_binary(left, right, SemanticExpression::addition),
            Symbol::Binary(Binary::Subtract) => {
                numeric_binary(left, right, SemanticExpression::subtraction)
            }
            Symbol::Binary(Binary::Multiply) => {
                numeric_binary(left, right, SemanticExpression::multiplication)
            }
            Symbol::Binary(Binary::Divide) => {
                numeric_binary(left, right, SemanticExpression::division)
            }
            Symbol::Binary(Binary::Power) => numeric_binary(left, right, SemanticExpression::power),
            Symbol::Binary(Binary::LogicAnd) => {
                logical_binary(left, right, SemanticExpression::and)
            }
            Symbol::Binary(Binary::LogicOr) => logical_binary(left, right, SemanticExpression::or),
            Symbol::Relation(_) => relation_expression(left, operation, right),
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
                // TODO 暂时将数值计算中缺少类型声明的变量声明为 f64
                VariableType::Float
            } else {
                VariableType::Boolean
            };
            let variable = with_compile_context!(ctx, ctx.resolve_variable(v, var_type));

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
                let otherwise_semantic = semantic_with_ctx(&*otherwise, true);
                let then_semantic = semantic_with_ctx(&*then, true);
                let cond_semantic = semantic_with_ctx(&*cond, false);

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
            Symbol::Unary(Unary::Plus) => semantic_with_ctx(expression, true),
            Symbol::Unary(Unary::Minus) => numeric_unary(expression, SemanticExpression::negation),
            Symbol::Unary(Unary::LogicNot) => logical_unary(expression, SemanticExpression::not),
            _ => panic!("unexpected unary operator: {}", symbol),
        },
        Expression::Relation(left, relation, right) => relation_expression(left, relation, right),
    }
}

fn numeric_binary(
    left: &Expression,
    right: &Expression,
    op: fn(&SemanticExpression, &SemanticExpression) -> SemanticExpression,
) -> SemanticExpression {
    let left = semantic_with_ctx(left, true);
    let right = semantic_with_ctx(right, true);
    op(&left, &right)
}

fn logical_binary(
    left: &Expression,
    right: &Expression,
    op: fn(&SemanticExpression, &SemanticExpression) -> SemanticExpression,
) -> SemanticExpression {
    let left = semantic_with_ctx(left, false);
    let right = semantic_with_ctx(right, false);
    op(&left, &right)
}

fn numeric_unary(
    expr: &Expression,
    op: fn(&SemanticExpression) -> SemanticExpression,
) -> SemanticExpression {
    let value = semantic_with_ctx(expr, true);
    op(&value)
}

fn logical_unary(
    expr: &Expression,
    op: fn(&SemanticExpression) -> SemanticExpression,
) -> SemanticExpression {
    let value = semantic_with_ctx(expr, false);
    op(&value)
}

fn relation_expression(
    left: &Expression,
    relation: &Symbol,
    right: &Expression,
) -> SemanticExpression {
    let left = semantic_with_ctx(left, true);
    let right = semantic_with_ctx(right, true);
    match (left, right) {
        (SemanticExpression::Numeric(left), SemanticExpression::Numeric(right)) => {
            SemanticExpression::Logical(LogicalExpression::relation(&left, relation, &right))
        }
        _ => panic!("relation operator applied to non-numeric expressions"),
    }
}
