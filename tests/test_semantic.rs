use symbolix::lexer::constant::Number;
use symbolix::lexer::symbol::{Relation, Symbol};
use symbolix::lexer::Lexer;
use symbolix::semantic::ast_to_semantic;
use symbolix::semantic::semantic_ir::{LogicalExpression, NumericExpression, SemanticExpression};
use symbolix::semantic::variable::VariableType;
use symbolix::{context, logical_bucket, numeric_bucket, var};

#[test]
fn test_unary_semantic() {
    context! {
        let x = var!("x", VariableType::Integer, None);
        let y = var!("y", VariableType::Integer, None);
        let input = "-x + y";
        let expected_expression = SemanticExpression::Numeric(
            NumericExpression::Addition(numeric_bucket![
                NumericExpression::Negation(Box::new(NumericExpression::Variable(x))),
                NumericExpression::Variable(y),
            ])
        );

        let mut lexer = Lexer::new(input);
        let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
        assert_eq!(ast_to_semantic(&parsed_expression), expected_expression);
        assert!(lexer.next_token().is_none());
    }

}

#[test]
fn test_logic_semantic() {
    context! {
        let _a = var!("a_bool", VariableType::Boolean, None);
        let _b = var!("b_bool", VariableType::Boolean, None);
        let input = "!a_bool && b_bool || true";
        let expected_expression = SemanticExpression::Logical(LogicalExpression::Constant(true));

        let mut lexer = Lexer::new(input);
        let parsed_expression = symbolix::parser::pratt_parsing(&mut lexer, symbolix::lexer::symbol::Precedence::Lowest);
        assert_eq!(ast_to_semantic(&parsed_expression), expected_expression);
        assert!(lexer.next_token().is_none());
    }
}

#[test]
fn test_common_semantic() {
    context! {
        let a = var!("a", VariableType::Integer, None);
        let b = var!("b", VariableType::Integer, None);
        let c = var!("c", VariableType::Integer, None);
        let d = var!("d", VariableType::Integer, None);
        let e = var!("e", VariableType::Integer, None);

        let input = "a + b * c - d / e";
        let expected_expression = SemanticExpression::Numeric(
            NumericExpression::Addition(numeric_bucket![
                NumericExpression::Variable(a),
                NumericExpression::Multiplication(numeric_bucket![
                    NumericExpression::Variable(b),
                    NumericExpression::Variable(c),
                ]),
                NumericExpression::Multiplication(numeric_bucket![
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
}

#[test]
fn test_conditional_parsing() {
    context! {
        let x = var!("x", VariableType::Integer, None);
        let input = "x > 100 ? x * (2 + 3) : x / 2";
        let expected_expression = SemanticExpression::Numeric(
            NumericExpression::Piecewise {
                cases: vec![(
                    LogicalExpression::Relation {
                        left: Box::new(NumericExpression::Variable(x.clone())),
                        operator: Symbol::Relation(Relation::GreaterThan),
                        right: Box::new(NumericExpression::Constant(Number::integer(100))),
                    },
                    NumericExpression::Multiplication(numeric_bucket![
                        NumericExpression::Constant(Number::integer(5)),
                        NumericExpression::Variable(x.clone()),
                    ]),
                )],
                otherwise: Some(Box::new(NumericExpression::Multiplication(numeric_bucket![
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
}

#[test]
fn test_nested_conditional_parsing() {
    context! {
        let a = var!("a", VariableType::Integer, None);
        let b = var!("b", VariableType::Integer, None);
        let c = var!("c", VariableType::Integer, None);
        let d = var!("d", VariableType::Integer, None);
        let e = var!("e", VariableType::Integer, None);
        let f = var!("f", VariableType::Integer, None);
        let g = var!("g", VariableType::Integer, None);

        let input = "a > b ? c < d ? e : f : g";
        let expected_expression = SemanticExpression::Numeric(
            NumericExpression::Piecewise {
                cases: vec![
                    (LogicalExpression::And(logical_bucket![
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
}