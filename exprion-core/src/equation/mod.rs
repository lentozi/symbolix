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
    optimizer::{normalize_logic, normalize_numeric},
    semantic::{
        semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
        variable::Variable,
    },
};
use piecewise::PiecewiseSolver;
use std::{
    collections::{HashMap, HashSet},
    fmt,
    panic::{catch_unwind, AssertUnwindSafe},
};

/// 方程结构体
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equation {
    /// 方程左侧表达式，expr == 0
    pub expr: NumericExpression,
    /// 该方程的未知变量
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

/// 关系表达式的解类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchResult {
    /// 有穷解
    Finite(Vec<NumericExpression>),
    /// 全解，表示关系恒成立
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
            expr,
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
            let (_branch_constraint, branch_expr) = piecewise::split_branch_equation(&branch);
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
                solved_branch = verify_branch(&branch.solve_for, &branch_expr, solved_branch)?;
                if !matches!(solved_branch.result, BranchResult::Finite(ref values) if values.is_empty())
                {
                    branches.push(solved_branch);
                }
            }
        }

        Ok(SolutionSet::new(self.solve_for.clone(), branches)
            .substitute_into_constraints()
            .merge_by_constraint())
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
    /// 构造解集对象
    pub fn new(target: Variable, branches: Vec<SolutionBranch>) -> Self {
        Self { target, branches }
    }

    /// 判断解集是否为空
    pub fn is_empty(&self) -> bool {
        self.branches.is_empty()
    }

    /// 判断解集中是否有全解
    pub fn has_identity_branch(&self) -> bool {
        self.branches
            .iter()
            .any(|branch| matches!(branch.result, BranchResult::Identity))
    }

    /// 将解集转换为 NumericExpression ，要求单段函数内的解唯一
    pub fn into_numeric_expression(self) -> Result<NumericExpression, SolveError> {
        if self.branches.is_empty() {
            return Ok(NumericExpression::piecewise(Vec::new(), None));
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

        Ok(NumericExpression::piecewise(cases, None))
    }

    pub fn simplify(&self) -> Self {
        self.substitute_into_constraints().merge_by_constraint()
    }

    pub fn substitute_into_constraints(&self) -> Self {
        let mut branches = Vec::new();

        for branch in &self.branches {
            match &branch.result {
                BranchResult::Finite(solutions) => {
                    for solution in solutions {
                        let constraint =
                            {
                                let mut constraint = branch.constraint.substitute(
                                    &self.target,
                                    Some(&SemanticExpression::numeric(solution.clone())),
                                );
                                normalize_logic(&mut constraint);
                                constraint
                            };

                        if constraint == LogicalExpression::constant(false) {
                            continue;
                        }

                        branches.push(SolutionBranch::finite(constraint, vec![solution.clone()]));
                    }
                }
                BranchResult::Identity => {
                    let constraint = {
                        let mut constraint = branch.constraint.clone();
                        normalize_logic(&mut constraint);
                        constraint
                    };
                    if constraint != LogicalExpression::constant(false) {
                        branches.push(SolutionBranch::identity(constraint));
                    }
                }
            }
        }

        SolutionSet::new(self.target.clone(), branches)
    }

    pub fn merge_by_constraint(&self) -> Self {
        let mut finite_groups: HashMap<LogicalExpression, Vec<NumericExpression>> = HashMap::new();
        let mut identity_constraints = Vec::new();

        for branch in &self.branches {
            match &branch.result {
                BranchResult::Finite(solutions) => {
                    finite_groups
                        .entry(branch.constraint.clone())
                        .or_default()
                        .extend(solutions.iter().cloned());
                }
                BranchResult::Identity => {
                    if !identity_constraints.contains(&branch.constraint) {
                        identity_constraints.push(branch.constraint.clone());
                    }
                }
            }
        }

        let mut branches = identity_constraints
            .into_iter()
            .map(SolutionBranch::identity)
            .collect::<Vec<_>>();

        for (constraint, solutions) in finite_groups {
            let branch = SolutionBranch::finite(constraint, solutions);
            if !matches!(branch.result, BranchResult::Finite(ref values) if values.is_empty()) {
                branches.push(branch);
            }
        }

        SolutionSet::new(self.target.clone(), branches)
    }
}

impl SolutionBranch {
    /// 构造有限解，在该条件下解为有限确定值
    pub fn finite(constraint: LogicalExpression, solutions: Vec<NumericExpression>) -> Self {
        Self {
            constraint,
            result: BranchResult::Finite(dedup_solutions(solutions)),
        }
    }

    /// 构造任意解，在该条件下等式恒成立，任意值都是解
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

/// 判断 NumericExpression 是否为零
pub(crate) fn is_zero(expr: &NumericExpression) -> bool {
    let mut expr = expr.clone();
    normalize_numeric(&mut expr);
    matches!(expr, NumericExpression::Constant(number) if number.is_zero())
}

/// 判断 NumericExpression 是否包含目标变量
pub(crate) fn contains_target(expr: &NumericExpression, target: &Variable) -> bool {
    match expr {
        NumericExpression::Constant(_) => false,
        NumericExpression::Variable(variable) => variable.same_identity(target),
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
    let mut seen_ids = HashSet::new();
    let mut seen_names = HashSet::new();
    collect_numeric_variables_inner(expr, &mut variables, &mut seen_ids, &mut seen_names);
    variables
}

fn collect_numeric_variables_inner(
    expr: &NumericExpression,
    variables: &mut Vec<Variable>,
    seen_ids: &mut HashSet<crate::context::NameId>,
    seen_names: &mut HashSet<String>,
) {
    match expr {
        NumericExpression::Constant(_) => {}
        NumericExpression::Variable(variable) => {
            if remember_variable(variable, seen_ids, seen_names) {
                variables.push(variable.clone());
            }
        }
        NumericExpression::Negation(inner) => {
            collect_numeric_variables_inner(inner, variables, seen_ids, seen_names)
        }
        NumericExpression::Addition(bucket) | NumericExpression::Multiplication(bucket) => {
            for expr in bucket.iter() {
                collect_numeric_variables_inner(&expr, variables, seen_ids, seen_names);
            }
        }
        NumericExpression::Power { base, exponent } => {
            collect_numeric_variables_inner(base, variables, seen_ids, seen_names);
            collect_numeric_variables_inner(exponent, variables, seen_ids, seen_names);
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            for (condition, expr) in cases {
                collect_logical_variables(condition, variables, seen_ids, seen_names);
                collect_numeric_variables_inner(expr, variables, seen_ids, seen_names);
            }
            if let Some(expr) = otherwise {
                collect_numeric_variables_inner(expr, variables, seen_ids, seen_names);
            }
        }
    }
}

/// 从 LogicalExpression 中收集变量
fn collect_logical_variables(
    expr: &LogicalExpression,
    variables: &mut Vec<Variable>,
    seen_ids: &mut HashSet<crate::context::NameId>,
    seen_names: &mut HashSet<String>,
) {
    match expr {
        LogicalExpression::Constant(_) => {}
        LogicalExpression::Variable(variable) => {
            if remember_variable(variable, seen_ids, seen_names) {
                variables.push(variable.clone());
            }
        }
        LogicalExpression::Not(inner) => {
            collect_logical_variables(inner, variables, seen_ids, seen_names)
        }
        LogicalExpression::And(bucket) | LogicalExpression::Or(bucket) => {
            for expr in bucket.iter() {
                collect_logical_variables(&expr, variables, seen_ids, seen_names);
            }
        }
        LogicalExpression::Relation { left, right, .. } => {
            collect_numeric_variables_inner(left, variables, seen_ids, seen_names);
            collect_numeric_variables_inner(right, variables, seen_ids, seen_names);
        }
    }
}

/// 检查分支是否合法，
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
                .filter(|solution| {
                    verify_solution(target, branch_expr, &branch.constraint, solution)
                })
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
    let substituted = expr.substitute(target, Some(solution));
    let equation_holds = match catch_unwind(AssertUnwindSafe(|| {
        let mut substituted = substituted;
        normalize_numeric(&mut substituted);
        substituted
    })) {
        Ok(NumericExpression::Constant(number)) => {
            number.is_zero() || number.to_float().abs() < 1e-9
        }
        Ok(_) => true,
        Err(_) => true,
    };

    equation_holds
        && logical_is_not_false(
            &constraint.substitute(target, Some(&SemanticExpression::numeric(solution.clone()))),
        )
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
    let mut expr = expr.clone();
    normalize_logic(&mut expr);
    !matches!(expr, LogicalExpression::Constant(false))
}

/// 对单段函数中的解去重
fn dedup_solutions(solutions: Vec<NumericExpression>) -> Vec<NumericExpression> {
    let mut deduped = Vec::new();
    for solution in solutions {
        let mut normalized = solution;
        normalize_numeric(&mut normalized);
        if !deduped.contains(&normalized) {
            deduped.push(normalized);
        }
    }
    deduped
}

fn remember_variable(
    variable: &Variable,
    seen_ids: &mut HashSet<crate::context::NameId>,
    seen_names: &mut HashSet<String>,
) -> bool {
    if variable.name_id != 0 {
        seen_ids.insert(variable.name_id)
    } else {
        seen_names.insert(variable.name.clone())
    }
}

#[doc(hidden)]
pub mod testing {
    use super::*;

    pub fn is_zero_public(expr: &NumericExpression) -> bool {
        is_zero(expr)
    }

    pub fn contains_target_public(expr: &NumericExpression, target: &Variable) -> bool {
        contains_target(expr, target)
    }

    pub fn collect_numeric_variables_public(expr: &NumericExpression) -> Vec<Variable> {
        collect_numeric_variables(expr)
    }

    pub fn collect_logical_variables_public(expr: &LogicalExpression) -> Vec<Variable> {
        let mut variables = Vec::new();
        let mut seen_ids = HashSet::new();
        let mut seen_names = HashSet::new();
        collect_logical_variables(expr, &mut variables, &mut seen_ids, &mut seen_names);
        variables
    }

    pub fn collect_domain_constraint_public(expr: &NumericExpression) -> LogicalExpression {
        collect_domain_constraint(expr)
    }

    pub fn constraint_allows_identity_public(expr: &LogicalExpression) -> bool {
        constraint_allows_identity(expr)
    }

    pub fn logical_is_not_false_public(expr: &LogicalExpression) -> bool {
        logical_is_not_false(expr)
    }

    pub fn dedup_solutions_public(solutions: Vec<NumericExpression>) -> Vec<NumericExpression> {
        dedup_solutions(solutions)
    }

    pub fn verify_solution_public(
        target: &Variable,
        expr: &NumericExpression,
        constraint: &LogicalExpression,
        solution: &NumericExpression,
    ) -> bool {
        verify_solution(target, expr, constraint, solution)
    }

    pub fn verify_branch_public(
        target: &Variable,
        expr: &NumericExpression,
        branch: SolutionBranch,
    ) -> Result<SolutionBranch, SolveError> {
        verify_branch(target, expr, branch)
    }
}
