use symbolix_core::{
    equation::{BranchResult, Equation, SolutionBranch, SolutionSet, SolveError},
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

fn flatten_finite_solutions(set: &SolutionSet) -> Vec<String> {
    let mut values = set
        .branches
        .iter()
        .flat_map(|branch| match &branch.result {
            BranchResult::Finite(solutions) => solutions
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            BranchResult::Identity => Vec::new(),
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
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
            NumericExpression::variable(x.clone())
                - NumericExpression::constant(Number::integer(1)),
        )],
        otherwise: Some(Box::new(
            NumericExpression::variable(x.clone())
                + NumericExpression::constant(Number::integer(1)),
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
    let raw = SemanticExpression::numeric(
        NumericExpression::variable(x.clone()) - NumericExpression::variable(x),
    );

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

#[test]
fn merges_solutions_with_same_constraint() {
    let x = numeric_var("x");
    let constraint = LogicalExpression::relation(
        &NumericExpression::variable(numeric_var("a")),
        &Symbol::Relation(Relation::GreaterThan),
        &NumericExpression::constant(Number::integer(0)),
    );

    let set = SolutionSet::new(
        x,
        vec![
            SolutionBranch::finite(
                constraint.clone(),
                vec![NumericExpression::constant(Number::integer(0))],
            ),
            SolutionBranch::finite(
                constraint,
                vec![NumericExpression::constant(Number::integer(2))],
            ),
        ],
    )
    .merge_by_constraint();

    assert_eq!(set.branches.len(), 1);
    let BranchResult::Finite(solutions) = &set.branches[0].result else {
        panic!("expected finite solutions");
    };
    assert_eq!(solutions.len(), 2);
}

#[test]
fn solves_single_variable_linear_equation() {
    let x = numeric_var("x");
    let raw = SemanticExpression::logical(LogicalExpression::relation(
        &(NumericExpression::constant(Number::integer(2)) * NumericExpression::variable(x.clone())
            + NumericExpression::constant(Number::integer(3))),
        &Symbol::Relation(Relation::Equal),
        &NumericExpression::constant(Number::integer(7)),
    ));

    let result = Equation::new(raw, x).unwrap().solve().unwrap();
    assert_eq!(result.branches.len(), 1);
    let BranchResult::Finite(solutions) = &result.branches[0].result else {
        panic!("expected finite solutions");
    };
    assert_eq!(solutions.len(), 1);
    assert_eq!(solutions[0].to_string(), "2");
}

#[test]
fn solves_piecewise_single_variable_linear_equation() {
    let x = numeric_var("x");
    let expr = NumericExpression::Piecewise {
        cases: vec![(
            LogicalExpression::relation(
                &NumericExpression::variable(x.clone()),
                &Symbol::Relation(Relation::LessThan),
                &NumericExpression::constant(Number::integer(0)),
            ),
            NumericExpression::constant(Number::integer(2))
                * NumericExpression::variable(x.clone())
                + NumericExpression::constant(Number::integer(4)),
        )],
        otherwise: Some(Box::new(
            NumericExpression::constant(Number::integer(2))
                * NumericExpression::variable(x.clone())
                - NumericExpression::constant(Number::integer(4)),
        )),
    };
    let raw = SemanticExpression::logical(LogicalExpression::relation(
        &expr,
        &Symbol::Relation(Relation::Equal),
        &NumericExpression::constant(Number::integer(0)),
    ));

    let result = Equation::new(raw, x).unwrap().solve().unwrap();
    let rendered = flatten_finite_solutions(&result);
    assert_eq!(rendered, vec!["-2".to_string(), "2".to_string()]);
}

#[test]
fn solves_single_variable_quadratic_equation() {
    let x = numeric_var("x");
    let raw = SemanticExpression::logical(LogicalExpression::relation(
        &(NumericExpression::power(
            &NumericExpression::variable(x.clone()),
            &NumericExpression::constant(Number::integer(2)),
        ) - NumericExpression::constant(Number::integer(5))
            * NumericExpression::variable(x.clone())
            + NumericExpression::constant(Number::integer(6))),
        &Symbol::Relation(Relation::Equal),
        &NumericExpression::constant(Number::integer(0)),
    ));

    let result = Equation::new(raw, x).unwrap().solve().unwrap();
    assert_eq!(result.branches.len(), 1);
    let BranchResult::Finite(solutions) = &result.branches[0].result else {
        panic!("expected finite solutions");
    };
    assert_eq!(solutions.len(), 2);
    let rendered = solutions
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert!(rendered.contains(&"2".to_string()));
    assert!(rendered.contains(&"3".to_string()));
}

#[test]
fn solves_piecewise_single_variable_quadratic_equation() {
    let x = numeric_var("x");
    let expr = NumericExpression::Piecewise {
        cases: vec![(
            LogicalExpression::relation(
                &NumericExpression::variable(x.clone()),
                &Symbol::Relation(Relation::LessThan),
                &NumericExpression::constant(Number::integer(0)),
            ),
            NumericExpression::power(
                &NumericExpression::variable(x.clone()),
                &NumericExpression::constant(Number::integer(2)),
            ) - NumericExpression::constant(Number::integer(1)),
        )],
        otherwise: Some(Box::new(
            NumericExpression::power(
                &NumericExpression::variable(x.clone()),
                &NumericExpression::constant(Number::integer(2)),
            ) - NumericExpression::constant(Number::integer(4)),
        )),
    };
    let raw = SemanticExpression::logical(LogicalExpression::relation(
        &expr,
        &Symbol::Relation(Relation::Equal),
        &NumericExpression::constant(Number::integer(0)),
    ));

    let result = Equation::new(raw, x).unwrap().solve().unwrap();
    let rendered = flatten_finite_solutions(&result);
    assert_eq!(rendered, vec!["-1".to_string(), "2".to_string()]);
}
