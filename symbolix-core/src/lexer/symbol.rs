// 符号类型枚举

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Unary(Unary),
    Binary(Binary),
    Ternary(Ternary),
    Relation(Relation),
    Other(Other),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unary {
    Plus,
    Minus,
    LogicNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Binary {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Power,
    LogicAnd,
    LogicOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relation {
    Equal,          // ==
    NotEqual,       // !=
    LessThan,       // <
    GreaterThan,    // >
    LessEqual,      // <=
    GreaterEqual,   // >=
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Conditional,      // ?
    ConditionalElse,  // :
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Other {
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Precedence {
    Lowest = 0,
    TERNARY,       //  :
    Conditional,   // ?
    LogicOr,       // ||
    LogicAnd,      // &&
    Equality,      // == !=
    Relational,    // < > <= >=
    Additive,      // + -
    Multiplicative,// * / %
    Power,         // ^
    Unary,         // ! -
}

pub fn get_precedence(symbol: &Symbol) -> Precedence {
    match symbol {
        Symbol::Ternary(Ternary::ConditionalElse) => Precedence::TERNARY,
        Symbol::Ternary(Ternary::Conditional) => Precedence::Conditional,
        Symbol::Binary(Binary::LogicOr) => Precedence::LogicOr,
        Symbol::Binary(Binary::LogicAnd) => Precedence::LogicAnd,
        Symbol::Relation(Relation::Equal | Relation::NotEqual) => Precedence::Equality,
        Symbol::Relation(Relation::LessThan | Relation::GreaterThan | Relation::LessEqual | Relation::GreaterEqual) => Precedence::Relational,
        Symbol::Binary(Binary::Add | Binary::Subtract) => Precedence::Additive,
        Symbol::Binary(Binary::Multiply | Binary::Divide | Binary::Modulus) => Precedence::Multiplicative,
        Symbol::Binary(Binary::Power) => Precedence::Power,
        Symbol::Unary(_) => Precedence::Unary,
        _ => Precedence::Lowest,
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SymbolType {
    Arithmetic,
    Relational,
    Logical,
    Conditional,
    Other,
}

pub fn get_symbol_type(symbol: &Symbol) -> SymbolType {
    match symbol {
        Symbol::Binary(Binary::Add | Binary::Subtract | Binary::Multiply | Binary::Divide | Binary::Modulus | Binary::Power) |
        Symbol::Unary(Unary::Plus | Unary::Minus) => SymbolType::Arithmetic,
        Symbol::Relation(_) => SymbolType::Relational,
        Symbol::Binary(Binary::LogicAnd | Binary::LogicOr) |
        Symbol::Unary(Unary::LogicNot) => SymbolType::Logical,
        Symbol::Ternary(_) => SymbolType::Conditional,
        _ =>  SymbolType::Other,
    }
}


impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Symbol::Binary(Binary::Add) | Symbol::Unary(Unary::Plus) => write!(f, "+"),
            Symbol::Binary(Binary::Subtract) | Symbol::Unary(Unary::Minus) => write!(f, "-"),
            Symbol::Binary(Binary::Multiply) => write!(f, "*"),
            Symbol::Binary(Binary::Divide) => write!(f, "/"),
            Symbol::Binary(Binary::Modulus) => write!(f, "%"),
            Symbol::Binary(Binary::Power) => write!(f, "^"),
            Symbol::Relation(Relation::Equal) => write!(f, "=="),
            Symbol::Relation(Relation::NotEqual) => write!(f, "!="),
            Symbol::Relation(Relation::LessThan) => write!(f, "<"),
            Symbol::Relation(Relation::GreaterThan) => write!(f, ">"),
            Symbol::Relation(Relation::LessEqual) => write!(f, "<="),
            Symbol::Relation(Relation::GreaterEqual) => write!(f, ">="),
            Symbol::Other(Other::LeftParen) => write!(f, "("),
            Symbol::Other(Other::RightParen) => write!(f, ")"),
            Symbol::Other(Other::Comma) => write!(f, ","),
            Symbol::Other(Other::Semicolon) => write!(f, ";"),
            Symbol::Ternary(Ternary::Conditional) => write!(f, "?"),
            Symbol::Ternary(Ternary::ConditionalElse) => write!(f, ":"),
            Symbol::Binary(Binary::LogicAnd) => write!(f, "&&"),
            Symbol::Binary(Binary::LogicOr) => write!(f, "||"),
            Symbol::Unary(Unary::LogicNot) => write!(f, "!"),
        }
    }
}
