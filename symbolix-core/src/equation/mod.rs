//! 方程求解模块

mod error;
mod linear;
mod piecewise;
mod polynomial;

pub use error::SolveError;
pub use linear::{LinearForm, LinearSolver};
pub use polynomial::{PolynomialForm, PolynomialSolver};

use crate::{
    lexer::constant::Number,
    lexer::symbol::{Relation, Symbol},
    optimizer::optimize,
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
        variable::Variable,
    },
};
use piecewise::PiecewiseSolver;
use std::{fmt, panic::{catch_unwind, AssertUnwindSafe}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    pub expr: NumericExpression,
    pub solve_for: Variable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionSet {
    pub target: Variable,
    pub branches: Vec<SolutionBranch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionBranch {
    pub constraint: LogicalExpression,
    pub result: BranchResult,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchResult {
    Finite(Vec<NumericExpression>),
    Identity,
}

pub trait Solver {
    fn can_solve(eq: &Equation) -> bool;
    fn solve(eq: &Equation) -> Result<SolutionSet, SolveError>;
}

impl Equation {
    /// 从 SemanticExpression 构建方程对象
    pub fn new(raw: SemanticExpression, solve_for: Variable) -> Result<Self, SolveError> {
        let expr = match raw {
            SemanticExpression::Logical(LogicalExpression::Relation {
                left,
                operator: Symbol::Relation(Relation::Equal),
                right,
            }) => left.as_ref() - right.as_ref(),
            SemanticExpression::Numeric(expr) => expr,
            _ => return Err(SolveError::UnsupportedEquationFormat),
        };

        Ok(Self {
            expr: optimize_numeric(expr),
            solve_for,
        })
    }

    /// 从 SemanticExpression 中推断变量，仅支持单变量的 SemanticExpression
    pub fn infer(raw: SemanticExpression) -> Result<Self, SolveError> {
        let expr = match &raw {
            SemanticExpression::Logical(LogicalExpression::Relation {
                left,
                operator: Symbol::Relation(Relation::Equal),
                right,
            }) => left.as_ref() - right.as_ref(),
            SemanticExpression::Numeric(expr) => expr.clone(),
            _ => return Err(SolveError::UnsupportedEquationFormat),
        };

        let variables = collect_numeric_variables(&expr);
        match variables.as_slice() {
            [solve_for] => Self::new(raw, solve_for.clone()),
            [] => Err(SolveError::NoVariableToSolve),
            _ => Err(SolveError::AmbiguousSolveTarget(variables)),
        }
    }

    /// 求解方程，返回 SolutionSet
    pub fn solve(&self) -> Result<SolutionSet, SolveError> {
        let mut branches = Vec::new();
        for branch in PiecewiseSolver::expand(self)? {
            let (branch_constraint, branch_expr) = piecewise::split_branch_equation(&branch);
            let domain_constraint = collect_domain_constraint(&branch_expr);
            let solved = if LinearSolver::can_solve(&branch) {
                LinearSolver::solve(&branch)?
            } else if PolynomialSolver::can_solve(&branch) {
                PolynomialSolver::solve(&branch)?
            } else {
                return Err(SolveError::UnsupportedSolver(format!(
                    "no solver matched equation for target {}",
                    branch.solve_for
                )));
            };
            for mut solved_branch in solved.branches {
                solved_branch.constraint =
                    LogicalExpression::and(&solved_branch.constraint, &domain_constraint);
                solved_branch.constraint =
                    LogicalExpression::and(&solved_branch.constraint, &branch_constraint);
                solved_branch = verify_branch(&branch.solve_for, &branch_expr, solved_branch)?;
                if !matches!(solved_branch.result, BranchResult::Finite(ref values) if values.is_empty())
                {
                    branches.push(solved_branch);
                }
            }
        }

        Ok(SolutionSet::new(self.solve_for.clone(), branches))
    }

    /// 求解方程，返回唯一解的 SemanticExpression，要求方程有且仅有一个解
    pub fn solve_unique(&self) -> Result<SemanticExpression, SolveError> {
        let solution_set = self.solve()?;
        if solution_set.branches.len() != 1 {
            return Err(SolveError::ExpectedUniqueSolutionSet);
        }

        let branch = &solution_set.branches[0];
        if branch.constraint != LogicalExpression::constant(true) {
            return Err(SolveError::ExpectedUnconditionalSolution);
        }
        let BranchResult::Finite(solutions) = &branch.result else {
            return Err(SolveError::ExpectedUniqueSolutionSet);
        };
        if solutions.len() != 1 {
            return Err(SolveError::ExpectedUniqueSolutionSet);
        }

        Ok(SemanticExpression::numeric(solutions[0].clone()))
    }
}

impl SolutionSet {
    pub fn new(target: Variable, branches: Vec<SolutionBranch>) -> Self {
        Self { target, branches }
    }

    pub fn is_empty(&self) -> bool {
        self.branches.is_empty()
    }

    pub fn has_identity_branch(&self) -> bool {
        self.branches
            .iter()
            .any(|branch| matches!(branch.result, BranchResult::Identity))
    }

    pub fn into_numeric_expression(self) -> Result<NumericExpression, SolveError> {
        if self.branches.is_empty() {
            return Ok(NumericExpression::piecewise(
                Vec::new(),
                None,
            ));
        }

        if self.branches.len() == 1
            && self.branches[0].constraint == LogicalExpression::constant(true)
        {
            return match self.branches.into_iter().next().unwrap().result {
                BranchResult::Finite(solutions) if solutions.len() == 1 => {
                    Ok(solutions.into_iter().next().unwrap())
                }
                _ => Err(SolveError::UnsupportedSolutionSetExpression),
            };
        }

        let mut cases = Vec::new();
        for branch in self.branches {
            match branch.result {
                BranchResult::Finite(mut solutions) if solutions.len() == 1 => {
                    cases.push((branch.constraint, solutions.remove(0)));
                }
                BranchResult::Finite(solutions) if solutions.is_empty() => {}
                _ => return Err(SolveError::UnsupportedSolutionSetExpression),
            }
        }

        Ok(NumericExpression::piecewise(
            cases,
            None,
        ))
    }
}

impl SolutionBranch {
    pub fn finite(constraint: LogicalExpression, solutions: Vec<NumericExpression>) -> Self {
        Self {
            constraint,
            result: BranchResult::Finite(dedup_solutions(solutions)),
        }
    }

    pub fn identity(constraint: LogicalExpression) -> Self {
        Self {
            constraint,
            result: BranchResult::Identity,
        }
    }
}

impl fmt::Display for SolutionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.branches.is_empty() {
            return write!(f, "{}: no solutions", self.target);
        }

        let parts = self
            .branches
            .iter()
            .map(|branch| format!("{} = {}", self.target, branch))
            .collect::<Vec<_>>()
            .join("; ");
        write!(f, "{parts}")
    }
}

impl fmt::Display for SolutionBranch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.result {
            BranchResult::Identity => {
                if self.constraint == LogicalExpression::constant(true) {
                    write!(f, "all values")
                } else {
                    write!(f, "all values when {}", self.constraint)
                }
            }
            BranchResult::Finite(solutions) => {
                let solution_text = if solutions.is_empty() {
                    "no solutions".to_string()
                } else if solutions.len() == 1 {
                    solutions[0].to_string()
                } else {
                    format!(
                        "{{{}}}",
                        solutions
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };

                if self.constraint == LogicalExpression::constant(true) {
                    write!(f, "{solution_text}")
                } else {
                    write!(f, "{solution_text}, when {}", self.constraint)
                }
            }
        }
    }
}

/// 优化 NumericExpression，返回优化后的 NumericExpression
pub(crate) fn optimize_numeric(expr: NumericExpression) -> NumericExpression {
    let fallback = expr.clone();
    match catch_unwind(AssertUnwindSafe(|| {
        let mut wrapped = SemanticExpression::numeric(expr);
        optimize(&mut wrapped);
        match wrapped {
            SemanticExpression::Numeric(expr) => expr,
            _ => unreachable!(),
        }
    })) {
        Ok(expr) => expr,
        Err(_) => simplify_numeric(fallback),
    }
}

/// 判断 NumericExpression 是否为零
pub(crate) fn is_zero(expr: &NumericExpression) -> bool {
    matches!(
        optimize_numeric(expr.clone()),
        NumericExpression::Constant(number) if number.is_zero()
    )
}

/// 判断 NumericExpression 是否包含目标变量
pub(crate) fn contains_target(expr: &NumericExpression, target: &Variable) -> bool {
    match expr {
        NumericExpression::Constant(_) => false,
        NumericExpression::Variable(variable) => variable == target,
        NumericExpression::Negation(inner) => contains_target(inner, target),
        NumericExpression::Addition(bucket) | NumericExpression::Multiplication(bucket) => {
            bucket.iter().any(|expr| contains_target(&expr, target))
        }
        NumericExpression::Power { base, exponent } => {
            contains_target(base, target) || contains_target(exponent, target)
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            cases.iter().any(|(_, expr)| contains_target(expr, target))
                || otherwise
                    .as_ref()
                    .is_some_and(|expr| contains_target(expr, target))
        }
    }
}

/// 从 NumericExpression 中收集变量
fn collect_numeric_variables(expr: &NumericExpression) -> Vec<Variable> {
    let mut variables = Vec::new();
    collect_numeric_variables_inner(expr, &mut variables);
    variables
}

fn collect_numeric_variables_inner(expr: &NumericExpression, variables: &mut Vec<Variable>) {
    match expr {
        NumericExpression::Constant(_) => {}
        NumericExpression::Variable(variable) => {
            if !variables.contains(variable) {
                variables.push(variable.clone());
            }
        }
        NumericExpression::Negation(inner) => collect_numeric_variables_inner(inner, variables),
        NumericExpression::Addition(bucket) | NumericExpression::Multiplication(bucket) => {
            for expr in bucket.iter() {
                collect_numeric_variables_inner(&expr, variables);
            }
        }
        NumericExpression::Power { base, exponent } => {
            collect_numeric_variables_inner(base, variables);
            collect_numeric_variables_inner(exponent, variables);
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            for (condition, expr) in cases {
                collect_logical_variables(condition, variables);
                collect_numeric_variables_inner(expr, variables);
            }
            if let Some(expr) = otherwise {
                collect_numeric_variables_inner(expr, variables);
            }
        }
    }
}

/// 从 LogicalExpression 中收集变量
fn collect_logical_variables(expr: &LogicalExpression, variables: &mut Vec<Variable>) {
    match expr {
        LogicalExpression::Constant(_) => {}
        LogicalExpression::Variable(variable) => {
            if !variables.contains(variable) {
                variables.push(variable.clone());
            }
        }
        LogicalExpression::Not(inner) => collect_logical_variables(inner, variables),
        LogicalExpression::And(bucket) | LogicalExpression::Or(bucket) => {
            for expr in bucket.iter() {
                collect_logical_variables(&expr, variables);
            }
        }
        LogicalExpression::Relation { left, right, .. } => {
            collect_numeric_variables_inner(left, variables);
            collect_numeric_variables_inner(right, variables);
        }
    }
}

fn verify_branch(
    target: &Variable,
    branch_expr: &NumericExpression,
    branch: SolutionBranch,
) -> Result<SolutionBranch, SolveError> {
    match branch.result {
        BranchResult::Identity => {
            if constraint_allows_identity(&branch.constraint) {
                Ok(SolutionBranch::identity(branch.constraint))
            } else {
                Ok(SolutionBranch::finite(branch.constraint, Vec::new()))
            }
        }
        BranchResult::Finite(solutions) => {
            let verified = solutions
                .into_iter()
                .filter(|solution| verify_solution(target, branch_expr, &branch.constraint, solution))
                .collect::<Vec<_>>();
            Ok(SolutionBranch::finite(branch.constraint, verified))
        }
    }
}

fn verify_solution(
    target: &Variable,
    expr: &NumericExpression,
    constraint: &LogicalExpression,
    solution: &NumericExpression,
) -> bool {
    let substituted = substitute_numeric(expr, target, solution);
    let equation_holds = match catch_unwind(AssertUnwindSafe(|| optimize_numeric(substituted))) {
        Ok(NumericExpression::Constant(number)) => {
            number.is_zero() || number.to_float().abs() < 1e-9
        }
        Ok(_) => true,
        Err(_) => true,
    };

    equation_holds && logical_is_not_false(&substitute_logical(constraint, target, solution))
}

pub(crate) fn substitute_numeric(
    expr: &NumericExpression,
    target: &Variable,
    replacement: &NumericExpression,
) -> NumericExpression {
    match expr {
        NumericExpression::Constant(_) => expr.clone(),
        NumericExpression::Variable(variable) => {
            if variable == target {
                replacement.clone()
            } else {
                expr.clone()
            }
        }
        NumericExpression::Negation(inner) => optimize_numeric(-substitute_numeric(inner, target, replacement)),
        NumericExpression::Addition(bucket) => {
            let terms = bucket
                .iter()
                .map(|term| substitute_numeric(&term, target, replacement))
                .collect::<Vec<_>>();
            sum_numeric_terms(terms)
        }
        NumericExpression::Multiplication(bucket) => {
            let factors = bucket
                .iter()
                .map(|factor| substitute_numeric(&factor, target, replacement))
                .collect::<Vec<_>>();
            multiply_numeric_terms(factors)
        }
        NumericExpression::Power { base, exponent } => optimize_numeric(NumericExpression::power(
            &substitute_numeric(base, target, replacement),
            &substitute_numeric(exponent, target, replacement),
        )),
        NumericExpression::Piecewise { cases, otherwise } => NumericExpression::Piecewise {
            cases: cases
                .iter()
                .map(|(condition, expr)| {
                    (
                        substitute_logical(condition, target, replacement),
                        substitute_numeric(expr, target, replacement),
                    )
                })
                .collect(),
            otherwise: otherwise
                .as_ref()
                .map(|expr| Box::new(substitute_numeric(expr, target, replacement))),
        },
    }
}

fn sum_numeric_terms(terms: Vec<NumericExpression>) -> NumericExpression {
    let mut iter = terms.into_iter();
    match iter.next() {
        Some(first) => optimize_numeric(iter.fold(first, |acc, term| acc + term)),
        None => NumericExpression::constant(Number::integer(0)),
    }
}

fn multiply_numeric_terms(factors: Vec<NumericExpression>) -> NumericExpression {
    let mut iter = factors.into_iter();
    match iter.next() {
        Some(first) => optimize_numeric(iter.fold(first, |acc, factor| acc * factor)),
        None => NumericExpression::constant(Number::integer(1)),
    }
}


fn substitute_logical(
    expr: &LogicalExpression,
    target: &Variable,
    replacement: &NumericExpression,
) -> LogicalExpression {
    match expr {
        LogicalExpression::Constant(_) | LogicalExpression::Variable(_) => expr.clone(),
        LogicalExpression::Not(inner) => !substitute_logical(inner, target, replacement),
        LogicalExpression::And(bucket) => bucket
            .iter()
            .fold(LogicalExpression::constant(true), |acc, term| {
                LogicalExpression::and(&acc, &substitute_logical(&term, target, replacement))
            }),
        LogicalExpression::Or(bucket) => bucket
            .iter()
            .fold(LogicalExpression::constant(false), |acc, term| {
                LogicalExpression::or(&acc, &substitute_logical(&term, target, replacement))
            }),
        LogicalExpression::Relation {
            left,
            operator,
            right,
        } => LogicalExpression::relation(
            &substitute_numeric(left, target, replacement),
            operator,
            &substitute_numeric(right, target, replacement),
        ),
    }
}

fn collect_domain_constraint(expr: &NumericExpression) -> LogicalExpression {
    match expr {
        NumericExpression::Constant(_) | NumericExpression::Variable(_) => {
            LogicalExpression::constant(true)
        }
        NumericExpression::Negation(inner) => collect_domain_constraint(inner),
        NumericExpression::Addition(bucket) | NumericExpression::Multiplication(bucket) => bucket
            .iter()
            .fold(LogicalExpression::constant(true), |acc, term| {
                LogicalExpression::and(&acc, &collect_domain_constraint(&term))
            }),
        NumericExpression::Power { base, exponent } => {
            let base_constraint = collect_domain_constraint(base);
            let exponent_constraint = collect_domain_constraint(exponent);
            let mut constraint = LogicalExpression::and(&base_constraint, &exponent_constraint);
            if let NumericExpression::Constant(number) = exponent.as_ref() {
                if number.to_float() < 0.0 {
                    constraint = LogicalExpression::and(
                        &constraint,
                        &LogicalExpression::relation(
                            base,
                            &Symbol::Relation(Relation::NotEqual),
                            &NumericExpression::constant(Number::integer(0)),
                        ),
                    );
                }
            }
            constraint
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            let mut constraint = LogicalExpression::constant(true);
            for (condition, expr) in cases {
                let branch_constraint =
                    LogicalExpression::and(condition, &collect_domain_constraint(expr));
                constraint = LogicalExpression::and(&constraint, &branch_constraint);
            }
            if let Some(expr) = otherwise {
                constraint = LogicalExpression::and(&constraint, &collect_domain_constraint(expr));
            }
            constraint
        }
    }
}

fn constraint_allows_identity(constraint: &LogicalExpression) -> bool {
    logical_is_not_false(constraint)
}

fn logical_is_not_false(expr: &LogicalExpression) -> bool {
    !matches!(simplify_logical(expr.clone()), LogicalExpression::Constant(false))
}

fn simplify_logical(expr: LogicalExpression) -> LogicalExpression {
    match expr {
        LogicalExpression::Constant(_) | LogicalExpression::Variable(_) => expr,
        LogicalExpression::Not(inner) => match simplify_logical(*inner) {
            LogicalExpression::Constant(value) => LogicalExpression::constant(!value),
            simplified => LogicalExpression::Not(Box::new(simplified)),
        },
        LogicalExpression::And(bucket) => {
            let mut terms = Vec::new();
            for term in bucket.iter() {
                match simplify_logical(term) {
                    LogicalExpression::Constant(false) => return LogicalExpression::constant(false),
                    LogicalExpression::Constant(true) => {}
                    simplified => terms.push(simplified),
                }
            }
            match terms.len() {
                0 => LogicalExpression::constant(true),
                1 => terms.into_iter().next().unwrap(),
                _ => LogicalExpression::And(terms.into_iter().collect()),
            }
        }
        LogicalExpression::Or(bucket) => {
            let mut terms = Vec::new();
            for term in bucket.iter() {
                match simplify_logical(term) {
                    LogicalExpression::Constant(true) => return LogicalExpression::constant(true),
                    LogicalExpression::Constant(false) => {}
                    simplified => terms.push(simplified),
                }
            }
            match terms.len() {
                0 => LogicalExpression::constant(false),
                1 => terms.into_iter().next().unwrap(),
                _ => LogicalExpression::Or(terms.into_iter().collect()),
            }
        }
        LogicalExpression::Relation {
            left,
            operator,
            right,
        } => {
            let left = optimize_numeric(*left);
            let right = optimize_numeric(*right);
            match (&left, &right) {
                (NumericExpression::Constant(left), NumericExpression::Constant(right)) => {
                    LogicalExpression::constant(compare_numbers(left, &operator, right))
                }
                _ => LogicalExpression::relation(&left, &operator, &right),
            }
        }
    }
}

fn compare_numbers(left: &Number, operator: &Symbol, right: &Number) -> bool {
    let left_value = left.to_float();
    let right_value = right.to_float();
    match operator {
        Symbol::Relation(Relation::Equal) => (left_value - right_value).abs() < 1e-9,
        Symbol::Relation(Relation::NotEqual) => (left_value - right_value).abs() >= 1e-9,
        Symbol::Relation(Relation::LessThan) => left_value < right_value,
        Symbol::Relation(Relation::GreaterThan) => left_value > right_value,
        Symbol::Relation(Relation::LessEqual) => left_value <= right_value,
        Symbol::Relation(Relation::GreaterEqual) => left_value >= right_value,
        _ => false,
    }
}

fn dedup_solutions(solutions: Vec<NumericExpression>) -> Vec<NumericExpression> {
    let mut deduped = Vec::new();
    for solution in solutions {
        let normalized = optimize_numeric(solution);
        if !deduped.contains(&normalized) {
            deduped.push(normalized);
        }
    }
    deduped
}

fn simplify_numeric(expr: NumericExpression) -> NumericExpression {
    match expr {
        NumericExpression::Constant(_) | NumericExpression::Variable(_) => expr,
        NumericExpression::Negation(inner) => match simplify_numeric(*inner) {
            NumericExpression::Constant(number) => NumericExpression::constant(-number),
            simplified => NumericExpression::Negation(Box::new(simplified)),
        },
        NumericExpression::Addition(bucket) => {
            let mut constant = Number::integer(0);
            let mut terms = Vec::new();
            for term in bucket.iter() {
                match simplify_numeric(term) {
                    NumericExpression::Constant(number) => constant = constant + number,
                    simplified => terms.push(simplified),
                }
            }
            if !constant.is_zero() {
                terms.insert(0, NumericExpression::constant(constant));
            }
            match terms.len() {
                0 => NumericExpression::constant(Number::integer(0)),
                1 => terms.into_iter().next().unwrap(),
                _ => NumericExpression::Addition(terms.into_iter().collect()),
            }
        }
        NumericExpression::Multiplication(bucket) => {
            let mut constant = Number::integer(1);
            let mut factors = Vec::new();
            for factor in bucket.iter() {
                match simplify_numeric(factor) {
                    NumericExpression::Constant(number) => {
                        if number.is_zero() {
                            return NumericExpression::constant(Number::integer(0));
                        }
                        constant = constant * number;
                    }
                    simplified => factors.push(simplified),
                }
            }
            if !constant.is_one() {
                factors.insert(0, NumericExpression::constant(constant.clone()));
            }
            match factors.len() {
                0 => NumericExpression::constant(constant),
                1 if constant.is_one() => factors.into_iter().next().unwrap(),
                _ => NumericExpression::Multiplication(factors.into_iter().collect()),
            }
        }
        NumericExpression::Power { base, exponent } => {
            let base = simplify_numeric(*base);
            let exponent = simplify_numeric(*exponent);
            match (&base, &exponent) {
                (_, NumericExpression::Constant(number)) if number.is_zero() => {
                    NumericExpression::constant(Number::integer(1))
                }
                (_, NumericExpression::Constant(number)) if number.is_one() => base,
                (NumericExpression::Constant(left), NumericExpression::Constant(right)) => {
                    NumericExpression::constant(Number::float(left.to_float().powf(right.to_float())))
                }
                _ => NumericExpression::Power {
                    base: Box::new(base),
                    exponent: Box::new(exponent),
                },
            }
        }
        NumericExpression::Piecewise { cases, otherwise } => NumericExpression::Piecewise {
            cases: cases
                .into_iter()
                .map(|(condition, expr)| (condition, simplify_numeric(expr)))
                .collect(),
            otherwise: otherwise.map(|expr| Box::new(simplify_numeric(*expr))),
        },
    }
}

