use crate::lexer::constant::Constant;
use crate::lexer::symbol::Symbol;
use crate::lexer::variable::Variable;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Constant(Constant),
    Symbol(Symbol),
    Variable(Variable),
}

impl Token {

}