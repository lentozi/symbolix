use symbolix_core::{
    equation::SolveError,
    error::{ErrorExt, ErrorKind},
    semantic::variable::{Variable, VariableType},
};

fn variable(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Float,
        value: None,
    }
}

#[test]
fn error_ids_are_monotonic_and_accessors_match() {
    let first = ErrorExt::new(ErrorKind::Io, "io".to_string(), false);
    let second = ErrorExt::semantic_error("semantic", true);

    assert!(second.error_id() > first.error_id());
    assert!(matches!(first.error_kind(), ErrorKind::Io));
    assert_eq!(first.error_message(), "io");
    assert!(!first.is_fatal());
    assert!(matches!(second.error_kind(), ErrorKind::Semantic));
    assert!(second.is_fatal());
}

#[test]
fn lexical_error_constructor_sets_expected_fields() {
    let error = ErrorExt::lexical_error("bad token", false);
    assert!(matches!(error.error_kind(), ErrorKind::Lexical));
    assert_eq!(error.error_message(), "bad token");
    assert!(!error.is_fatal());
}

#[test]
fn solve_error_display_messages_cover_all_variants() {
    let cases = vec![
        (
            SolveError::UnsupportedEquationFormat,
            "unsupported equation format".to_string(),
        ),
        (
            SolveError::NoVariableToSolve,
            "equation does not contain a variable".to_string(),
        ),
        (
            SolveError::AmbiguousSolveTarget(vec![variable("x"), variable("y")]),
            "solve target is ambiguous: x, y".to_string(),
        ),
        (
            SolveError::UnsupportedSolver("custom".to_string()),
            "custom".to_string(),
        ),
        (
            SolveError::NonLinearExpression,
            "equation is not linear in the target".to_string(),
        ),
        (
            SolveError::NonPolynomialExpression,
            "equation is not a supported polynomial in the target".to_string(),
        ),
        (
            SolveError::ExpectedUniqueSolutionSet,
            "expected a unique solution".to_string(),
        ),
        (
            SolveError::ExpectedUnconditionalSolution,
            "expected an unconditional solution".to_string(),
        ),
        (
            SolveError::UnsupportedSolutionSetExpression,
            "solution set cannot be represented as a numeric expression".to_string(),
        ),
        (
            SolveError::VerificationFailed,
            "failed to verify solver output".to_string(),
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
