use exprion_api::{scope, Var};

#[test]
fn api_normalizes_variable_and_expression_arithmetic() {
    scope(|| {
        let x = Var::number("x");
        let y = Var::number("y");

        let expr = &x + &y * 2.0 - 1.0;
        let rendered = expr.semantic().to_string();

        assert!(rendered.contains("x"));
        assert!(rendered.contains("y"));
        assert!(rendered.contains("2"));
        assert!(rendered.contains("1"));
    });
}

#[test]
fn api_supports_left_scalar_arithmetic_with_var() {
    scope(|| {
        let x = Var::number("x");

        let expr = 2.0 * &x + 1.0;
        let rendered = expr.semantic().to_string();

        assert!(expr.semantic().is_numeric());
        assert!(rendered.contains("x"));
        assert!(rendered.contains("2"));
    });
}

#[test]
fn api_supports_relations_boolean_ops_and_pow() {
    scope(|| {
        let x = Var::number("x");
        let y = Var::number("y");

        let relation = x.gt(1.0) & y.lt(10.0);
        assert!(relation.semantic().is_logical());

        let power = (&x + 1.0).pow(&y);
        assert!(power.semantic().is_numeric());
    });
}

#[test]
fn api_solves_equation_with_inferred_target() {
    scope(|| {
        let x = Var::number("x");
        let equation = (&x + 2.0).eq_expr(6.0);

        let solved = equation.solve_unique().unwrap();

        assert_eq!(solved.semantic().to_string(), "4");
    });
}

#[test]
fn api_solves_equation_with_explicit_target() {
    scope(|| {
        let x = Var::number("x");
        let y = Var::number("y");
        let equation = (&x + &y).eq_expr(10.0);

        let solved = equation.solve_for(&x).unwrap().into_expr().unwrap();
        let rendered = solved.semantic().to_string();

        assert!(rendered.contains("10"));
        assert!(rendered.contains("y"));
    });
}

#[test]
fn api_jit_compiles_numeric_expression() {
    scope(|| {
        let x = Var::number("x");
        let z = Var::number("z");
        let expr = &z + &x * 2.0 + 1.0;

        let compiled = expr.jit_compile().unwrap();
        let result = compiled.calculate_named(&[("z", 10.0), ("x", 3.0)]).unwrap();

        assert_eq!(compiled.variables(), vec!["x".to_string(), "z".to_string()]);
        assert!((result - 17.0).abs() < 1e-9);
    });
}

#[test]
fn nested_scope_reuses_compile_context_and_enters_new_var_scope() {
    scope(|| {
        let x = Var::number("x");
        let outer_name_id = x.raw().name_id;

        scope(|| {
            let x_inner = Var::number("x");
            let y = Var::number("y");
            let expr = &x_inner + &y;
            let rendered = expr.semantic().to_string();

            assert_eq!(x_inner.raw().name_id, outer_name_id);
            assert!(rendered.contains("x"));
            assert!(rendered.contains("y"));
        });

        let z = Var::number("z");
        let rendered = (&x + &z).semantic().to_string();

        assert!(rendered.contains("x"));
        assert!(rendered.contains("z"));
    });
}
