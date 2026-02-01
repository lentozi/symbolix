pub mod bucket;
pub mod context;
mod macros;
pub mod semantic_ir;
pub mod variable;

use crate::lexer::constant::Constant;
use crate::lexer::symbol::Symbol;
use crate::lexer::symbol::{Binary, Ternary, Unary};
use crate::parser::expression::Expression;
use crate::semantic::semantic_ir::logic::LogicalExpression;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::semantic_ir::SemanticExpression;
use crate::semantic::variable::VariableType;
use crate::with_context;

fn push_left(stack: &mut Vec<Expression>, root: Expression) {
    let mut visiting = Some(root);

    loop {
        (*stack).push(visiting.clone().unwrap());
        if let Some(Expression::BinaryExpression(left, _, _) | Expression::Relation(left, _, _)) =
            visiting
        {
            visiting = Some(*left);
            continue;
        }
        break;
    }
}

pub fn ast_to_semantic(expr: &Expression) -> SemanticExpression {
    let mut expr_stack: Vec<Expression> = Vec::new();
    let mut semantic_stack: Vec<SemanticExpression> = Vec::new();
    let mut last_visited: Option<Expression> = None;

    // 将左子树压入栈中
    push_left(&mut expr_stack, expr.clone());

    while expr_stack.len() > 0 {
        if last_visited.is_none() {
            // 之前还没有开始访问，最先访问的一定是叶节点
            let current = expr_stack.pop().unwrap();
            visit_leaf_node(&mut semantic_stack, current.clone());
            last_visited = Some(current);
        } else {
            let expression = match expr_stack.last() {
                Some(Expression::BinaryExpression(_, operation, right)) => {
                    Some((*operation, (**right).clone()))
                }
                Some(Expression::Relation(_, operation, right)) => {
                    Some((*operation, (**right).clone()))
                }
                _ => None,
            };
            if expression.is_some() {
                // 当前节点是二元表达式，取出操作符和右子树
                let (operation, right) = expression.unwrap();
                if right != *last_visited.as_ref().unwrap() {
                    // 右子树还没有被访问，继续访问右子树
                    push_left(&mut expr_stack, right);
                } else {
                    // 左右子树均已访问，访问根节点
                    let current = expr_stack.pop().unwrap();
                    let right_semantic = semantic_stack.pop().unwrap();
                    let left_semantic = semantic_stack.pop().unwrap();

                    match operation {
                        Symbol::Binary(Binary::Add) => match (left_semantic, right_semantic) {
                            (
                                SemanticExpression::Numeric(left),
                                SemanticExpression::Numeric(right),
                            ) => semantic_stack.push(SemanticExpression::Numeric(
                                NumericExpression::addition(left, right),
                            )),
                            _ => panic!("'+' operator applied to non-numeric expressions"),
                        },
                        Symbol::Binary(Binary::Subtract) => match (left_semantic, right_semantic) {
                            (
                                SemanticExpression::Numeric(left),
                                SemanticExpression::Numeric(right),
                            ) => semantic_stack.push(SemanticExpression::Numeric(
                                NumericExpression::subtraction(left, right),
                            )),
                            _ => panic!(
                                "'-' operator applied to mismatched or logical expression types"
                            ),
                        },
                        Symbol::Binary(Binary::Multiply) => match (left_semantic, right_semantic) {
                            (
                                SemanticExpression::Numeric(left),
                                SemanticExpression::Numeric(right),
                            ) => semantic_stack.push(SemanticExpression::Numeric(
                                NumericExpression::multiplication(left, right),
                            )),
                            _ => panic!("'*' operator applied to non-numeric expressions"),
                        },
                        Symbol::Binary(Binary::Divide) => match (left_semantic, right_semantic) {
                            (
                                SemanticExpression::Numeric(left),
                                SemanticExpression::Numeric(right),
                            ) => semantic_stack.push(SemanticExpression::Numeric(
                                NumericExpression::division(left, right),
                            )),
                            _ => panic!("'/' operator applied to non-numeric expressions"),
                        },
                        Symbol::Binary(Binary::Power) => match (left_semantic, right_semantic) {
                            (
                                SemanticExpression::Numeric(base),
                                SemanticExpression::Numeric(exponent),
                            ) => semantic_stack.push(SemanticExpression::Numeric(
                                NumericExpression::power(base, exponent),
                            )),
                            _ => panic!("'^' operator applied to non-numeric expressions"),
                        },
                        Symbol::Binary(Binary::LogicAnd) => match (left_semantic, right_semantic) {
                            (
                                SemanticExpression::Logical(left),
                                SemanticExpression::Logical(right),
                            ) => semantic_stack.push(SemanticExpression::Logical(
                                LogicalExpression::and(left, right),
                            )),
                            _ => panic!("'&&' operator applied to non-logical expressions"),
                        },
                        Symbol::Binary(Binary::LogicOr) => match (left_semantic, right_semantic) {
                            (
                                SemanticExpression::Logical(left),
                                SemanticExpression::Logical(right),
                            ) => semantic_stack.push(SemanticExpression::Logical(
                                LogicalExpression::or(left, right),
                            )),
                            _ => panic!("'||' operator applied to non-logical expressions"),
                        },
                        Symbol::Relation(_) => match (left_semantic, right_semantic) {
                            (
                                SemanticExpression::Numeric(left),
                                SemanticExpression::Numeric(right),
                            ) => semantic_stack.push(SemanticExpression::Logical(
                                LogicalExpression::relation(left, operation, right),
                            )),
                            _ => panic!("relation operator applied to non-numeric expressions"),
                        },
                        _ => panic!("expected binary operator, found {}", operation),
                    }
                    last_visited = Some(current);
                }
            } else {
                // 当前节点不是二元表达式，直接访问根节点
                let current = expr_stack.pop().unwrap();
                visit_leaf_node(&mut semantic_stack, current.clone());
                last_visited = Some(current);
            }
        }
    }

    assert_eq!(semantic_stack.len(), 1);
    semantic_stack.pop().unwrap()
}

pub fn visit_leaf_node(stack: &mut Vec<SemanticExpression>, node: Expression) {
    match node {
        Expression::Variable(ref v) => {
            // 变量必须显式声明类型

            let var = with_context!(ctx, {
                match ctx.symbols.borrow_mut().find(v.as_str()) {
                    Some(var) => var,
                    None => panic!("variable '{}' not declared", v),
                }
            });
            match var.var_type {
                VariableType::Boolean => {
                    stack.push(SemanticExpression::Logical(LogicalExpression::variable(
                        var,
                    )));
                }
                VariableType::Integer | VariableType::Float | VariableType::Fraction => {
                    stack.push(SemanticExpression::Numeric(NumericExpression::variable(
                        var,
                    )));
                }
                VariableType::Unknown => unreachable!(),
            }
        }
        Expression::Constant(Constant::Number(ref n)) => {
            let n = (*n).clone();
            stack.push(SemanticExpression::Numeric(NumericExpression::constant(n)))
        }
        Expression::Constant(Constant::Boolean(ref b)) => {
            let b = (*b).clone();
            stack.push(SemanticExpression::Logical(LogicalExpression::constant(b)))
        }
        Expression::TernaryExpression(cond, symbol1, then, symbol2, otherwise) => {
            if symbol1 == Symbol::Ternary(Ternary::Conditional)
                && symbol2 == Symbol::Ternary(Ternary::ConditionalElse)
            {
                let otherwise_semantic = ast_to_semantic(otherwise.as_ref());
                let then_semantic = ast_to_semantic(then.as_ref());
                let cond_semantic = ast_to_semantic(cond.as_ref());

                match (cond_semantic, then_semantic, otherwise_semantic) {
                    (
                        SemanticExpression::Logical(c),
                        SemanticExpression::Numeric(t),
                        SemanticExpression::Numeric(o),
                    ) => stack.push(SemanticExpression::Numeric(NumericExpression::piecewise(
                        vec![(c, t)],
                        Some(o),
                    ))),
                    _ => panic!("ternary expression with mismatched semantic types"),
                }
            } else {
                panic!(
                    "unsupported symbols in ternary expression: {}, {}",
                    symbol1, symbol2
                );
            }
        }
        Expression::UnaryExpression(symbol, expr) => {
            let expr_semantic = ast_to_semantic(&expr);
            match symbol {
                Symbol::Unary(Unary::Plus) => match expr_semantic {
                    SemanticExpression::Numeric(n) => stack.push(SemanticExpression::Numeric(n)),
                    _ => panic!("'+' operator applied to non-numeric expression"),
                },
                Symbol::Unary(Unary::Minus) => match expr_semantic {
                    SemanticExpression::Numeric(n) => {
                        stack.push(SemanticExpression::Numeric(NumericExpression::negation(n)))
                    }
                    _ => panic!("'-' operator applied to non-numeric expression"),
                },
                Symbol::Unary(Unary::LogicNot) => {
                    println!("{:?}", expr_semantic);
                    match expr_semantic {
                        SemanticExpression::Logical(l) => {
                            stack.push(SemanticExpression::Logical(LogicalExpression::not(l)))
                        }
                        _ => panic!("'!' operator applied to non-logical expression"),
                    }
                }
                _ => panic!("unexpected unary operator: {}", symbol),
            }
        }
        _ => panic!("expected variable or constant expression"),
    }
}
