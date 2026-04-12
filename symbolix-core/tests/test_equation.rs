use symbolix_core::{
    equation::{BranchResult, Equation, SolveError},
    lexer::{
        constant::Number,
        symbol::{Relation, Symbol},
    },
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
        variable::{Variable, VariableType},
    },
};

fn numeric_var(name: &str) -> Variable {
    Variable {
        name: name.to_string(),
        var_type: VariableType::Float,
        value: None,
    }
}

#[test]
fn solves_linear_equation_with_other_variables_as_parameters() {
    let x = numeric_var("x");
    let y = numeric_var("y");
    let raw = SemanticExpression::numeric(
        NumericExpression::constant(Number::integer(2)) * NumericExpression::variable(x.clone())
            + NumericExpression::variable(y.clone())
            - NumericExpression::constant(Number::integer(4)),
    );

    let result = Equation::new(raw, x).unwrap().solve().unwrap();
    assert_eq!(result.branches.len(), 1);
    let BranchResult::Finite(solutions) = &result.branches[0].result else {
        panic!("expected finite solutions");
    };
    assert_eq!(solutions.len(), 1);
    let rendered = format!("{}", solutions[0]);
    assert!(rendered.contains('y'));
    assert!(!rendered.contains('x'));
}

#[test]
fn infer_rejects_ambiguous_target() {
    let x = numeric_var("x");
    let y = numeric_var("y");
    let raw = SemanticExpression::logical(LogicalExpression::relation(
        &NumericExpression::variable(x),
        &Symbol::Relation(Relation::Equal),
        &(NumericExpression::variable(y) + NumericExpression::constant(Number::integer(1))),
    ));

    let error = Equation::infer(raw).unwrap_err();
    assert!(matches!(error, SolveError::AmbiguousSolveTarget(_)));
}

#[test]
fn solves_piecewise_equation_into_multiple_branches() {
    let x = numeric_var("x");
    let y = numeric_var("y");
    let expr = NumericExpression::Piecewise {
        cases: vec![(
            LogicalExpression::relation(
                &NumericExpression::variable(y.clone()),
                &Symbol::Relation(Relation::GreaterThan),
                &NumericExpression::constant(Number::integer(0)),
            ),
            NumericExpression::variable(x.clone()) - NumericExpression::constant(Number::integer(1)),
        )],
        otherwise: Some(Box::new(
            NumericExpression::variable(x.clone()) + NumericExpression::constant(Number::integer(1)),
        )),
    };

    let result = Equation::new(SemanticExpression::numeric(expr), x)
        .unwrap()
        .solve()
        .unwrap();

    assert_eq!(result.branches.len(), 2);
    assert!(result.branches.iter().all(
        |branch| matches!(branch.result, BranchResult::Finite(ref solutions) if solutions.len() == 1)
    ));
}

#[test]
fn solves_quadratic_equation_with_two_solutions() {
    let x = numeric_var("x");
    let raw = SemanticExpression::numeric(
        NumericExpression::power(
            &NumericExpression::variable(x.clone()),
            &NumericExpression::constant(Number::integer(2)),
        ) - NumericExpression::constant(Number::integer(1)),
    );

    let result = Equation::new(raw, x).unwrap().solve().unwrap();
    assert_eq!(result.branches.len(), 1);
    let BranchResult::Finite(solutions) = &result.branches[0].result else {
        panic!("expected finite solutions");
    };
    assert_eq!(solutions.len(), 2);
}

#[test]
fn solves_identity_equation_as_identity_branch() {
    let x = numeric_var("x");
    let raw =
        SemanticExpression::numeric(NumericExpression::variable(x.clone()) - NumericExpression::variable(x));

    let result = Equation::infer(raw).unwrap().solve().unwrap();
    assert_eq!(result.branches.len(), 1);
    assert!(matches!(result.branches[0].result, BranchResult::Identity));
}

#[test]
fn carries_non_zero_domain_constraint_for_parameter_denominator() {
    let x = numeric_var("x");
    let a = numeric_var("a");
    let raw = SemanticExpression::logical(LogicalExpression::relation(
        &(NumericExpression::variable(x.clone()) / NumericExpression::variable(a.clone())),
        &Symbol::Relation(Relation::Equal),
        &NumericExpression::constant(Number::integer(1)),
    ));

    let result = Equation::new(raw, x).unwrap().solve().unwrap();
    assert_eq!(result.branches.len(), 1);
    let constraint = format!("{}", result.branches[0].constraint);
    assert!(constraint.contains("!="));
    assert!(constraint.contains('a'));
}
