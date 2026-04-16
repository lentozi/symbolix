use exprion_core::{
    lexer::{constant::Constant, symbol::{Binary, Relation, Symbol, Ternary, Unary}, Lexer},
    new_compile_context,
    parser::expression::Expression,
    parser::Parser,
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
        variable::VariableType,
        Analyzer,
    },
    with_compile_context,
};
use std::panic::{catch_unwind, AssertUnwindSafe};

#[test]
fn analyzer_tracks_numeric_then_logical_expressions() {
    new_compile_context! {
        let mut analyzer = Analyzer::new();

        let mut lexer = Lexer::new("x + 1");
        let numeric = analyzer.analyze_with_ctx(&Parser::pratt(&mut lexer));
        assert!(matches!(numeric, SemanticExpression::Numeric(_)));
        assert!(analyzer.is_numeric());

        let mut lexer = Lexer::new("x > 1");
        let logical = analyzer.analyze_with_ctx(&Parser::pratt(&mut lexer));
        assert!(matches!(logical, SemanticExpression::Logical(_)));
        assert!(!analyzer.is_numeric());

        let vars = with_compile_context!(ctx, ctx.collect_all_variables());
        assert!(vars.iter().any(|var| var.name == "x"));
        assert!(vars.iter().any(|var| var.var_type == VariableType::Float));
    }
}

#[test]
fn analyzer_reuses_registered_variables_from_context() {
    new_compile_context! {
        let _ = exprion_core::semantic::variable::Variable::new("shared", VariableType::Boolean, None);

        let mut analyzer = Analyzer::new();
        let expr = Expression::variable("shared".to_string());
        let semantic = analyzer.analyze_with_ctx(&expr);

        assert!(matches!(semantic, SemanticExpression::Numeric(NumericExpression::Variable(_))));

        let vars = with_compile_context!(ctx, ctx.collect_all_variables());
        let shared = vars.into_iter().find(|var| var.name == "shared").unwrap();
        assert_eq!(shared.var_type, VariableType::Boolean);
    }
}

#[test]
fn analyzer_builds_piecewise_semantic_expression_for_ternary() {
    new_compile_context! {
        let mut lexer = Lexer::new("x > 0 ? x : -x");
        let expr = Parser::pratt(&mut lexer);
        let mut analyzer = Analyzer::new();
        let semantic = analyzer.analyze_with_ctx(&expr);

        match semantic {
            SemanticExpression::Numeric(NumericExpression::Piecewise { cases, otherwise }) => {
                assert_eq!(cases.len(), 1);
                assert!(matches!(cases[0].0, LogicalExpression::Relation { .. }));
                assert!(otherwise.is_some());
            }
            other => panic!("expected piecewise numeric expression, got {other}"),
        }
    }
}

#[test]
fn analyzer_covers_manual_expression_shapes_and_boolean_context() {
    new_compile_context! {
        let mut analyzer = Analyzer::new();

        let plus = Expression::unary(
            Symbol::Unary(Unary::Plus),
            Expression::constant(Constant::integer(3)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&plus).to_string(), "3");

        let minus = Expression::unary(
            Symbol::Unary(Unary::Minus),
            Expression::constant(Constant::integer(3)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&minus).to_string(), "-3");

        let logic_not = Expression::unary(
            Symbol::Unary(Unary::LogicNot),
            Expression::constant(Constant::Boolean(true)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&logic_not).to_string(), "false");

        let relation = Expression::relation(
            Expression::variable("x".to_string()),
            Symbol::Relation(Relation::LessEqual),
            Expression::constant(Constant::integer(5)),
        );
        let relation_semantic = analyzer.analyze_with_ctx(&relation);
        assert!(matches!(relation_semantic, SemanticExpression::Logical(_)));

        let logic_expr = Expression::binary(
            Expression::constant(Constant::Boolean(true)),
            Symbol::Binary(Binary::LogicAnd),
            Expression::variable("flag".to_string()),
        );
        let logical = analyzer.analyze_with_ctx(&logic_expr);
        assert!(matches!(logical, SemanticExpression::Logical(_)));

        let add = Expression::binary(
            Expression::constant(Constant::integer(1)),
            Symbol::Binary(Binary::Add),
            Expression::constant(Constant::integer(2)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&add).to_string(), "3");

        let sub = Expression::binary(
            Expression::constant(Constant::integer(5)),
            Symbol::Binary(Binary::Subtract),
            Expression::constant(Constant::integer(2)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&sub).to_string(), "3");

        let mul = Expression::binary(
            Expression::constant(Constant::integer(2)),
            Symbol::Binary(Binary::Multiply),
            Expression::constant(Constant::integer(3)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&mul).to_string(), "6");

        let div = Expression::binary(
            Expression::constant(Constant::integer(6)),
            Symbol::Binary(Binary::Divide),
            Expression::constant(Constant::integer(2)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&div).to_string(), "3");

        let pow = Expression::binary(
            Expression::constant(Constant::integer(2)),
            Symbol::Binary(Binary::Power),
            Expression::constant(Constant::integer(3)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&pow).to_string(), "8");

        let logic_or = Expression::binary(
            Expression::constant(Constant::Boolean(false)),
            Symbol::Binary(Binary::LogicOr),
            Expression::constant(Constant::Boolean(true)),
        );
        assert_eq!(analyzer.analyze_with_ctx(&logic_or).to_string(), "true");

        let vars = with_compile_context!(ctx, ctx.collect_all_variables());
        assert!(vars.iter().any(|var| var.name == "flag" && var.var_type == VariableType::Boolean));
    }
}

#[test]
fn analyzer_panics_on_invalid_manual_expressions() {
    new_compile_context! {
        let mut analyzer = Analyzer::new();

        let invalid_ternary = Expression::ternary(
            Expression::constant(Constant::Boolean(true)),
            Symbol::Other(exprion_core::lexer::symbol::Other::Comma),
            Expression::constant(Constant::integer(1)),
            Symbol::Ternary(Ternary::ConditionalElse),
            Expression::constant(Constant::integer(2)),
        );
        assert!(catch_unwind(AssertUnwindSafe(|| analyzer.analyze_with_ctx(&invalid_ternary))).is_err());

        let invalid_relation = Expression::binary(
            Expression::constant(Constant::Boolean(true)),
            Symbol::Relation(Relation::Equal),
            Expression::constant(Constant::integer(1)),
        );
        assert!(catch_unwind(AssertUnwindSafe(|| analyzer.analyze_with_ctx(&invalid_relation))).is_err());

        let invalid_binary_symbol = Expression::binary(
            Expression::constant(Constant::integer(1)),
            Symbol::Other(exprion_core::lexer::symbol::Other::Comma),
            Expression::constant(Constant::integer(2)),
        );
        assert!(catch_unwind(AssertUnwindSafe(|| analyzer.analyze_with_ctx(&invalid_binary_symbol))).is_err());

        let invalid_unary_symbol = Expression::unary(
            Symbol::Other(exprion_core::lexer::symbol::Other::Semicolon),
            Expression::constant(Constant::integer(1)),
        );
        assert!(catch_unwind(AssertUnwindSafe(|| analyzer.analyze_with_ctx(&invalid_unary_symbol))).is_err());

        let invalid_minus = Expression::unary(
            Symbol::Unary(Unary::Minus),
            Expression::constant(Constant::Boolean(true)),
        );
        assert!(catch_unwind(AssertUnwindSafe(|| analyzer.analyze_with_ctx(&invalid_minus))).is_err());

        let invalid_not = Expression::unary(
            Symbol::Unary(Unary::LogicNot),
            Expression::constant(Constant::integer(1)),
        );
        assert!(catch_unwind(AssertUnwindSafe(|| analyzer.analyze_with_ctx(&invalid_not))).is_err());

        let invalid_ternary_shape = Expression::ternary(
            Expression::constant(Constant::integer(1)),
            Symbol::Ternary(Ternary::Conditional),
            Expression::constant(Constant::integer(1)),
            Symbol::Ternary(Ternary::ConditionalElse),
            Expression::constant(Constant::integer(2)),
        );
        assert!(catch_unwind(AssertUnwindSafe(|| analyzer.analyze_with_ctx(&invalid_ternary_shape))).is_err());
    }
}
