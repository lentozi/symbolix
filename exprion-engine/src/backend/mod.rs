mod mcjit;

use exprion_core::semantic::semantic_ir::numeric::NumericExpression;

use crate::{JitError, ParameterInfo};

pub(crate) trait CompiledNumericKernel {
    fn call(&self, arguments: &[f64]) -> f64;
}

pub(crate) trait Backend {
    fn compile_numeric(
        expr: &NumericExpression,
        parameters: &[ParameterInfo],
    ) -> Result<Box<dyn CompiledNumericKernel>, JitError>;
}

pub(crate) fn compile_numeric(
    expr: &NumericExpression,
    parameters: &[ParameterInfo],
) -> Result<Box<dyn CompiledNumericKernel>, JitError> {
    mcjit::McjitBackend::compile_numeric(expr, parameters)
}
