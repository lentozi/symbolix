use exprion_core::{
    lexer::constant::{Fraction, Number},
    new_compile_context,
};

#[test]
fn lexer_tokenizes_identifiers() {
    use exprion_core::lexer::token::Token;
    use exprion_core::lexer::Lexer;

    let input = "foo bar _baz qux123";
    let mut lexer = Lexer::new(input);

    let expected = vec![
        Token::variable(String::from("foo")),
        Token::variable(String::from("bar")),
        Token::variable(String::from("_baz")),
        Token::variable(String::from("qux123")),
    ];

    assert_eq!(lexer.tokens(), expected);
}

#[test]
fn lexer_tokenizes_float_and_scientific() {
    use exprion_core::lexer::constant::Constant;
    use exprion_core::lexer::token::Token;
    use exprion_core::lexer::Lexer;

    let input = "123.456 1.23e-4 1E6";
    let mut lexer = Lexer::new(input);

    let expected = vec![
        Token::constant(Constant::float(123.456)),
        Token::constant(Constant::float(1.23e-4)),
        Token::constant(Constant::float(1e6)),
    ];

    assert_eq!(lexer.tokens(), expected);
}

#[test]
fn lexer_tokenizes_operators_and_punctuators() {
    use exprion_core::lexer::symbol::{Binary, Relation, Symbol, Unary};
    use exprion_core::lexer::token::Token;
    use exprion_core::lexer::Lexer;

    new_compile_context! {
        let input = "+ - * / % ^ && || ! < > <= >= == !=";
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::symbol(Symbol::Binary(Binary::Add)),
            Token::symbol(Symbol::Binary(Binary::Subtract)),
            Token::symbol(Symbol::Binary(Binary::Multiply)),
            Token::symbol(Symbol::Binary(Binary::Divide)),
            Token::symbol(Symbol::Binary(Binary::Modulus)),
            Token::symbol(Symbol::Binary(Binary::Power)),
            Token::symbol(Symbol::Binary(Binary::LogicAnd)),
            Token::symbol(Symbol::Binary(Binary::LogicOr)),
            Token::symbol(Symbol::Unary(Unary::LogicNot)),
            Token::symbol(Symbol::Relation(Relation::LessThan)),
            Token::symbol(Symbol::Relation(Relation::GreaterThan)),
            Token::symbol(Symbol::Relation(Relation::LessEqual)),
            Token::symbol(Symbol::Relation(Relation::GreaterEqual)),
            Token::symbol(Symbol::Relation(Relation::Equal)),
            Token::symbol(Symbol::Relation(Relation::NotEqual)),
        ];

        assert_eq!(lexer.tokens(), expected);
    }
}

#[test]
fn lexer_handles_unicode_identifiers() {
    use exprion_core::lexer::token::Token;
    use exprion_core::lexer::Lexer;

    new_compile_context! {
        let mut lexer = Lexer::new("变量 αβγδε");

        let expected = vec![
            Token::variable(String::from("变量")),
            Token::variable(String::from("αβγδε")),
        ];

        assert_eq!(lexer.tokens(), expected);
    }
}

#[test]
fn lexer_position_and_span_consistency() {
    use exprion_core::lexer::token::Token;
    use exprion_core::lexer::Lexer;

    new_compile_context! {
        let input = "foo bar _baz
                    qux123";
        let mut lexer = Lexer::new(input);

        let expected = vec![
            Token::variable(String::from("foo")),
            Token::variable(String::from("bar")),
            Token::variable(String::from("_baz")),
            Token::variable(String::from("qux123")),
        ];

        assert_eq!(lexer.tokens(), expected);
    }
}

#[test]
#[should_panic]
fn lexer_reports_numeric_errors() {
    new_compile_context! {
        let _frac = Fraction::new(5, 0);
    }
}

#[test]
#[should_panic]
fn lexer_reports_invalid_division() {
    new_compile_context! {
        let frac = Number::fraction(5, 2);
        let zero = Number::integer(0);
        let _div = frac / zero;
    }
}

#[test]
fn lexer_reports_invalid_token_with_position() {
    use exprion_core::lexer::token::Token;
    use exprion_core::lexer::Lexer;

    new_compile_context! {
        let mut lexer = Lexer::new("foo bar _baz
                    123qux");

        let expected = vec![
            Token::variable(String::from("foo")),
            Token::variable(String::from("bar")),
            Token::variable(String::from("_baz")),
            Token::invalid("123qux"),
        ];

        assert_eq!(lexer.tokens(), expected);
    }
}
