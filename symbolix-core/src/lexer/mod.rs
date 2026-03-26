use crate::{
    error::ErrorExt,
    lexer::{token::Token, tools::parse_token},
    push_compile_error,
};

pub mod constant;
pub mod symbol;
pub mod token;
mod tools;
mod macros;

/// 词法分析器，支持流式输出 token 和输出 token 数组
pub struct Lexer {
    remaining: String,
}

impl Lexer {
    /// 初始化词法分析器，输入为要进行词法分析的字符串
    pub fn new(input: &str) -> Self {
        let appended_input = format!(" {} ", input);
        Lexer {
            remaining: appended_input.to_string(),
        }
    }

    /// 流式输出 token，每次调用返回下一个 token 值
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
                push_compile_error!(ErrorExt::lexical_error(
                    format!("词法解析错误：{:?}", e).as_ref(),
                    true
                ));
                None
            }
        }
    }

    /// 查看下一个 token 值但不对其进行消耗
    pub fn peek_token(&self) -> Option<Token> {
        if self.remaining.is_empty() {
            return None;
        }
        match parse_token(self.remaining.as_str()) {
            Ok((_, token)) => Some(token),
            Err(e) => {
                push_compile_error!(ErrorExt::lexical_error(
                    format!("词法解析错误：{:?}", e).as_ref(),
                    true
                ));
                None
            }
        }
    }

    /// 一次性返回所有 token
    pub fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            tokens.push(token);
        }
        tokens
    }
}
