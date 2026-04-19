mod convert;
mod equation;
mod expr;
mod jit;
mod ops;
mod scope;
mod var;

pub use convert::IntoExpr;
pub use equation::{Equation, SolutionSet, SolveError};
pub use expr::Expr;
pub use jit::{
    compile_numeric, numeric_cache_stats, reset_numeric_cache_stats, JitError, JitFunction,
    NumericCacheStats, ParameterInfo,
};
pub use scope::scope;
pub use var::Var;
