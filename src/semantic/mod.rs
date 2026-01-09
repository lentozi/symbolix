use crate::lexer::constant::{Constant, Number};
use crate::lexer::symbol::Symbol;
use crate::lexer::variable::Variable;
use crate::parser::expression::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticExpression {
    Numeric(NumericExpression),
    Logical(LogicalExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumericExpression {
    Constant(Number),
    Variable(Variable),
    Negation(Box<NumericExpression>),
    Addition(Vec<NumericExpression>),
    Multiplication(Vec<NumericExpression>), // a/b = a * b^(-1)
    Power {
        base: Box<NumericExpression>,
        exponent: Box<NumericExpression>, // 是否允许任意表达式？允许：超越函数；不允许：仅允许常数指数
    },
    Piecewise {
        cases: Vec<(LogicalExpression, NumericExpression)>,
        otherwise: Option<Box<NumericExpression>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalExpression {
    Constant(bool),
    Variable(Variable),
    Not(Box<LogicalExpression>),
    And(Vec<LogicalExpression>),
    Or(Vec<LogicalExpression>),
    Relation {
        left: Box<NumericExpression>,
        operator: Symbol,
        right: Box<NumericExpression>,
    },
}

fn push_left(stack: &mut Vec<Expression>, root: Expression) {
    let mut visiting = Some(root);

    loop {
        (*stack).push(visiting.clone().unwrap());
        if let Some(Expression::BinaryExpression(left, _, _)) = visiting {
            visiting = Some(*left);
            continue;
        }
        break;
    }
}

pub fn ast_to_semantic(expr: &Expression) -> SemanticExpression {
    let mut expr_stack: Vec<Expression> = Vec::new();
    let mut last_visited: Option<Expression> = None;
    let mut semantic_stack: Vec<SemanticExpression> = Vec::new();

    // 将所有子树的左子节点压入栈中
    push_left(&mut expr_stack, expr.clone());

    while expr_stack.len() > 0 {
        let mut current = expr_stack.last().cloned().unwrap();
        if let Some(last) = &last_visited {
            if let Expression::BinaryExpression(_, _, right) = &current {
                if **right == *last {           // 右子树已被访问，访问根节点
                    current = expr_stack.pop().unwrap();
                    match current {
                        Expression::BinaryExpression(_, symbol, _) => {
                            let right_semantic = semantic_stack.pop().unwrap();
                            let left_semantic = semantic_stack.pop().unwrap();
                            match symbol {
                                Symbol::Plus => {
                                    if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                        (left_semantic, right_semantic) {
                                        semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Addition(vec![l, r])));
                                    } else {
                                        panic!("'+' operator applied to non-numeric expressions");
                                    }
                                }
                                Symbol::Minus => {
                                    if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                        (left_semantic, right_semantic) {
                                        semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Addition(vec![
                                            l, NumericExpression::Negation(Box::new(r)),
                                        ])));
                                    } else {
                                        panic!("'-' operator applied to non-numeric expressions");
                                    }
                                }
                                Symbol::Asterisk => {
                                    if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                        (left_semantic, right_semantic) {
                                        semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Multiplication(vec![l, r])));
                                    } else {
                                        panic!("'*' operator applied to non-numeric expressions");
                                    }
                                }
                                Symbol::Slash => {
                                    if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                        (left_semantic, right_semantic) {
                                        semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Multiplication(vec![
                                            l, NumericExpression::Power {
                                                base: Box::new(r),
                                                exponent: Box::new(NumericExpression::Constant(Number::integer(-1))),
                                            },
                                        ])));
                                    } else {
                                        panic!("'/' operator applied to non-numeric expressions");
                                    }
                                }
                                _ => panic!("unsupported binary operator: {}", symbol),
                            }
                        }
                        Expression::Variable(ref v) => {
                            semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Variable((*v).clone())));
                        }
                        _ => panic!("expected binary expression"),
                    }
                    last_visited = Some(current);
                } else {        // 右子树没有被访问，访问右子树
                    push_left(&mut expr_stack, (**right).clone())
                }
            } else {
                current = expr_stack.pop().unwrap();

                match current {
                    Expression::BinaryExpression(_, symbol, _) => {
                        let right_semantic = semantic_stack.pop().unwrap();
                        let left_semantic = semantic_stack.pop().unwrap();
                        match symbol {
                            Symbol::Plus => {
                                if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                    (left_semantic, right_semantic) {
                                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Addition(vec![l, r])));
                                } else {
                                    panic!("'+' operator applied to non-numeric expressions");
                                }
                            }
                            Symbol::Minus => {
                                if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                    (left_semantic, right_semantic) {
                                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Addition(vec![
                                        l, NumericExpression::Negation(Box::new(r)),
                                    ])));
                                } else {
                                    panic!("'-' operator applied to non-numeric expressions");
                                }
                            }
                            Symbol::Asterisk => {
                                if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                    (left_semantic, right_semantic) {
                                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Multiplication(vec![l, r])));
                                } else {
                                    panic!("'*' operator applied to non-numeric expressions");
                                }
                            }
                            Symbol::Slash => {
                                if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                    (left_semantic, right_semantic) {
                                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Multiplication(vec![
                                        l, NumericExpression::Power {
                                            base: Box::new(r),
                                            exponent: Box::new(NumericExpression::Constant(Number::integer(-1))),
                                        },
                                    ])));
                                } else {
                                    panic!("'/' operator applied to non-numeric expressions");
                                }
                            }
                            _ => panic!("unsupported binary operator: {}", symbol),
                        }
                    }
                    Expression::Variable(ref v) => {
                        semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Variable((*v).clone())));
                    }
                    _ => panic!("expected binary expression"),
                }
                last_visited = Some(current);
            }
        } else {
            current = expr_stack.pop().unwrap();

            match current {
                Expression::BinaryExpression(_, symbol, _) => {
                    let right_semantic = semantic_stack.pop().unwrap();
                    let left_semantic = semantic_stack.pop().unwrap();
                    match symbol {
                        Symbol::Plus => {
                            if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                (left_semantic, right_semantic) {
                                semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Addition(vec![l, r])));
                            } else {
                                panic!("'+' operator applied to non-numeric expressions");
                            }
                        }
                        Symbol::Minus => {
                            if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                (left_semantic, right_semantic) {
                                semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Addition(vec![
                                    l, NumericExpression::Negation(Box::new(r)),
                                ])));
                            } else {
                                panic!("'-' operator applied to non-numeric expressions");
                            }
                        }
                        Symbol::Asterisk => {
                            if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                (left_semantic, right_semantic) {
                                semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Multiplication(vec![l, r])));
                            } else {
                                panic!("'*' operator applied to non-numeric expressions");
                            }
                        }
                        Symbol::Slash => {
                            if let (SemanticExpression::Numeric(l), SemanticExpression::Numeric(r)) =
                                (left_semantic, right_semantic) {
                                semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Multiplication(vec![
                                    l, NumericExpression::Power {
                                        base: Box::new(r),
                                        exponent: Box::new(NumericExpression::Constant(Number::integer(-1))),
                                    },
                                ])));
                            } else {
                                panic!("'/' operator applied to non-numeric expressions");
                            }
                        }
                        _ => panic!("unsupported binary operator: {}", symbol),
                    }
                }
                Expression::Variable(ref v) => {
                    semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Variable((*v).clone())));
                }
                _ => panic!("expected binary expression"),
            }
            last_visited = Some(current);
        }
    }


    assert_eq!(semantic_stack.len(), 1);
    semantic_stack.pop().unwrap()
}

pub fn push_left_override<'a>(stack: &mut Vec<&'a Expression>, root: &'a Expression) {
    let mut visiting = Some(root);

    loop {
        stack.push(visiting.unwrap());
        if let Some(Expression::BinaryExpression(left, _, _)) = visiting {
            visiting = Some(left);
            continue;
        }
        break;
    }
}

fn is_numeric(expr: &Expression) -> bool {
    match expr {
        Expression::Constant(Constant::Number(_)) | Expression::Variable(_) => true,
        Expression::UnaryExpression(Symbol::Minus, _) => true,
        Expression::UnaryExpression(Symbol::LogicNot, _) => false,
        Expression::BinaryExpression(_, Symbol::Plus, _) => true,
        Expression::TernaryExpression(_, _, _, _, _) => false,
        Expression::Relation(_, _, _) => false,
        Expression::Constant(Constant::Boolean(_)) => false,
        _ => panic!("unsupported expression type"),
    };
    todo!()
}


pub fn ast_to_semantic_override(expr: &Expression) -> SemanticExpression {
    let mut expr_stack: Vec<Expression> = Vec::new();
    let mut semantic_stack: Vec<SemanticExpression> = Vec::new();
    let mut last_visited: Option<Expression> = None;

    let numeric = is_numeric(expr);

    // 将左子树压入栈中
    push_left(&mut expr_stack, expr.clone());

    // while expr_stack.len() > 0 {
    //     if last_visited.is_none() {     // 之前还没有开始访问，最先访问的一定是叶节点
    //         let current = expr_stack.pop().unwrap();
    //         match current {
    //             Expression::Variable(v) => {
    //                 semantic_stack.push(SemanticExpression::Numeric(NumericExpression::Variable(v)));
    //             }
    //             Expression::Constant(c) => {
    //                 semantic_stack.push()
    //             }
    //         }
    //     }
    // }

    semantic_stack.pop().unwrap()
}