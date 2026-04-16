use symbolix_core::{
    equation::{BranchResult, Equation, PolynomialForm, PolynomialSolver, SolutionBranch, SolutionSet, SolveError, Solver},
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

#[test]
fn solution_set_helpers_cover_unique_numeric_expression_and_display() {
    let x = numeric_var("x");
    let unique = Equation::new(
        SemanticExpression::logical(LogicalExpression::relation(
            &NumericExpression::variable(x.clone()),
            &Symbol::Relation(Relation::Equal),
            &NumericExpression::constant(Number::integer(2)),
        )),
        x.clone(),
    )
    .unwrap();

    let solved = unique.solve().unwrap();
    assert!(!solved.is_empty());
    assert!(!solved.has_identity_branch());
    assert_eq!(solved.clone().into_numeric_expression().unwrap().to_string(), "2");
    assert_eq!(unique.solve_unique().unwrap().to_string(), "2");
    assert!(format!("{}", solved).contains("x = 2"));
}

#[test]
fn solution_set_helpers_cover_identity_empty_and_error_paths() {
    let x = numeric_var("x");

    let empty = SolutionSet::new(x.clone(), vec![]);
    assert!(empty.is_empty());
    assert_eq!(empty.clone().into_numeric_expression().unwrap().to_string(), "");
    assert!(format!("{}", empty).contains("no solutions"));

    let identity = SolutionSet::new(x.clone(), vec![SolutionBranch::identity(LogicalExpression::constant(true))]);
    assert!(identity.has_identity_branch());
    assert_eq!(format!("{}", identity.branches[0]), "all values");

    let conditional_identity =
        SolutionBranch::identity(LogicalExpression::relation(
            &NumericExpression::variable(numeric_var("a")),
            &Symbol::Relation(Relation::GreaterThan),
            &NumericExpression::constant(Number::integer(0)),
        ));
    assert!(format!("{}", conditional_identity).contains("all values when"));

    let unsupported = SolutionSet::new(
        x.clone(),
        vec![SolutionBranch::finite(
            LogicalExpression::constant(true),
            vec![
                NumericExpression::constant(Number::integer(1)),
                NumericExpression::constant(Number::integer(2)),
            ],
        )],
    );
    assert!(matches!(
        unsupported.clone().into_numeric_expression(),
        Err(SolveError::UnsupportedSolutionSetExpression)
    ));
    assert!(matches!(
        SolutionSet::new(x.clone(), vec![conditional_identity.clone()]).into_numeric_expression(),
        Err(SolveError::UnsupportedSolutionSetExpression)
    ));
    assert!(matches!(
        Equation::new(
            SemanticExpression::logical(LogicalExpression::relation(
                &NumericExpression::power(
                    &NumericExpression::variable(x.clone()),
                    &NumericExpression::constant(Number::integer(2)),
                ),
                &Symbol::Relation(Relation::Equal),
                &NumericExpression::constant(Number::integer(1)),
            )),
            x.clone(),
        )
        .unwrap()
        .solve_unique(),
        Err(SolveError::ExpectedUniqueSolutionSet)
    ));

    let conditional = Equation::new(
        SemanticExpression::logical(LogicalExpression::relation(
            &(NumericExpression::variable(x.clone()) / NumericExpression::variable(numeric_var("a"))),
            &Symbol::Relation(Relation::Equal),
            &NumericExpression::constant(Number::integer(1)),
        )),
        x.clone(),
    )
    .unwrap();
    assert!(matches!(
        conditional.solve_unique(),
        Err(SolveError::ExpectedUnconditionalSolution)
    ));
}

#[test]
fn equation_infer_new_and_simplify_cover_remaining_paths() {
    let x = numeric_var("x");
    let unsupported = Equation::new(
        SemanticExpression::logical(LogicalExpression::constant(true)),
        x.clone(),
    )
    .unwrap_err();
    assert!(matches!(unsupported, SolveError::UnsupportedEquationFormat));

    let no_var = Equation::infer(SemanticExpression::numeric(NumericExpression::constant(
        Number::integer(0),
    )))
    .unwrap_err();
    assert!(matches!(no_var, SolveError::NoVariableToSolve));

    let simplified = SolutionSet::new(
        x,
        vec![
            SolutionBranch::finite(
                LogicalExpression::constant(true),
                vec![NumericExpression::constant(Number::integer(1))],
            ),
            SolutionBranch::finite(
                LogicalExpression::constant(false),
                vec![NumericExpression::constant(Number::integer(2))],
            ),
        ],
    )
    .simplify();
    assert_eq!(simplified.branches.len(), 1);
    assert_eq!(simplified.branches[0].to_string(), "1");
}

#[test]
fn solution_set_piecewise_conversion_and_display_cover_remaining_paths() {
    let x = numeric_var("x");
    let a = numeric_var("a");

    let piecewise = SolutionSet::new(
        x.clone(),
        vec![
            SolutionBranch::finite(
                LogicalExpression::relation(
                    &NumericExpression::variable(a.clone()),
                    &Symbol::Relation(Relation::GreaterThan),
                    &NumericExpression::constant(Number::integer(0)),
                ),
                vec![NumericExpression::constant(Number::integer(1))],
            ),
            SolutionBranch::finite(
                LogicalExpression::constant(true),
                vec![],
            ),
        ],
    );
    let expr = piecewise.clone().into_numeric_expression().unwrap();
    assert!(expr.to_string().contains("1"));
    assert!(format!("{}", piecewise).contains("when"));

    let no_solution_branch = SolutionBranch::finite(
        LogicalExpression::constant(true),
        vec![],
    );
    assert_eq!(format!("{}", no_solution_branch), "no solutions");

    let multi_solution_branch = SolutionBranch::finite(
        LogicalExpression::constant(true),
        vec![
            NumericExpression::constant(Number::integer(1)),
            NumericExpression::constant(Number::integer(2)),
        ],
    );
    assert!(format!("{}", multi_solution_branch).contains("{1, 2}"));
}

#[test]
fn equation_solve_reports_unsupported_solver_for_non_polynomial_non_linear_case() {
    let x = numeric_var("x");
    let raw = SemanticExpression::logical(LogicalExpression::relation(
        &NumericExpression::power(
            &NumericExpression::variable(x.clone()),
            &NumericExpression::variable(x.clone()),
        ),
        &Symbol::Relation(Relation::Equal),
        &NumericExpression::constant(Number::integer(1)),
    ));

    let error = Equation::new(raw, x).unwrap().solve().unwrap_err();
    assert!(matches!(error, SolveError::UnsupportedSolver(_)));
}

#[test]
fn polynomial_form_extract_degree_and_coefficient_cover_remaining_paths() {
    let x = numeric_var("x");
    let y = numeric_var("y");

    let quadratic = NumericExpression::power(
        &NumericExpression::variable(x.clone()),
        &NumericExpression::constant(Number::integer(2)),
    ) + (NumericExpression::constant(Number::integer(3)) * NumericExpression::variable(x.clone()))
        + NumericExpression::constant(Number::integer(2));
    let form = PolynomialForm::extract(&quadratic, &x, 2).unwrap();
    assert_eq!(form.degree(), Some(2));
    assert_eq!(form.coefficient(2).to_string(), "1");
    assert_eq!(form.coefficient(1).to_string(), "3");
    assert_eq!(form.coefficient(0).to_string(), "2");
    assert_eq!(form.coefficient(3).to_string(), "0");

    let negated = PolynomialForm::extract(
        &-(NumericExpression::variable(x.clone()) + NumericExpression::constant(Number::integer(1))),
        &x,
        2,
    )
    .unwrap();
    assert_eq!(negated.degree(), Some(1));

    let parameter_only = PolynomialForm::extract(&NumericExpression::variable(y.clone()), &x, 2).unwrap();
    assert_eq!(parameter_only.degree(), Some(0));

    let zero_power = PolynomialForm::extract(
        &NumericExpression::power(
            &NumericExpression::variable(x.clone()),
            &NumericExpression::constant(Number::integer(0)),
        ),
        &x,
        2,
    )
    .unwrap();
    assert_eq!(zero_power.degree(), Some(0));

    assert!(PolynomialForm::extract(
        &NumericExpression::power(
            &NumericExpression::variable(x.clone()),
            &NumericExpression::constant(Number::integer(-1)),
        ),
        &x,
        2,
    )
    .is_none());
    assert!(PolynomialForm::extract(
        &NumericExpression::power(
            &NumericExpression::variable(x.clone()),
            &NumericExpression::variable(y.clone()),
        ),
        &x,
        2,
    )
    .is_none());
    assert!(PolynomialForm::extract(
        &NumericExpression::Piecewise {
            cases: vec![(LogicalExpression::constant(true), NumericExpression::variable(x.clone()))],
            otherwise: None,
        },
        &x,
        2,
    )
    .is_none());
}

#[test]
fn polynomial_solver_can_solve_and_rejects_non_quadratic_cases() {
    let x = numeric_var("x");
    let quadratic_eq = Equation::new(
        SemanticExpression::logical(LogicalExpression::relation(
            &NumericExpression::power(
                &NumericExpression::variable(x.clone()),
                &NumericExpression::constant(Number::integer(2)),
            ),
            &Symbol::Relation(Relation::Equal),
            &NumericExpression::constant(Number::integer(1)),
        )),
        x.clone(),
    )
    .unwrap();
    assert!(PolynomialSolver::can_solve(&quadratic_eq));

    let linear_eq = Equation::new(
        SemanticExpression::logical(LogicalExpression::relation(
            &NumericExpression::variable(x.clone()),
            &Symbol::Relation(Relation::Equal),
            &NumericExpression::constant(Number::integer(1)),
        )),
        x.clone(),
    )
    .unwrap();
    assert!(!PolynomialSolver::can_solve(&linear_eq));

    let degenerate_quadratic = Equation::new(
        SemanticExpression::logical(LogicalExpression::relation(
            &(NumericExpression::constant(Number::integer(0))
                * NumericExpression::power(
                    &NumericExpression::variable(x.clone()),
                    &NumericExpression::constant(Number::integer(2)),
                )
                + NumericExpression::constant(Number::integer(1))),
            &Symbol::Relation(Relation::Equal),
            &NumericExpression::constant(Number::integer(0)),
        )),
        x,
    )
    .unwrap();
    assert!(matches!(
        PolynomialSolver::solve(&degenerate_quadratic),
        Err(SolveError::NonPolynomialExpression)
    ));
}
