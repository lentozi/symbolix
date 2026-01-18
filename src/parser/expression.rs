use std::fmt;
use tree_drawer::tree::OwnedTree;
use crate::lexer::constant::Constant;
use crate::lexer::symbol::Symbol;

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // 常量表达式
    Constant(Constant),
    // 变量表达式
    Variable(String),
    // 一元表达式
    UnaryExpression(Symbol, Box<Expression>),
    // 二元表达式
    BinaryExpression(Box<Expression>, Symbol, Box<Expression>),
    // 三元表达式
    TernaryExpression(Box<Expression>, Symbol, Box<Expression>, Symbol, Box<Expression>),
    // 关系表达式
    Relation(Box<Expression>, Symbol, Box<Expression>),
}

impl Expression {
    pub fn constant(constant: Constant) -> Expression {
        Expression::Constant(constant)
    }

    pub fn variable(variable: String) -> Expression {
        Expression::Variable(variable)
    }

    pub fn unary(symbol: Symbol, expr: Expression) -> Expression {
        Expression::UnaryExpression(symbol, Box::new(expr))
    }

    pub fn binary(left: Expression, symbol: Symbol, right: Expression) -> Expression {
        Expression::BinaryExpression(Box::new(left), symbol, Box::new(right))
    }
    
    pub fn relation(left: Expression, symbol: Symbol, right: Expression) -> Expression {
        Expression::Relation(Box::new(left), symbol, Box::new(right))
    }

    pub fn ternary(cond: Expression, qmark: Symbol, then_expr: Expression, colon: Symbol, else_expr: Expression) -> Expression {
        Expression::TernaryExpression(Box::new(cond), qmark, Box::new(then_expr), colon, Box::new(else_expr))
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Constant(c) => write!(f, "{}", c),
            Expression::Variable(v) => write!(f, "{}", v),
            Expression::UnaryExpression(op, expr) => write!(f, "({} {})", op, expr),
            Expression::BinaryExpression(left, op, right) | 
            Expression::Relation(left, op, right) => write!(f, "({} {} {})", left, op, right),
            Expression::TernaryExpression(cond, qmark, then_expr, colon, else_expr) => {
                write!(f, "({} {} {} {} {})", cond, qmark, then_expr, colon, else_expr)
            }
        }
    }
}

impl Expression {
    pub fn to_owned_tree(&self) -> OwnedTree {
        match self {
            Expression::Constant(c) => {
                OwnedTree::new(format!("{c}"))
            }

            Expression::Variable(name) => {
                OwnedTree::new(name.clone())
            }

            Expression::UnaryExpression(op, expr) => {
                OwnedTree::new(format!("{op}"))
                    .with_child(expr.to_owned_tree())
            }

            Expression::BinaryExpression(lhs, op, rhs) => {
                OwnedTree::new(format!("{op}"))
                    .with_child(lhs.to_owned_tree())
                    .with_child(rhs.to_owned_tree())
            }

            Expression::TernaryExpression(cond, op1, then_expr, op2, else_expr) => {
                // ?: 或 if-then-else
                OwnedTree::new(format!("{op1}{op2}"))
                    .with_child(cond.to_owned_tree())
                    .with_child(then_expr.to_owned_tree())
                    .with_child(else_expr.to_owned_tree())
            }

            Expression::Relation(lhs, op, rhs) => {
                OwnedTree::new(format!("{op}"))
                    .with_child(lhs.to_owned_tree())
                    .with_child(rhs.to_owned_tree())
            }
        }
    }
}
