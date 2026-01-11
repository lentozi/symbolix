use symbolix::lexer::constant::{Constant, Number};
use symbolix::lexer::Lexer;
use symbolix::lexer::symbol::{Binary, Relation, Symbol, Ternary, Unary};
use symbolix::parser::expression::Expression;
use symbolix::semantic::{ast_to_semantic, clear_symbol_table};
use symbolix::semantic::semantic_expression::{LogicalExpression, NumericExpression, SemanticExpression};
use symbolix::semantic::variable::{Variable, VariableType};

#[test]
fn test_unary_semantic() {
    let x = Variable::new("x", VariableType::Integer);
    let y = Variable::new("y", VariableType::Integer);
    let input = "-x + y";
    let expected_expression = SemanticExpression::Numeric(
        NumericExpression::Addition(vec![
            NumericExpression::Negation(Box::new(NumericExpression::Variable(x))),
            NumericExpression::Variable(y),
        ])
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(ast_to_semantic(&parsed_expression), expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_logic_semantic() {
    let _a = Variable::new("a_bool", VariableType::Boolean);
    let _b = Variable::new("b_bool", VariableType::Boolean);
    let input = "!a_bool && b_bool || true";
    let expected_expression = SemanticExpression::Logical(LogicalExpression::Constant(true));

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(ast_to_semantic(&parsed_expression), expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_common_semantic() {
    let a = Variable::new("a", VariableType::Integer);
    let b = Variable::new("b", VariableType::Integer);
    let c = Variable::new("c", VariableType::Integer);
    let d = Variable::new("d", VariableType::Integer);
    let e = Variable::new("e", VariableType::Integer);

    let input = "a + b * c - d / e";
    let expected_expression = SemanticExpression::Numeric(
        NumericExpression::Addition(vec![
            NumericExpression::Variable(a),
            NumericExpression::Multiplication(vec![
                NumericExpression::Variable(b),
                NumericExpression::Variable(c),
            ]),
            NumericExpression::Multiplication(vec![
                NumericExpression::Constant(Number::integer(-1)),
                NumericExpression::Variable(d),
                NumericExpression::Power {
                    base: Box::new(NumericExpression::Variable(e)),
                    exponent: Box::new(NumericExpression::Constant(Number::integer(-1))),
                },
            ])
        ])
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(ast_to_semantic(&parsed_expression), expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_conditional_parsing() {
    let x = Variable::new("x", VariableType::Integer);
    let input = "x > 100 ? x * (2 + 3) : x / 2";
    let expected_expression = SemanticExpression::Numeric(
        NumericExpression::Piecewise {
            cases: vec![(
                LogicalExpression::Relation {
                    left: Box::new(NumericExpression::Variable(x.clone())),
                    operator: Symbol::Relation(Relation::GreaterThan),
                    right: Box::new(NumericExpression::Constant(Number::integer(100))),
                },
                NumericExpression::Multiplication(vec![
                    NumericExpression::Constant(Number::integer(5)),
                    NumericExpression::Variable(x.clone()),
                ]),
            )],
            otherwise: Some(Box::new(NumericExpression::Multiplication(vec![
                NumericExpression::Variable(x),
                NumericExpression::Power {
                    base: Box::new(NumericExpression::Constant(Number::integer(2))),
                    exponent: Box::new(NumericExpression::Constant(Number::integer(-1))),
                }
            ])))
        }
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(ast_to_semantic(&parsed_expression), expected_expression);
    assert!(lexer.next_token().is_none());
}

#[test]
fn test_nested_conditional_parsing() {
    clear_symbol_table();
    let a = Variable::new("a", VariableType::Integer);
    let b = Variable::new("b", VariableType::Integer);
    let c = Variable::new("c", VariableType::Integer);
    let d = Variable::new("d", VariableType::Integer);
    let e = Variable::new("e", VariableType::Integer);
    let f = Variable::new("f", VariableType::Integer);
    let g = Variable::new("g", VariableType::Integer);

    let input = "a > b ? c < d ? e : f : g";
    let expected_expression = SemanticExpression::Numeric(
        NumericExpression::Piecewise {
            cases: vec![
                (LogicalExpression::And(vec![
                    LogicalExpression::Relation {
                        left: Box::new(NumericExpression::Variable(a.clone())),
                        operator: Symbol::Relation(Relation::GreaterThan),
                        right: Box::new(NumericExpression::Variable(b.clone())),
                    },
                    LogicalExpression::Relation {
                        left: Box::new(NumericExpression::Variable(c.clone())),
                        operator: Symbol::Relation(Relation::LessThan),
                        right: Box::new(NumericExpression::Variable(d.clone())),
                    },
                ]), NumericExpression::Variable(e.clone())),
                (LogicalExpression::Relation {
                    left: Box::new(NumericExpression::Variable(a)),
                    operator: Symbol::Relation(Relation::GreaterThan),
                    right: Box::new(NumericExpression::Variable(b)),
                }, NumericExpression::Variable(f.clone())),
            ],
            otherwise: Some(Box::new(NumericExpression::Variable(g))),
        }
    );

    let mut lexer = Lexer::new(input);
    let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
    assert_eq!(ast_to_semantic(&parsed_expression), expected_expression);
    assert!(lexer.next_token().is_none());
}