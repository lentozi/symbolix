use symbolix_core::{
    lexer::Lexer,
    new_compile_context,
    parser::Parser,
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
        variable::VariableType,
        Analyzer,
    },
    with_compile_context,
};

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
