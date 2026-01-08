pub mod expression;

use crate::lexer::token::Token;
use crate::lexer::Lexer;
use crate::lexer::symbol::{get_precedence, get_symbol_type, Precedence, Symbol, SymbolType};
use crate::parser::expression::Expression;

pub fn pratt_parsing(lexer: &mut Lexer, min_precedence: Precedence) -> Expression {
    // nud
    let mut left_expr = match lexer.next_token() {
        Some(Token::Constant(c)) => Expression::constant(c),
        Some(Token::Variable(v)) => Expression::variable(v),
        Some(Token::Symbol(s@ (Symbol::Minus | Symbol::LogicNot))) => {
            let expr = pratt_parsing(lexer, Precedence::Unary);
            Expression::unary(s, expr)
        },
        Some(Token::Symbol(Symbol::LeftParen)) => {
            let expr = pratt_parsing(lexer, Precedence::Lowest);
            if let Some(Token::Symbol(Symbol::RightParen)) = lexer.next_token() {
                expr
            } else {
                panic!("expected closing parenthesis");
            }
        },
        _ => panic!("unexpected token"),
    };

    // led
    loop {
        let operation = match lexer.peek_token() {
            Some(Token::Symbol(Symbol::RightParen)) => break,
            Some(Token::Symbol(s)) => s,
            None => break,
            _ => panic!("unexpected token, expected operator"),
        };

        if get_precedence(&operation) < min_precedence {
            break;
        }

        if operation == Symbol::Conditional {
            lexer.next_token(); // consume '?'
            let then_expr = pratt_parsing(lexer, Precedence::Conditional);
            if let Some(Token::Symbol(Symbol::ConditionalElse)) = lexer.next_token() {
                let else_expr = pratt_parsing(lexer, Precedence::Conditional);
                left_expr = Expression::ternary(left_expr, Symbol::Conditional, then_expr, Symbol::ConditionalElse, else_expr);
                continue;
            } else {
                panic!("expected ':' in ternary expression, found {:?}", lexer.peek_token());
            }
        }

        lexer.next_token(); // consume operator
        let right = pratt_parsing(lexer, get_precedence(&operation));
        left_expr = match get_symbol_type(&operation) {
            SymbolType::Relational => Expression::relation(left_expr, operation, right),
            SymbolType::Arithmetic | SymbolType::Logical => Expression::binary(left_expr, operation, right),
            _ => panic!("unsupported operator '{}'", operation),
        };
    }

    left_expr
}