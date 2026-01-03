use std::str::FromStr;
use nom::character::complete::{digit1, i64, multispace0};
use nom::{IResult, Parser};
use crate::lexer::symbol::Symbol;
use crate::lexer::token::Token;
use crate::lexer::constant::{Constant, Number};
use nom::sequence::{preceded, separated_pair};
use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::bytes::tag;
use nom::character::char;
use nom::character::complete::digit0;
use nom::combinator::{map, map_res, recognize};
use crate::lexer::variable::Variable;

pub mod constant;
pub mod symbol;
pub mod token;
pub mod variable;

fn parse_integer(input: &str) -> IResult<&str, i64> {
    i64(input)
}

fn parse_float(input: &str) -> IResult<&str, f64> {
    // xx.xx    xx.     .xx
    let float_str = recognize(alt((
        separated_pair(digit0, char('.'), digit0),
        separated_pair(digit1, tag("e"), digit1),
    )));

    map_res(float_str, |s: &str| f64::from_str(s)).parse(input)
}

fn parse_boolean(input: &str) -> IResult<&str, bool> {
    alt((
        map(tag("true"), |_| true),
        map(tag("false"), |_| false),
    )).parse(input)
}

fn parse_symbol(input: &str) -> IResult<&str, Symbol> {
    alt((
            map(tag("+"), |_| Symbol::Plus),
            map(tag("-"), |_| Symbol::Minus),
            map(tag("*"), |_| Symbol::Asterisk),
            map(tag("/"), |_| Symbol::Slash),
            map(tag("%"), |_| Symbol::Percent),
            map(tag("^"), |_| Symbol::Caret),
            map(tag("=="), |_| Symbol::Equal),
            map(tag("!="), |_| Symbol::NotEqual),
            map(tag("<="), |_| Symbol::LessEqual),
            map(tag(">="), |_| Symbol::GreaterEqual),
            map(tag("<"), |_| Symbol::LessThan),
            map(tag(">"), |_| Symbol::GreaterThan),
            map(tag("("), |_| Symbol::LeftParen),
            map(tag(")"), |_| Symbol::RightParen),
            map(tag(","), |_| Symbol::Comma),
            map(tag(";"), |_| Symbol::Semicolon),
            map(tag("?"), |_| Symbol::Conditional),
            map(tag(":"), |_| Symbol::ConditionalElse),
            map(tag("&&"), |_| Symbol::LogicAdd),
            map(tag("||"), |_| Symbol::LogicOr),
            map(tag("!"), |_| Symbol::LogicNot),
    )).parse(input)
}

fn parse_variable(input: &str) -> IResult<&str, Variable> {
    let first_char = nom::character::complete::satisfy(|c| c.is_alphabetic() || c == '_');
    let rest_chars = take_while(|c: char| c.is_alphanumeric() || c == '_');

    let var_parser = recognize(
        nom::sequence::pair(first_char, rest_chars)
    );

    map(var_parser, |s: &str| Variable::new(s)).parse(input)
}

/// Parse a single token from the start of `input`, skipping leading whitespace.
/// Returns the remaining input and the parsed `Token`.
pub fn parse_token(input: &str) -> IResult<&str, Token> {
    preceded(
        multispace0,
        alt((
            map(parse_boolean, |b| Token::constant(Constant::boolean(b))),
            map(parse_variable, |v| Token::variable(v)),
            map(parse_float, |n| Token::constant(Constant::number(Number::float(n)))),
            map(parse_integer, |n| Token::constant(Constant::number(Number::integer(n)))),
            map(parse_symbol, |s| Token::symbol(s)),
        ))
    ).parse(input)
}

pub struct Lexer {
    remaining: String,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let appended_input = format!(" {} ", input);
        Lexer {
            remaining: appended_input.to_string(),
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        if self.remaining.is_empty() {
            return None;
        }
        match parse_token(self.remaining.as_str()) {
            Ok((new_rest, token)) => {
                self.remaining = new_rest.trim_start().to_string();
                Some(token)
            }
            Err(e) => {
                eprintln!("Error parsing token: {:?}", e);
                None
            }
        }
    }

    pub fn peek_token(&self) -> Option<Token> {
        if self.remaining.is_empty() {
            return None;
        }
        match parse_token(self.remaining.as_str()) {
            Ok((_, token)) => Some(token),
            Err(e) => {
                eprintln!("Error parsing token: {:?}", e);
                None
            }
        }
    }
}
