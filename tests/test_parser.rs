use symbolix::parser::expression::Expression;
use symbolix::lexer::constant::{Constant, Number};
use symbolix::lexer::Lexer;
use symbolix::lexer::symbol::{Binary, Relation, Symbol, Ternary, Unary};

#[test]
fn test_unary_parsing() {
    let input = "-x + y - !z";
    let expected_expression = Expression::binary(
        Expression::unary(Symbol::Unary(Unary::Minus), Expression::variable("x".parse().unwrap())),
        Symbol::Binary(Binary::Add),
        Expression::binary(
            Expression::variable("y".parse().unwrap()),
            Symbol::Binary(Binary::Subtract),
            Expression::unary(Symbol::Unary(Unary::LogicNot), Expression::variable("z".parse().unwrap())),
        ),
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(parsed_expression, expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_double_unary_parsing() {
    let input = "!!a + --b";
    let expected_expression = Expression::binary(
        Expression::unary(
            Symbol::Unary(Unary::LogicNot),
            Expression::unary(Symbol::Unary(Unary::LogicNot), Expression::variable("a".parse().unwrap())),
        ),
        Symbol::Binary(Binary::Add),
        Expression::unary(
            Symbol::Unary(Unary::Minus),
            Expression::unary(Symbol::Unary(Unary::Minus), Expression::variable("b".parse().unwrap())),
        ),
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(parsed_expression, expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_logic_parsing() {
    let input = "!a && b || true";
    let expected_expression = Expression::binary(
        Expression::binary(
            Expression::unary(Symbol::Unary(Unary::LogicNot), Expression::variable("a".parse().unwrap())),
            Symbol::Binary(Binary::LogicAnd),
            Expression::variable("b".parse().unwrap()),
        ),
        Symbol::Binary(Binary::LogicOr),
        Expression::constant(Constant::boolean(true)),
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(parsed_expression, expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_symbol_precedence() {
    let input = "a + b * c - d / e";
    let expected_expression = Expression::binary(
        Expression::variable("a".parse().unwrap()),
        Symbol::Binary(Binary::Add),
        Expression::binary(
            Expression::binary(
                Expression::variable("b".parse().unwrap()),
                Symbol::Binary(Binary::Multiply),
                Expression::variable("c".parse().unwrap()),
            ),
            Symbol::Binary(Binary::Subtract),
            Expression::binary(
                Expression::variable("d".parse().unwrap()),
                Symbol::Binary(Binary::Divide),
                Expression::variable("e".parse().unwrap()),
            ),
        )
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(parsed_expression, expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_conditional_parsing() {
    let input = "x > 100 ? x * (2 + 3) : x / 2";
    let expected_expression = Expression::ternary(
        Expression::relation(
            Expression::variable("x".parse().unwrap()),
            Symbol::Relation(Relation::GreaterThan),
            Expression::constant(Constant::number(Number::integer(100))),
        ),
        Symbol::Ternary(Ternary::Conditional),
        Expression::binary(
            Expression::variable("x".parse().unwrap()),
            Symbol::Binary(Binary::Multiply),
            Expression::binary(
                Expression::constant(Constant::number(Number::integer(2))),
                Symbol::Binary(Binary::Add),
                Expression::constant(Constant::number(Number::integer(3))),
            ),
        ),
        Symbol::Ternary(Ternary::ConditionalElse),
        Expression::binary(
            Expression::variable("x".parse().unwrap()),
            Symbol::Binary(Binary::Divide),
            Expression::constant(Constant::number(Number::integer(2))),
        ),
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(parsed_expression, expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_nested_conditional_parsing() {
    let input = "a > b ? c < d ? e : f : g";
    let expected_expression = Expression::ternary(
        Expression::relation(
            Expression::variable("a".parse().unwrap()),
            Symbol::Relation(Relation::GreaterThan),
            Expression::variable("b".parse().unwrap()),
        ),
        Symbol::Ternary(Ternary::Conditional),
        Expression::ternary(
            Expression::relation(
                Expression::variable("c".parse().unwrap()),
                Symbol::Relation(Relation::LessThan),
                Expression::variable("d".parse().unwrap()),
            ),
            Symbol::Ternary(Ternary::Conditional),
            Expression::variable("e".parse().unwrap()),
            Symbol::Ternary(Ternary::ConditionalElse),
            Expression::variable("f".parse().unwrap()),
        ),
        Symbol::Ternary(Ternary::ConditionalElse),
        Expression::variable("g".parse().unwrap()),
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(parsed_expression, expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
#[should_panic(expected = "unexpected token")]
fn test_invalid_expression() {
    let input = "x + * y";
    let mut lexer = Lexer::new(input);
    let _parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
}

#[test]
#[should_panic(expected = "unexpected token")]
fn test_invalid_ternary_expression() {
    let input = "x > 0 ? : x";
    let mut lexer = Lexer::new(input);
    let _parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
}

#[test]
#[should_panic(expected = "expected closing parenthesis")]
fn test_unmatched_parentheses() {
    let input = "(x + y * (z - 1)";
    let mut lexer = Lexer::new(input);
    let _parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
}