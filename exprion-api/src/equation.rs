use std::fmt;

use exprion_core::equation as core_equation;

use crate::{Expr, Var};

pub use exprion_core::equation::SolveError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Equation(pub(crate) core_equation::Equation);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolutionSet(pub(crate) core_equation::SolutionSet);

impl Equation {
    pub fn new(expr: Expr, solve_for: &Var) -> Result<Self, SolveError> {
        Ok(Self(core_equation::Equation::new(
            expr.into_semantic(),
            solve_for.raw().clone(),
        )?))
    }

    pub fn infer(expr: Expr) -> Result<Self, SolveError> {
        Ok(Self(core_equation::Equation::infer(expr.into_semantic())?))
    }

    pub fn expr(&self) -> Expr {
        Expr::numeric(self.0.expr.clone())
    }

    pub fn solve_for(&self) -> Var {
        Var(self.0.solve_for.clone())
    }

    pub fn solve(&self) -> Result<SolutionSet, SolveError> {
        Ok(SolutionSet(self.0.solve()?))
    }

    pub fn solve_unique(&self) -> Result<Expr, SolveError> {
        Ok(Expr::from_semantic(self.0.solve_unique()?))
    }

    pub fn raw(&self) -> &core_equation::Equation {
        &self.0
    }
}

impl SolutionSet {
    pub fn target(&self) -> Var {
        Var(self.0.target.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn has_identity_branch(&self) -> bool {
        self.0.has_identity_branch()
    }

    pub fn into_expr(self) -> Result<Expr, SolveError> {
        Ok(Expr::numeric(self.0.into_numeric_expression()?))
    }

    pub fn raw(&self) -> &core_equation::SolutionSet {
        &self.0
    }
}

impl fmt::Display for Equation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = 0", self.0.expr)
    }
}

impl fmt::Display for SolutionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
