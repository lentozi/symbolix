pub mod expression;

use crate::lexer::symbol::{get_precedence, Precedence, Symbol};
use crate::lexer::symbol::{Binary, Other, Ternary, Unary};
use crate::lexer::token::Token;
use crate::lexer::Lexer;
use crate::parser::expression::Expression;

pub struct Parser;

impl Parser {
    pub fn pratt(lexer: &mut Lexer) -> Expression {
        pratt_parsing(lexer, Precedence::Lowest)
    }
}

fn infix_right_precedence(operation: &Symbol) -> Precedence {
    match operation {
        Symbol::Binary(Binary::Power) => get_precedence(operation),
        Symbol::Binary(_) | Symbol::Relation(_) => match get_precedence(operation) {
            Precedence::TERNARY => Precedence::Conditional,
            Precedence::Conditional => Precedence::LogicOr,
            Precedence::LogicOr => Precedence::LogicAnd,
            Precedence::LogicAnd => Precedence::Equality,
            Precedence::Equality => Precedence::Relational,
            Precedence::Relational => Precedence::Additive,
            Precedence::Additive => Precedence::Multiplicative,
            Precedence::Multiplicative => Precedence::Power,
            Precedence::Power => Precedence::Unary,
            Precedence::Unary | Precedence::Lowest => Precedence::Unary,
        },
        _ => get_precedence(operation),
    }
}

fn pratt_parsing(lexer: &mut Lexer, min_precedence: Precedence) -> Expression {
    // nud
    let mut left_expr = match lexer.next_token() {
        Some(Token::Constant(c)) => Expression::constant(c),
        Some(Token::Variable(v)) => Expression::variable(v),
        Some(Token::Symbol(s@ (Symbol::Binary(Binary::Subtract) | Symbol::Binary(Binary::Add) | Symbol::Unary(Unary::LogicNot)))) => {
            let expr = pratt_parsing(lexer, Precedence::Unary);
            let symbol = match s {
                Symbol::Binary(Binary::Add) => Symbol::Unary(Unary::Plus),
                Symbol::Binary(Binary::Subtract) => Symbol::Unary(Unary::Minus),
                Symbol::Unary(Unary::LogicNot) => Symbol::Unary(Unary::LogicNot),
                _ => panic!("unsupported unary operation {}", s)
            };
            Expression::unary(symbol, expr)
        },
        Some(Token::Symbol(Symbol::Other(Other::LeftParen))) => {
            let expr = pratt_parsing(lexer, Precedence::Lowest);
            if let Some(Token::Symbol(Symbol::Other(Other::RightParen))) = lexer.next_token() {
                expr
            } else {
                panic!("expected closing parenthesis");
            }
        },
        _ => panic!("unexpected token"),
    };

    // led
    loop {
        let operation = match lexer.peek_symbol() {
            Some(Symbol::Other(Other::RightParen)) => break,
            Some(s @ (Symbol::Binary(_) | Symbol::Ternary(_) | Symbol::Relation(_))) => s,
            None => break,
            _ => panic!("unexpected token, expected operator"),
        };

        if get_precedence(&operation) < min_precedence {
            break;
        }

        if operation == Symbol::Ternary(Ternary::Conditional) {
            lexer.next_token(); // consume '?'
            let then_expr = pratt_parsing(lexer, Precedence::Conditional);
            if let Some(Token::Symbol(Symbol::Ternary(Ternary::ConditionalElse))) = lexer.next_token() {
                let else_expr = pratt_parsing(lexer, Precedence::Conditional);
                left_expr = Expression::ternary(left_expr, Symbol::Ternary(Ternary::Conditional), then_expr, Symbol::Ternary(Ternary::ConditionalElse), else_expr);
                continue;
            } else {
                panic!("expected ':' in ternary expression, found {:?}", lexer.peek_token());
            }
        }

        lexer.next_token(); // consume operator
        let right = pratt_parsing(lexer, infix_right_precedence(&operation));
        left_expr = match operation {
            Symbol::Binary(_) => Expression::binary(left_expr, operation, right),
            Symbol::Relation(_) => Expression::relation(left_expr, operation, right),
            _ => panic!("unsupported binary operation {}", operation)
        };
    }

    left_expr
}
