use crate::semantic::variable::Variable;
use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveError {
    UnsupportedEquationFormat,
    NoVariableToSolve,
    AmbiguousSolveTarget(Vec<Variable>),
    UnsupportedSolver(String),
    NonLinearExpression,
    NonPolynomialExpression,
    ExpectedUniqueSolutionSet,
    ExpectedUnconditionalSolution,
    VerificationFailed,
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolveError::UnsupportedEquationFormat => write!(f, "unsupported equation format"),
            SolveError::NoVariableToSolve => write!(f, "equation does not contain a variable"),
            SolveError::AmbiguousSolveTarget(variables) => {
                let names = variables
                    .iter()
                    .map(|variable| variable.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "solve target is ambiguous: {names}")
            }
            SolveError::UnsupportedSolver(message) => write!(f, "{message}"),
            SolveError::NonLinearExpression => write!(f, "equation is not linear in the target"),
            SolveError::NonPolynomialExpression => {
                write!(f, "equation is not a supported polynomial in the target")
            }
            SolveError::ExpectedUniqueSolutionSet => write!(f, "expected a unique solution"),
            SolveError::ExpectedUnconditionalSolution => {
                write!(f, "expected an unconditional solution")
            }
            SolveError::VerificationFailed => write!(f, "failed to verify solver output"),
        }
    }
}

impl Error for SolveError {}
