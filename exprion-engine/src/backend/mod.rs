mod mcjit;

use exprion_core::semantic::semantic_ir::{logic::LogicalExpression, numeric::NumericExpression};

use crate::{JitError, ParameterInfo};

pub(crate) trait CompiledNumericKernel {
    fn call(&self, arguments: &[f64]) -> f64;
}

pub(crate) trait CompiledLogicalKernel {
    fn call(&self, arguments: &[f64]) -> bool;
}

pub(crate) trait Backend {
    fn compile_numeric(
        expr: &NumericExpression,
        parameters: &[ParameterInfo],
    ) -> Result<Box<dyn CompiledNumericKernel>, JitError>;

    fn compile_logical(
        expr: &LogicalExpression,
        parameters: &[ParameterInfo],
    ) -> Result<Box<dyn CompiledLogicalKernel>, JitError>;
}

pub(crate) fn compile_numeric(
    expr: &NumericExpression,
    parameters: &[ParameterInfo],
) -> Result<Box<dyn CompiledNumericKernel>, JitError> {
    mcjit::McjitBackend::compile_numeric(expr, parameters)
}

pub(crate) fn compile_logical(
    expr: &LogicalExpression,
    parameters: &[ParameterInfo],
) -> Result<Box<dyn CompiledLogicalKernel>, JitError> {
    mcjit::McjitBackend::compile_logical(expr, parameters)
}
