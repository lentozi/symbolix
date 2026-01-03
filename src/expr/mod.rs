use std::fmt;
use crate::lexer::token::Token;
use crate::lexer::Lexer;
use crate::lexer::constant::{Constant, Number};
use crate::lexer::symbol::{get_precedence, Precedence, Symbol};
use crate::lexer::variable::Variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // 常量表达式
    Constant(Constant),
    // 变量表达式
    Variable(Variable),
    // 一元表达式
    UnaryExpression(Symbol, Box<Expression>),
    // 二元表达式
    BinaryExpression(Box<Expression>, Symbol, Box<Expression>),
    // 三元表达式
    TernaryExpression(Box<Expression>, Symbol, Box<Expression>, Symbol, Box<Expression>),
}

impl Expression {
    pub fn constant(constant: Constant) -> Expression {
        Expression::Constant(constant)
    }

    pub fn variable(variable: Variable) -> Expression {
        Expression::Variable(variable)
    }

    pub fn unary(symbol: Symbol, expr: Expression) -> Expression {
        Expression::UnaryExpression(symbol, Box::new(expr))
    }

    pub fn binary(left: Expression, symbol: Symbol, right: Expression) -> Expression {
        Expression::BinaryExpression(Box::new(left), symbol, Box::new(right))
    }

    pub fn ternary(cond: Expression, qmark: Symbol, then_expr: Expression, colon: Symbol, else_expr: Expression) -> Expression {
        Expression::TernaryExpression(Box::new(cond), qmark, Box::new(then_expr), colon, Box::new(else_expr))
    }

    pub fn fix_ternary(self, else_expr: Expression) -> Expression {
        if let Expression::TernaryExpression(cond, qmark, then_expr, colon, _) = self {
            Expression::TernaryExpression(cond, qmark, then_expr, colon, Box::new(else_expr))
        } else {
            panic!("fix_ternary called on non-ternary expression");
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Constant(c) => write!(f, "{}", c),
            Expression::Variable(v) => write!(f, "{}", v),
            Expression::UnaryExpression(op, expr) => write!(f, "({} {})", op, expr),
            Expression::BinaryExpression(left, op, right) => write!(f, "({} {} {})", left, op, right),
            Expression::TernaryExpression(cond, qmark, then_expr, colon, else_expr) => {
                write!(f, "({} {} {} {} {})", cond, qmark, then_expr, colon, else_expr)
            }
        }
    }
}

pub fn syntax_analyzer(mut lexer: Lexer) -> Expression {
    while let Some(token) = lexer.next_token() {
        print!("{}  ", token);
    }
    Expression::constant(Constant::Number(Number::integer(1)))
}

pub fn parse_expression(lexer: &mut Lexer, min_precedence: Precedence) -> Expression {
    let mut left_expr = match lexer.next_token().expect("expect a token") {
        Token::Constant(c) => Expression::constant(c),
        Token::Variable(v) => Expression::variable(v),
        Token::Symbol(Symbol::LeftParen) => {
            let expr = parse_expression(lexer, min_precedence);
            if let Some(Token::Symbol(Symbol::RightParen)) = lexer.next_token() {
                expr
            } else {
                panic!("Expected closing parenthesis");
            }
        },
        Token::Symbol(Symbol::Minus) | Token::Symbol(Symbol::LogicNot) => {
            let symbol = match lexer.next_token().expect("expect a token") {
                Token::Symbol(s) => s,
                _ => panic!("Expected a symbol after unary operator"),
            };
            let expr = parse_expression(lexer, min_precedence);
            Expression::unary(symbol, expr)
        },
        _ => panic!("Unexpected token"),
    };

    // 循环处理后缀操作符、二元操作符、三元操作符
    loop {
        let (left_precedence, right_precedence, _next_expr) = match lexer.peek_token().expect("expect a token") {
            Token::Symbol(Symbol::Plus) | Token::Symbol(Symbol::Minus) => (Precedence::Additive, Precedence::Additive, true),
            Token::Symbol(Symbol::Asterisk) | Token::Symbol(Symbol::Slash)
            | Token::Symbol(Symbol::Percent) => (Precedence::Multiplicative, Precedence::Multiplicative, true),
            Token::Symbol(Symbol::Caret) => (Precedence::Power, Precedence::Power, true),
            Token::Symbol(Symbol::Equal) | Token::Symbol(Symbol::NotEqual) => (Precedence::Equality, Precedence::Equality, true),
            Token::Symbol(Symbol::LessThan) | Token::Symbol(Symbol::GreaterThan)
            | Token::Symbol(Symbol::LessEqual) | Token::Symbol(Symbol::GreaterEqual) => (Precedence::Relational, Precedence::Relational, true),
            Token::Symbol(Symbol::LogicAdd) => (Precedence::LogicAnd, Precedence::LogicAnd, true),
            Token::Symbol(Symbol::LogicOr) => (Precedence::LogicOr, Precedence::LogicOr, true),
            Token::Symbol(Symbol::Conditional) => (Precedence::Conditional, Precedence::Conditional, true),
            _ => break,
        };

        if left_precedence < min_precedence {
            break;
        }

        // 保存操作符
        let op = match lexer.next_token().expect("expect a token") {
            Token::Symbol(s) => s,
            _ => panic!("Expected a symbol"),
        };

        // 对三元运算符的特殊处理
        let right_expr = if op == Symbol::Conditional {
            let then_expr = parse_expression(lexer, Precedence::Conditional);
            if let Some(Token::Symbol(Symbol::ConditionalElse)) = lexer.next_token() {
                let else_expr = parse_expression(lexer, Precedence::Conditional);
                Expression::ternary(left_expr.clone(), op, then_expr, Symbol::ConditionalElse, else_expr)
            } else {
                panic!("Expected ':' in ternary expression");
            }
        } else {
            let rhs = parse_expression(lexer, right_precedence);
            Expression::binary(left_expr.clone(), op, rhs)
        };

        left_expr = right_expr;
    }
    left_expr
}

pub fn parse_expression1(lexer: &mut Lexer, _min_precedence: Precedence) -> Expression {
    let mut operand_stack: Vec<Expression> = Vec::new();
    let mut op_stack: Vec<(Symbol, Precedence)> = Vec::new();

    loop {
        let left = match lexer.next_token() {
            Some(Token::Constant(c)) => Expression::constant(c),
            Some(Token::Variable(v)) => Expression::variable(v),
            Some(Token::Symbol(Symbol::LeftParen)) => {
                let expr = parse_expression1(lexer, Precedence::Lowest);
                if let Some(Token::Symbol(Symbol::RightParen)) = lexer.next_token() {
                    expr
                } else {
                    panic!("Expected closing parenthesis");
                }
            },
            Some(Token::Symbol(s)) if s == Symbol::Minus || s == Symbol::LogicNot => {
                let expr = parse_expression1(lexer, Precedence::Unary);
                Expression::unary(s, expr)
            },
            _ => break,
        };

        operand_stack.push(left);

        loop {
            let op = match lexer.peek_token() {
                Some(Token::Symbol(s)) => s,
                _ => break,
            };

            let prec = get_precedence(&op);

            while let Some(&(_, top_prec)) = op_stack.last() {
                if top_prec >= prec {
                    let right = operand_stack.pop().unwrap();
                    let left = operand_stack.pop().unwrap();
                    let top_op = op_stack.pop().unwrap().0;
                    if top_op == Symbol::Conditional {
                        let else_expr = right;
                        let then_expr = left;
                        let cond_expr = operand_stack.pop().unwrap();
                        let ternary_expr = Expression::ternary(cond_expr, top_op, then_expr, Symbol::ConditionalElse, else_expr);
                        operand_stack.push(ternary_expr);
                    } else {
                        let binary_expr = Expression::binary(left, top_op, right);
                        operand_stack.push(binary_expr);
                    }
                } else { break; }
            }

            if op == Symbol::Conditional {
                op_stack.push((op, prec));
                lexer.next_token(); // consume '?'
                continue;
            } else {
                op_stack.push((op, prec));
                lexer.next_token(); // consume operator
            }

            // 解析右操作数
            let right = parse_expression1(lexer, prec);
            operand_stack.push(right);
        }

        if operand_stack.len() == 1 {
            break;
        }
    }

    while let Some((op, _)) = op_stack.pop() {
        let right = operand_stack.pop().unwrap();
        let left = operand_stack.pop().unwrap();
        if op == Symbol::Conditional {
            let else_expr = right;
            let then_expr = left;
            let cond_expr = operand_stack.pop().unwrap();
            let ternary_expr = Expression::ternary(cond_expr, op, then_expr, Symbol::ConditionalElse, else_expr);
            operand_stack.push(ternary_expr);
        } else {
            let binary_expr = Expression::binary(left, op, right);
            operand_stack.push(binary_expr);
        }
    }

    operand_stack.pop().unwrap()
}

pub fn parse_expression2(lexer: &mut Lexer) -> Expression {
    // Implementation of another parsing strategy can be added here
    let mut expr_stack: Vec<Expression> = Vec::new();
    let mut op_stack: Vec<Symbol> = Vec::new();

    while let Some(token) = lexer.next_token() {
        match token {
            Token::Constant(c) => expr_stack.push(Expression::constant(c)),
            Token::Variable(v) => expr_stack.push(Expression::variable(v)),
            Token::Symbol(s) => {
                // Handle operators and parentheses here
                match s {
                    Symbol::LeftParen => op_stack.push(Symbol::LeftParen),
                    Symbol::RightParen => {
                        while let Some(op) = op_stack.pop() {
                            match op {
                                Symbol::LeftParen => break,
                                Symbol::Conditional => unreachable!(),
                                _ => {
                                    let right = expr_stack.pop().unwrap();
                                    let left = expr_stack.pop().unwrap();
                                    let binary_expr = Expression::binary(left, op, right);
                                    expr_stack.push(binary_expr);
                                }
                            }
                        }
                    },
                    Symbol::Conditional => {
                        while let Some(op) = op_stack.last() {
                            match op {
                                Symbol::Plus
                                | Symbol::Minus
                                | Symbol::Asterisk
                                | Symbol::Slash
                                | Symbol::Equal
                                | Symbol::NotEqual
                                | Symbol::LessThan
                                | Symbol::GreaterThan
                                | Symbol::LessEqual
                                | Symbol::GreaterEqual
                                | Symbol::LogicAdd
                                | Symbol::LogicOr
                                | Symbol::Caret if *op >= s => {
                                    let op = op_stack.pop().unwrap();
                                    let right = expr_stack.pop().unwrap();
                                    let left = expr_stack.pop().unwrap();
                                    let binary_expr = Expression::binary(left, op, right);
                                    expr_stack.push(binary_expr);
                                },
                                // TODO 如果遇到左括号呢，如果三元表达式嵌套呢
                                _ => break,
                            }
                        }
                        op_stack.push(Symbol::Conditional)
                    },
                    Symbol::ConditionalElse => {
                        while let Some(op) = op_stack.pop() {
                            match op {
                                Symbol::Conditional => {
                                    let then_expr = expr_stack.pop().unwrap();
                                    let cond_expr = expr_stack.pop().unwrap();

                                    // 把 (cond, then) 暂存为一个未完成的三元
                                    expr_stack.push(Expression::ternary(
                                        cond_expr,
                                        Symbol::Conditional,
                                        then_expr,
                                        Symbol::ConditionalElse,
                                        Expression::constant(Constant::Number(Number::integer(0)))
                                    ));
                                    break;
                                },
                                _ => {
                                    let right = expr_stack.pop().unwrap();
                                    let left = expr_stack.pop().unwrap();
                                    let binary_expr = Expression::binary(left, op, right);
                                    expr_stack.push(binary_expr);
                                }
                            }
                        }
                    },
                    _ => {
                        // 普通二元运算符
                        while let Some(op) = op_stack.last() {
                            match op {
                                Symbol::Plus
                                | Symbol::Minus
                                | Symbol::Asterisk
                                | Symbol::Slash
                                | Symbol::Equal
                                | Symbol::NotEqual
                                | Symbol::LessThan
                                | Symbol::GreaterThan
                                | Symbol::LessEqual
                                | Symbol::GreaterEqual
                                | Symbol::LogicAdd
                                | Symbol::LogicOr
                                | Symbol::Caret if *op >= s => {
                                    let op = op_stack.pop().unwrap();
                                    let right = expr_stack.pop().unwrap();
                                    let left = expr_stack.pop().unwrap();
                                    let binary_expr = Expression::binary(left, op, right);
                                    expr_stack.push(binary_expr);
                                },
                                _ => break,
                            }
                        }
                        op_stack.push(s);
                    }
                }
            },
        }
    }

    while let Some(op) = op_stack.pop() {
        let right = expr_stack.pop().unwrap();
        let left = expr_stack.pop().unwrap();
        let binary_expr = Expression::binary(left, op, right);
        expr_stack.push(binary_expr);
    }

    // 修补三元式
    let last = expr_stack.pop().unwrap();
    let to_fix = expr_stack.pop().unwrap();
    expr_stack.push(Expression::fix_ternary(to_fix, last));

    expr_stack.pop().unwrap()
}

pub fn pratt_parsing(lexer: &mut Lexer, min_precedence: Precedence) -> Expression {
    // nud
    let mut left_expr = match lexer.next_token() {
        Some(Token::Constant(c)) => Expression::constant(c),
        Some(Token::Variable(v)) => Expression::variable(v),
        Some(Token::Symbol(s@ (Symbol::Minus | Symbol::LogicNot))) => {
            // s is bound to the actual symbol matched (Minus or LogicNot)
            let expr = pratt_parsing(lexer, Precedence::Unary);
            Expression::unary(s, expr)
        },
        Some(Token::Symbol(Symbol::LeftParen)) => {
            let expr = pratt_parsing(lexer, Precedence::Lowest);
            if let Some(Token::Symbol(Symbol::RightParen)) = lexer.next_token() {
                expr
            } else {
                panic!("expected closing parenthesis");
            }
        },
        _ => panic!("unexpected token"),
    };

    // led
    loop {
        let operation = match lexer.peek_token() {
            Some(Token::Symbol(Symbol::RightParen)) => break,
            Some(Token::Symbol(s)) => s,
            None => break,
            _ => panic!("unexpected token, expected operator"),
        };

        if get_precedence(&operation) < min_precedence {
            break;
        }

        if operation == Symbol::Conditional {
            lexer.next_token(); // consume '?'
            let then_expr = pratt_parsing(lexer, Precedence::Conditional);
            if let Some(Token::Symbol(Symbol::ConditionalElse)) = lexer.next_token() {
                let else_expr = pratt_parsing(lexer, Precedence::Conditional);
                left_expr = Expression::ternary(left_expr, Symbol::Conditional, then_expr, Symbol::ConditionalElse, else_expr);
                continue;
            } else {
                panic!("expected ':' in ternary expression, found {:?}", lexer.peek_token());
            }
        }

        lexer.next_token(); // consume operator
        let right = pratt_parsing(lexer, get_precedence(&operation));
        left_expr = Expression::binary(left_expr, operation, right);
    }

    left_expr
}