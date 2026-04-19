use exprion_engine as engine;

use crate::Expr;

pub use engine::{JitError, NumericCacheStats, ParameterInfo};

pub struct JitFunction(pub(crate) engine::JitNumericFunction);

impl JitFunction {
    pub fn arity(&self) -> usize {
        self.0.arity()
    }

    pub fn variables(&self) -> Vec<String> {
        self.0.variables()
    }

    pub fn parameters(&self) -> &[ParameterInfo] {
        self.0.parameters()
    }

    pub fn calculate(&self, arguments: &[f64]) -> Result<f64, JitError> {
        self.0.calculate(arguments)
    }

    pub fn calculate_named(&self, arguments: &[(&str, f64)]) -> Result<f64, JitError> {
        self.0.calculate_named(arguments)
    }

    pub fn calculate_unchecked(&self, arguments: &[f64]) -> f64 {
        self.0.calculate_unchecked(arguments)
    }

    pub fn raw(&self) -> &engine::JitNumericFunction {
        &self.0
    }
}

pub fn compile_numeric(expr: Expr) -> Result<JitFunction, JitError> {
    Ok(JitFunction(engine::jit_compile_numeric(expr.into_semantic())?))
}

pub fn numeric_cache_stats() -> NumericCacheStats {
    engine::numeric_cache_stats()
}

pub fn reset_numeric_cache_stats() {
    engine::reset_numeric_cache_stats();
}
