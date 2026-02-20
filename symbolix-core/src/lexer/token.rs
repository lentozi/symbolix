use crate::lexer::constant::Constant;
use crate::lexer::symbol::Symbol;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Constant(Constant),
    Symbol(Symbol),
    Variable(String),
    Invalid(String),
}

impl Token {
    pub fn constant(constant: Constant) -> Token {
        Token::Constant(constant)
    }

    pub fn symbol(symbol: Symbol) -> Token {
        Token::Symbol(symbol)
    }

    pub fn variable(variable: String) -> Token {
        Token::Variable(variable)
    }

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
