use crate::lexer::constant::Constant;
use crate::lexer::symbol::Symbol;
use std::fmt;
use std::fmt::Display;

/// 词法分析器生成的词法单元
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// 常量
    Constant(Constant),
    /// 符号
    Symbol(Symbol),
    /// 变量
    Variable(String),
    /// 无效的词法单元
    Invalid(String),
}

impl Token {
    /// 创建常量词法单元
    pub fn constant(constant: Constant) -> Token {
        Token::Constant(constant)
    }

    /// 创建符号词法单元
    pub fn symbol(symbol: Symbol) -> Token {
        Token::Symbol(symbol)
    }

    /// 创建变量词法单元
    pub fn variable(variable: String) -> Token {
        Token::Variable(variable)
    }

    /// 创建无效的词法单元
    pub fn invalid(error: &str) -> Token {
        Token::Invalid(String::from(error))
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Constant(c) => write!(f, "Constant({})", c),
            Token::Symbol(s) => write!(f, "Symbol({})", s),
            Token::Variable(v) => write!(f, "Variable({})", v),
            Token::Invalid(i) => write!(f, "Invalid({})", i),
        }
    }
}
