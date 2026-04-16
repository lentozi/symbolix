//! 词法分析工具模块，提供了一些常用的词法分析工具函数。

use crate::error::ErrorExt;
use crate::lexer::constant::{Constant, Number};
use crate::lexer::symbol::Symbol;
use crate::lexer::symbol::{Binary, Other, Relation, Ternary, Unary};
use crate::lexer::token::Token;
use crate::push_compile_error;
use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::bytes::tag;
use nom::character::char;
use nom::character::complete::digit0;
use nom::character::complete::{digit1, i64, multispace0};
use nom::combinator::{map, map_res, opt, recognize};
use nom::sequence::preceded;
use nom::{IResult, Parser};
use std::str::FromStr;

/// 解析整数的工具函数，用于词法分析器内部使用。
fn parse_integer_tool(input: &str) -> IResult<&str, i64> {
    i64(input)
}

/// 解析浮点数的工具函数，用于词法分析器内部使用。
fn parse_float_tool(input: &str) -> IResult<&str, f64> {
    // 三种被认为是“浮点”的情况（都不包含纯整数）：
    // 1) 有小数点的形式（1.23 或 1. 或 1.0），可选指数
    let float_with_point = recognize((sign, digit1, char('.'), digit0, opt(exponent)));
    // 2) 小数点开头的形式（.23），可选指数
    let point_leading = recognize((sign, char('.'), digit1, opt(exponent)));
    // 3) 整数 + 指数 的形式（1e3、1E-4）
    let int_with_exp = recognize((sign, digit1, exponent));

    // 选择三种形式中的任意一种
    let float_parser = alt((float_with_point, point_leading, int_with_exp));

    // 把匹配到的 &str 解析成 f64
    map_res(float_parser, |s: &str| f64::from_str(s)).parse(input)
}

/// 解析布尔值的工具函数，用于词法分析器内部使用。
fn parse_boolean(input: &str) -> IResult<&str, bool> {
    alt((map(tag("true"), |_| true), map(tag("false"), |_| false))).parse(input)
}

/// 解析符号的工具函数，用于词法分析器内部使用。
fn parse_symbol(input: &str) -> IResult<&str, Symbol> {
    alt((
        map(tag("+"), |_| Symbol::Binary(Binary::Add)),
        map(tag("-"), |_| Symbol::Binary(Binary::Subtract)),
        map(tag("*"), |_| Symbol::Binary(Binary::Multiply)),
        map(tag("/"), |_| Symbol::Binary(Binary::Divide)),
        map(tag("%"), |_| Symbol::Binary(Binary::Modulus)),
        map(tag("^"), |_| Symbol::Binary(Binary::Power)),
        map(tag("=="), |_| Symbol::Relation(Relation::Equal)),
        map(tag("!="), |_| Symbol::Relation(Relation::NotEqual)),
        map(tag("<="), |_| Symbol::Relation(Relation::LessEqual)),
        map(tag(">="), |_| Symbol::Relation(Relation::GreaterEqual)),
        map(tag("<"), |_| Symbol::Relation(Relation::LessThan)),
        map(tag(">"), |_| Symbol::Relation(Relation::GreaterThan)),
        map(tag("("), |_| Symbol::Other(Other::LeftParen)),
        map(tag(")"), |_| Symbol::Other(Other::RightParen)),
        map(tag(","), |_| Symbol::Other(Other::Comma)),
        map(tag(";"), |_| Symbol::Other(Other::Semicolon)),
        map(tag("?"), |_| Symbol::Ternary(Ternary::Conditional)),
        map(tag(":"), |_| Symbol::Ternary(Ternary::ConditionalElse)),
        map(tag("&&"), |_| Symbol::Binary(Binary::LogicAnd)),
        map(tag("||"), |_| Symbol::Binary(Binary::LogicOr)),
        map(tag("!"), |_| Symbol::Unary(Unary::LogicNot)),
    ))
    .parse(input)
}

/// 解析变量的工具函数，用于词法分析器内部使用。
fn parse_variable(input: &str) -> IResult<&str, String> {
    let first_char = nom::character::complete::satisfy(|c| c.is_alphabetic() || c == '_');
    let rest_chars = take_while(|c: char| c.is_alphanumeric() || c == '_');

    let var_parser = recognize(nom::sequence::pair(first_char, rest_chars));

    map(var_parser, |s: &str| s.to_string()).parse(input)
}

/// 可选符号解析器，返回 `Option<char>`（Some('+')/Some('-')/None）
fn sign(input: &str) -> IResult<&str, Option<char>> {
    opt(alt((char('+'), char('-')))).parse(input)
}

/// 指数部分解析器：匹配 `e` 或 `E`，可选 `+`/`-`，后面至少一位数字
/// 返回 (char, Option<char>, &str) 这样的元组（例如 ('e', Some('+'), "12")）
fn exponent(input: &str) -> IResult<&str, (char, Option<char>, &str)> {
    (
        alt((char('e'), char('E'))),
        opt(alt((char('+'), char('-')))),
        digit1,
    )
        .parse(input)
}

/// 工具函数，如果剩余输入第一个字符是字母或下划线，返回true
fn rest_starts_with_ident(rest: &str) -> bool {
    rest.chars()
        .next()
        .map_or(false, |c| c.is_alphabetic() || c == '_')
}

/// 解析整数词法单元的工具函数，用于词法分析器内部使用。
fn parse_integer_token(input: &str) -> IResult<&str, Token> {
    match parse_integer_tool(input) {
        Ok((rest, n)) => {
            if rest_starts_with_ident(rest) {
                // 数字后紧跟标识符，视为词法错误
                push_compile_error!(ErrorExt::lexical_error("无效标识符", false));

                // 消费该标识符部分，组合成非法片段
                let mut ident_len_bytes = 0usize;
                for c in rest.chars() {
                    if c.is_alphabetic() || c == '_' {
                        ident_len_bytes += c.len_utf8();
                    } else {
                        break;
                    }
                }
                let consumed_bytes = input.len() - rest.len() + ident_len_bytes;
                let invalid_str = &input[..consumed_bytes];
                let new_rest = &input[consumed_bytes..];
                Ok((new_rest, Token::invalid(invalid_str)))
            } else {
                Ok((rest, Token::constant(Constant::number(Number::integer(n)))))
            }
        }
        Err(e) => Err(e), // 错误留给上层统一处理
    }
}

/// 解析浮点数词法单元的工具函数，用于词法分析器内部使用。
pub fn parse_float_token(input: &str) -> IResult<&str, Token> {
    match parse_float_tool(input) {
        Ok((rest, n)) => {
            if rest_starts_with_ident(rest) {
                // 数字后紧跟标识符，视为词法错误
                push_compile_error!(ErrorExt::lexical_error("无效标识符", false));

                // 消费该标识符部分，组合成非法片段
                let mut ident_len_bytes = 0usize;
                for c in rest.chars() {
                    if c.is_alphabetic() || c == '_' {
                        ident_len_bytes += c.len_utf8();
                    } else {
                        break;
                    }
                }
                let consumed_bytes = input.len() - rest.len() + ident_len_bytes;
                let invalid_str = &input[..consumed_bytes];
                let new_rest = &input[consumed_bytes..];
                Ok((new_rest, Token::invalid(invalid_str)))
            } else {
                Ok((rest, Token::constant(Constant::number(Number::float(n)))))
            }
        }
        Err(e) => Err(e), // 错误留给上层统一处理
    }
}

/// Parse a single token from the start of `input`, skipping leading whitespace.
/// Returns the remaining input and the parsed `Token`.
pub fn parse_token(input: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            map(parse_boolean, |b| Token::constant(Constant::boolean(b))),
            map(parse_variable, |v| Token::variable(v)),
            map(parse_symbol, |s| Token::symbol(s)),
            map(parse_float_token, |n| n),
            map(parse_integer_token, |n| n),
        )),
    )
    .parse(input)
}
