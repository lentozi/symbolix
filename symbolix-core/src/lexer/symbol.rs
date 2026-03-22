// 符号类型枚举

use std::fmt;

/// 符号枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    /// 一元运算符
    Unary(Unary),
    /// 二元运算符
    Binary(Binary),
    /// 三元运算符
    Ternary(Ternary),
    /// 关系运算符
    Relation(Relation),
    /// 其他符号
    Other(Other),
}

/// 一元运算符枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Unary {
    /// 正号
    Plus,
    /// 负号
    Minus,
    /// 逻辑非
    LogicNot,
}

/// 二元运算符枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Binary {
    /// 加号
    Add,
    /// 减号
    Subtract,
    /// 乘号
    Multiply,
    /// 除号
    Divide,
    /// 取模
    Modulus,
    /// 幂运算
    Power,
    /// 逻辑与
    LogicAnd,
    /// 逻辑或
    LogicOr,
}

/// 关系运算符枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Relation {
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
    /// 小于
    LessThan,
    /// 大于
    GreaterThan,
    /// 小于等于
    LessEqual,
    /// 大于等于
    GreaterEqual,
}

/// 三元运算符枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    /// 符号“ ? ”
    Conditional,
    /// 符号“ : ”
    ConditionalElse,
}

/// 其他符号枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Other {
    /// 左括号
    LeftParen,
    /// 右括号
    RightParen,
    /// 逗号
    Comma,
    /// 分号
    Semicolon,
}

/// 符号类型枚举
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum SymbolType {
    /// 算数类型符号
    Arithmetic,
    /// 关系类型符号
    Relational,
    /// 逻辑类型符号
    Logical,
    /// 三元运算符类型符号
    Conditional,
    /// 其他类型符号
    Other,
}

/// 符号优先级枚举
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Precedence {
    Lowest = 0,
    /// 符号“ : ”
    TERNARY,
    /// 符号“ ? ”
    Conditional,
    /// 符号“ || ”
    LogicOr,
    /// 符号“ && ”
    LogicAnd,
    /// 符号“ == ”和“ != ”
    Equality,
    /// 符号“ < ”和“ > ”
    Relational,
    /// 符号“ + ”和“ - ”
    Additive,
    /// 符号“ * ”和“ / ”
    Multiplicative,
    /// 符号“ ^ ”
    Power,
    /// 符号“ ! ”和“ - ”
    Unary,
}

/// 根据符号获取优先级
pub fn get_precedence(symbol: &Symbol) -> Precedence {
    match symbol {
        Symbol::Ternary(Ternary::ConditionalElse) => Precedence::TERNARY,
        Symbol::Ternary(Ternary::Conditional) => Precedence::Conditional,
        Symbol::Binary(Binary::LogicOr) => Precedence::LogicOr,
        Symbol::Binary(Binary::LogicAnd) => Precedence::LogicAnd,
        Symbol::Relation(Relation::Equal | Relation::NotEqual) => Precedence::Equality,
        Symbol::Relation(
            Relation::LessThan
            | Relation::GreaterThan
            | Relation::LessEqual
            | Relation::GreaterEqual,
        ) => Precedence::Relational,
        Symbol::Binary(Binary::Add | Binary::Subtract) => Precedence::Additive,
        Symbol::Binary(Binary::Multiply | Binary::Divide | Binary::Modulus) => {
            Precedence::Multiplicative
        }
        Symbol::Binary(Binary::Power) => Precedence::Power,
        Symbol::Unary(_) => Precedence::Unary,
        _ => Precedence::Lowest,
    }
}

/// 根据符号获取符号类型
pub fn get_symbol_type(symbol: &Symbol) -> SymbolType {
    match symbol {
        Symbol::Binary(
            Binary::Add
            | Binary::Subtract
            | Binary::Multiply
            | Binary::Divide
            | Binary::Modulus
            | Binary::Power,
        )
        | Symbol::Unary(Unary::Plus | Unary::Minus) => SymbolType::Arithmetic,
        Symbol::Relation(_) => SymbolType::Relational,
        Symbol::Binary(Binary::LogicAnd | Binary::LogicOr) | Symbol::Unary(Unary::LogicNot) => {
            SymbolType::Logical
        }
        Symbol::Ternary(_) => SymbolType::Conditional,
        _ => SymbolType::Other,
    }
}

// TODO 仿照 display 实现 latex 风格输出
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
