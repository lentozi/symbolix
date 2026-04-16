use std::panic::{catch_unwind, AssertUnwindSafe};

use symbolix_core::{lexer::{symbol::{Binary, Symbol}, token::Token, Lexer}, new_compile_context};

#[test]
fn lexer_peek_next_and_tokens_handle_stream_progression() {
    let mut lexer = Lexer::new("x + 1");
    assert!(matches!(lexer.peek_token(), Some(Token::Variable(ref v)) if v == "x"));
    assert!(matches!(lexer.peek_token(), Some(Token::Variable(ref v)) if v == "x"));

    assert!(matches!(lexer.next_token(), Some(Token::Variable(ref v)) if v == "x"));
    assert!(matches!(lexer.next_token(), Some(Token::Symbol(Symbol::Binary(Binary::Add)))));
    assert!(matches!(lexer.peek_token(), Some(Token::Constant(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Constant(_))));
    assert!(lexer.next_token().is_none());
    assert!(lexer.peek_token().is_none());

    let mut lexer = Lexer::new("a && b");
    let tokens = lexer.tokens();
    assert_eq!(tokens.len(), 3);
}

#[test]
fn lexer_panics_on_invalid_input_through_context() {
    let result = catch_unwind(AssertUnwindSafe(|| {
        new_compile_context! {
            let lexer = Lexer::new("@");
            let _ = lexer.peek_token();
        }
    }));
    assert!(result.is_err());
}
