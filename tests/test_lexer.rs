use symbolix::lexer::Lexer;
use symbolix::lexer::constant::{Constant, Number};
use symbolix::lexer::symbol::{Binary, Other, Relation, Symbol, Ternary, Unary};
use symbolix::lexer::token::Token;
use symbolix::lexer::variable::Variable;

#[test]
fn test_parsing() {
    let input = "123 + 45.67 * (89 - 0.1)";
    let mut lexer = Lexer::new(input);
    let expected_tokens = vec![
        Token::Constant(Constant::Number(Number::Integer(123))),
        Token::Symbol(Symbol::Binary(Binary::Add)),
        Token::Constant(Constant::Number(Number::Float(45.67))),
        Token::Symbol(Symbol::Binary(Binary::Multiply)),
        Token::Symbol(Symbol::Other(Other::LeftParen)),
        Token::Constant(Constant::Number(Number::Integer(89))),
        Token::Symbol(Symbol::Binary(Binary::Subtract)),
        Token::Constant(Constant::Number(Number::Float(0.1))),
        Token::Symbol(Symbol::Other(Other::RightParen)),
    ];

    for expected in expected_tokens {
        let token = lexer.next_token().expect("Expected a token");
        assert_eq!(token, expected);
    }

    assert!(lexer.next_token().is_none());
}

#[test]
fn test_ternary_expression() {
    let input = "x > 0 ? x : -x";
    let mut lexer = Lexer::new(input);
    let expected_tokens = vec![
        Token::Variable(Variable::new("x")),
        Token::Symbol(Symbol::Relation(Relation::GreaterThan)),
        Token::Constant(Constant::Number(Number::Integer(0))),
        Token::Symbol(Symbol::Ternary(Ternary::Conditional)),
        Token::Variable(Variable::new("x")),
        Token::Symbol(Symbol::Ternary(Ternary::ConditionalElse)),
        Token::Symbol(Symbol::Binary(Binary::Subtract)),
        Token::Variable(Variable::new("x")),
    ];

    for expected in expected_tokens {
        let token = lexer.next_token().expect("Expected a token");
        assert_eq!(token, expected);
    }

    assert!(lexer.next_token().is_none());
}

#[test]
fn test_logical_expression() {
    let input = "a && true || !c";
    let mut lexer = Lexer::new(input);
    let expected_tokens = vec![
        Token::Variable(Variable::new("a")),
        Token::Symbol(Symbol::Binary(Binary::LogicAnd)),
        Token::Constant(Constant::boolean(true)),
        Token::Symbol(Symbol::Binary(Binary::LogicOr)),
        Token::Symbol(Symbol::Unary(Unary::LogicNot)),
        Token::Variable(Variable::new("c")),
    ];

    for expected in expected_tokens {
        let token = lexer.next_token().expect("Expected a token");
        assert_eq!(token, expected);
    }

    assert!(lexer.next_token().is_none());
}

#[test]
fn test_variable_parsing() {
    let input = "var_name123 + anotherVar - _tempVar";
    let mut lexer = Lexer::new(input);
    let expected_tokens = vec![
        Token::Variable(Variable::new("var_name123")),
        Token::Symbol(Symbol::Binary(Binary::Add)),
        Token::Variable(Variable::new("anotherVar")),
        Token::Symbol(Symbol::Binary(Binary::Subtract)),
        Token::Variable(Variable::new("_tempVar")),
    ];

    for expected in expected_tokens {
        let token = lexer.next_token().expect("Expected a token");
        assert_eq!(token, expected);
    }

    assert!(lexer.next_token().is_none());
}

#[test]
fn test_boolean_parsing() {
    let input = "true && false || true";
    let mut lexer = Lexer::new(input);
    let expected_tokens = vec![
        Token::Constant(Constant::Boolean(true)),
        Token::Symbol(Symbol::Binary(Binary::LogicAnd)),
        Token::Constant(Constant::Boolean(false)),
        Token::Symbol(Symbol::Binary(Binary::LogicOr)),
        Token::Constant(Constant::Boolean(true)),
    ];

    for expected in expected_tokens {
        let token = lexer.next_token().expect("Expected a token");
        assert_eq!(token, expected);
    }

    assert!(lexer.next_token().is_none());
}
