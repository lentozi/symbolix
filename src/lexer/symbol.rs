// 符号类型枚举

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Plus,        // +
    Minus,       // -
    Asterisk,    // *
    Slash,       // /
    Percent,     // %
    Caret,       // ^
    Equal,       // ==
    NotEqual,    // !=
    LessThan,    // <
    GreaterThan, // >
    LessEqual,   // <=
    GreaterEqual,// >=
    LeftParen,   // (
    RightParen,  // )
    Comma,       // ,
    Semicolon,   // ;
    Conditional, // ?
    ConditionalElse, // :
    LogicAdd,    // &&
    LogicOr,     // ||
    LogicNot,    // !
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Symbol::Plus => write!(f, "+"),
            Symbol::Minus => write!(f, "-"),
            Symbol::Asterisk => write!(f, "*"),
            Symbol::Slash => write!(f, "/"),
            Symbol::Percent => write!(f, "%"),
            Symbol::Caret => write!(f, "^"),
            Symbol::Equal => write!(f, "=="),
            Symbol::NotEqual => write!(f, "!="),
            Symbol::LessThan => write!(f, "<"),
            Symbol::GreaterThan => write!(f, ">"),
            Symbol::LessEqual => write!(f, "<="),
            Symbol::GreaterEqual => write!(f, ">="),
            Symbol::LeftParen => write!(f, "("),
            Symbol::RightParen => write!(f, ")"),
            Symbol::Comma => write!(f, ","),
            Symbol::Semicolon => write!(f, ";"),
            Symbol::Conditional => write!(f, "?"),
            Symbol::ConditionalElse => write!(f, ":"),
            Symbol::LogicAdd => write!(f, "&&"),
            Symbol::LogicOr => write!(f, "||"),
            Symbol::LogicNot => write!(f, "!"),
        }
    }
}

impl Symbol {
    // 自定义优先级（数值越大优先级越高）
    fn precedence(&self) -> u8 {
        match self {
            Symbol::LeftParen
            | Symbol::RightParen
            | Symbol::Comma
            | Symbol::Semicolon => 0,
            Symbol::Conditional
            | Symbol::ConditionalElse => 1,
            Symbol::LogicOr => 2,
            Symbol::LogicAdd => 3,
            Symbol::Equal
            | Symbol::NotEqual => 4,
            Symbol::LessThan
            | Symbol::GreaterThan
            | Symbol::LessEqual
            | Symbol::GreaterEqual => 5,
            Symbol::Plus
            | Symbol::Minus => 6,
            Symbol::Asterisk
            | Symbol::Slash => 7,
            Symbol::Caret => 8,
            Symbol::Percent
            | Symbol::LogicNot => 9,

        }
    }

    // 枚举序号，用作优先级相等时的稳定比较
    fn ordinal(&self) -> u8 {
        match self {
            Symbol::Plus => 0,
            Symbol::Minus => 1,
            Symbol::Asterisk => 2,
            Symbol::Slash => 3,
            Symbol::Percent => 4,
            Symbol::Caret => 5,
            Symbol::Equal => 6,
            Symbol::NotEqual => 7,
            Symbol::LessThan => 8,
            Symbol::GreaterThan => 9,
            Symbol::LessEqual => 10,
            Symbol::GreaterEqual => 11,
            Symbol::LeftParen => 12,
            Symbol::RightParen => 13,
            Symbol::Comma => 14,
            Symbol::Semicolon => 15,
            Symbol::Conditional => 16,
            Symbol::ConditionalElse => 17,
            Symbol::LogicAdd => 18,
            Symbol::LogicOr => 19,
            Symbol::LogicNot => 20
        }
    }
}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let p = self.precedence().cmp(&other.precedence());
        if p != std::cmp::Ordering::Equal {
            return p;
        }
        self.ordinal().cmp(&other.ordinal())
    }
}