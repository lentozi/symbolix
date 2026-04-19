mod mcjit;

use crate::{
    lowering::LoweredNumericExpr,
    JitError, ParameterInfo,
};

pub(crate) trait CompiledNumericKernel {
    fn call(&self, arguments: &[f64]) -> f64;
}

pub(crate) trait Backend {
    fn compile_numeric(
        expr: &LoweredNumericExpr,
        parameters: &[ParameterInfo],
    ) -> Result<Box<dyn CompiledNumericKernel>, JitError>;
}

pub(crate) fn compile_numeric(
    expr: &LoweredNumericExpr,
    parameters: &[ParameterInfo],
) -> Result<Box<dyn CompiledNumericKernel>, JitError> {
    mcjit::McjitBackend::compile_numeric(expr, parameters)
}
