use std::cell::{Cell, RefCell};

use crate::{
    error::ErrorExt,
    lexer::{symbol::Symbol, token::Token, tools::parse_token_no_ws},
    push_compile_error,
};

pub mod constant;
mod macros;
mod number_trait;
pub mod symbol;
pub mod token;
mod tools;

pub use number_trait::NumberTrait;

#[doc(hidden)]
pub mod testing {
    pub use super::tools::{parse_float_token, parse_token};
}

/// 词法分析器，支持流式输出 token 和输出 token 数组
pub struct Lexer {
    input: String,
    offset: Cell<usize>,
    lookahead: RefCell<Option<Lookahead>>,
}

#[derive(Clone)]
struct Lookahead {
    consumed: usize,
    token: Token,
}

impl Lexer {
    /// 初始化词法分析器，输入为要进行词法分析的字符串
    pub fn new(input: &str) -> Self {
        let appended_input = format!(" {} ", input);
        Lexer {
            input: appended_input,
            offset: Cell::new(0),
            lookahead: RefCell::new(None),
        }
    }

    /// 流式输出 token，每次调用返回下一个 token 值
    pub fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let current = self.remaining();
        if current.is_empty() {
            self.offset.set(self.input.len());
            self.lookahead.borrow_mut().take();
            return None;
        }

        if let Some(lookahead) = self.lookahead.borrow_mut().take() {
            self.offset.set(self.offset.get() + lookahead.consumed);
            return Some(lookahead.token);
        }

        self.parse_current_token(current).map(|lookahead| {
            self.offset.set(self.offset.get() + lookahead.consumed);
            lookahead.token
        })
    }

    /// 查看下一个 token 值但不对其进行消耗
    pub fn peek_token(&self) -> Option<Token> {
        self.skip_whitespace();
        let current = self.remaining();
        if current.is_empty() {
            self.lookahead.borrow_mut().take();
            return None;
        }

        if let Some(lookahead) = self.lookahead.borrow().as_ref() {
            return Some(lookahead.token.clone());
        }

        let lookahead = self.parse_current_token(current)?;
        let token = lookahead.token.clone();
        *self.lookahead.borrow_mut() = Some(lookahead);
        Some(token)
    }

    pub fn peek_symbol(&self) -> Option<Symbol> {
        match self.peek_token() {
            Some(Token::Symbol(symbol)) => Some(symbol),
            _ => None,
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

    fn remaining(&self) -> &str {
        &self.input[self.offset.get()..]
    }

    fn parse_current_token(&self, current: &str) -> Option<Lookahead> {
        match parse_token_no_ws(current) {
            Ok((new_rest, token)) => Some(Lookahead {
                consumed: current.len() - new_rest.len(),
                token,
            }),
            Err(e) => {
                push_compile_error!(ErrorExt::lexical_error(
                    format!("词法解析错误：{:?}", e).as_ref(),
                    true
                ));
                None
            }
        }
    }

    fn skip_whitespace(&self) {
        let current = self.remaining();
        let consumed = current
            .char_indices()
            .find(|(_, ch)| !ch.is_whitespace())
            .map(|(index, _)| index)
            .unwrap_or(current.len());

        if consumed > 0 {
            let mut lookahead = self.lookahead.borrow_mut();
            if lookahead.is_some() {
                lookahead.take();
            }
            self.offset.set(self.offset.get() + consumed);
        }
    }
}
