use symbolix::parser::expression::Expression;
use symbolix::lexer::constant::{Constant, Number};
use symbolix::lexer::Lexer;
use symbolix::lexer::symbol::Symbol;
use symbolix::lexer::variable::Variable;

#[test]
fn test_unary_parsing() {
    let input = "-x + y - !z";
    let expected_expression = Expression::binary(
        Expression::unary(Symbol::Minus, Expression::variable(Variable::new("x"))),
        Symbol::Plus,
        Expression::binary(
            Expression::variable(Variable::new("y")),
            Symbol::Minus,
            Expression::unary(Symbol::LogicNot, Expression::variable(Variable::new("z"))),
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
            Symbol::LogicNot,
            Expression::unary(Symbol::LogicNot, Expression::variable(Variable::new("a"))),
        ),
        Symbol::Plus,
        Expression::unary(
            Symbol::Minus,
            Expression::unary(Symbol::Minus, Expression::variable(Variable::new("b"))),
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
            Expression::unary(Symbol::LogicNot, Expression::variable(Variable::new("a"))),
            Symbol::LogicAdd,
            Expression::variable(Variable::new("b")),
        ),
        Symbol::LogicOr,
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
        Expression::variable(Variable::new("a")),
        Symbol::Plus,
        Expression::binary(
            Expression::binary(
                Expression::variable(Variable::new("b")),
                Symbol::Asterisk,
                Expression::variable(Variable::new("c")),
            ),
            Symbol::Minus,
            Expression::binary(
                Expression::variable(Variable::new("d")),
                Symbol::Slash,
                Expression::variable(Variable::new("e")),
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
        Expression::binary(
            Expression::variable(Variable::new("x")),
            Symbol::GreaterThan,
            Expression::constant(Constant::number(Number::integer(100))),
        ),
        Symbol::Conditional,
        Expression::binary(
            Expression::variable(Variable::new("x")),
            Symbol::Asterisk,
            Expression::binary(
                Expression::constant(Constant::number(Number::integer(2))),
                Symbol::Plus,
                Expression::constant(Constant::number(Number::integer(3))),
            ),
        ),
        Symbol::ConditionalElse,
        Expression::binary(
            Expression::variable(Variable::new("x")),
            Symbol::Slash,
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
        Expression::binary(
            Expression::variable(Variable::new("a")),
            Symbol::GreaterThan,
            Expression::variable(Variable::new("b")),
        ),
        Symbol::Conditional,
        Expression::ternary(
            Expression::binary(
                Expression::variable(Variable::new("c")),
                Symbol::LessThan,
                Expression::variable(Variable::new("d")),
            ),
            Symbol::Conditional,
            Expression::variable(Variable::new("e")),
            Symbol::ConditionalElse,
            Expression::variable(Variable::new("f")),
        ),
        Symbol::ConditionalElse,
        Expression::variable(Variable::new("g")),
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