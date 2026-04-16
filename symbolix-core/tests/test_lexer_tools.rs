use std::panic::{catch_unwind, AssertUnwindSafe};

use symbolix_core::{
    lexer::{token::Token, Lexer},
    new_compile_context,
};

#[test]
fn parse_float_token_handles_valid_and_invalid_identifier_suffixes() {
    let mut lexer = Lexer::new("1.5 foo");
    assert!(matches!(lexer.next_token(), Some(Token::Constant(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Variable(ref v)) if v == "foo"));

    new_compile_context! {
        let mut lexer = Lexer::new("1.5abc");
        assert!(matches!(lexer.next_token(), Some(Token::Invalid(_))));
        assert!(lexer.next_token().is_none());
    }
}

#[test]
fn parse_token_handles_boolean_variable_symbol_and_integer_invalid_suffix() {
    let mut lexer = Lexer::new("true && flag");
    assert!(matches!(lexer.next_token(), Some(Token::Constant(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Symbol(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Variable(ref v)) if v == "flag"));

    let mut lexer = Lexer::new("value");
    assert!(matches!(lexer.next_token(), Some(Token::Variable(_))));
    assert!(lexer.next_token().is_none());

    let mut lexer = Lexer::new("<= x");
    assert!(matches!(lexer.next_token(), Some(Token::Symbol(_))));
    assert!(matches!(lexer.next_token(), Some(Token::Variable(ref v)) if v == "x"));

    let result = catch_unwind(AssertUnwindSafe(|| {
        new_compile_context! {
            let mut lexer = Lexer::new("12abc");
            assert!(matches!(lexer.next_token(), Some(Token::Invalid(_))));
            assert!(lexer.next_token().is_none());
        }
    }));
    assert!(result.is_ok());
}
