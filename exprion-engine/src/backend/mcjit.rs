use std::{ffi::{c_char, c_uint, c_void}, mem, ptr, sync::{Mutex, Once, OnceLock}};

use exprion_core::{
    lexer::symbol::{Relation, Symbol},
};

use crate::{
    backend::{Backend, CompiledNumericKernel},
    lowering::{LoweredLogicalExpr, LoweredNumericExpr},
    JitError, ParameterInfo,
};

type NumericEntry = unsafe extern "C" fn(*const f64) -> f64;

type LLVMContextRef = *mut c_void;
type LLVMModuleRef = *mut c_void;
type LLVMBuilderRef = *mut c_void;
type LLVMTypeRef = *mut c_void;
type LLVMValueRef = *mut c_void;
type LLVMBasicBlockRef = *mut c_void;
type LLVMExecutionEngineRef = *mut c_void;
type LLVMCodeModel = i32;
type LLVMRealPredicate = i32;

const LLVM_CODE_MODEL_DEFAULT: LLVMCodeModel = 0;
const LLVM_RETURN_STATUS_ACTION: i32 = 2;
const LLVM_REAL_OEQ: LLVMRealPredicate = 1;
const LLVM_REAL_OGT: LLVMRealPredicate = 2;
const LLVM_REAL_OGE: LLVMRealPredicate = 3;
const LLVM_REAL_OLT: LLVMRealPredicate = 4;
const LLVM_REAL_OLE: LLVMRealPredicate = 5;
const LLVM_REAL_ONE: LLVMRealPredicate = 14;

pub(crate) struct McjitBackend;

pub(crate) struct LlvmNumericKernel {
    _execution_engine: LLVMExecutionEngineRef,
    entry: NumericEntry,
}

impl Backend for McjitBackend {
    fn compile_numeric(
        expr: &LoweredNumericExpr,
        parameters: &[ParameterInfo],
    ) -> Result<Box<dyn CompiledNumericKernel>, JitError> {
        Ok(Box::new(LlvmNumericKernel::compile(expr, parameters)?))
    }
}

impl CompiledNumericKernel for LlvmNumericKernel {
    fn call(&self, arguments: &[f64]) -> f64 {
        unsafe { (self.entry)(arguments.as_ptr()) }
    }
}

impl LlvmNumericKernel {
    fn compile(
        expr: &LoweredNumericExpr,
        parameters: &[ParameterInfo],
    ) -> Result<Self, JitError> {
        let _guard = llvm_global_lock()
            .lock()
            .map_err(|_| JitError::Codegen("LLVM global lock was poisoned".to_string()))?;
        initialize_native_target()?;

        let context = unsafe { LLVMContextCreate() };
        if context.is_null() {
            return Err(JitError::Codegen("LLVMContextCreate returned null".to_string()));
        }

        let module = unsafe {
            LLVMModuleCreateWithNameInContext(static_cstr(b"exprion_engine_module\0"), context)
        };
        if module.is_null() {
            unsafe { LLVMContextDispose(context) };
            return Err(JitError::Codegen(
                "LLVMModuleCreateWithNameInContext returned null".to_string(),
            ));
        }

        let builder = unsafe { LLVMCreateBuilderInContext(context) };
        if builder.is_null() {
            unsafe {
                LLVMDisposeModule(module);
                LLVMContextDispose(context);
            }
            return Err(JitError::Codegen(
                "LLVMCreateBuilderInContext returned null".to_string(),
            ));
        }

        let compiled = (|| {
            let double_type = unsafe { LLVMDoubleTypeInContext(context) };
            let pointer_type = unsafe { LLVMPointerType(double_type, 0) };
            let mut params = [pointer_type];
            let function_type =
                unsafe { LLVMFunctionType(double_type, params.as_mut_ptr(), 1, 0) };
            let function = unsafe {
                LLVMAddFunction(module, static_cstr(b"exprion_numeric_entry\0"), function_type)
            };
            let runtime_pow = declare_runtime_pow(module, context)?;
            let runtime_sqrt = declare_runtime_sqrt(module, context)?;

            let entry_block = unsafe {
                LLVMAppendBasicBlockInContext(context, function, static_cstr(b"entry\0"))
            };
            unsafe { LLVMPositionBuilderAtEnd(builder, entry_block) };

            let args_ptr = unsafe { LLVMGetParam(function, 0) };
            let return_value = lower_numeric(
                expr,
                builder,
                context,
                double_type,
                args_ptr,
                runtime_pow,
                runtime_sqrt,
            )?;
            unsafe {
                LLVMBuildRet(builder, return_value);
            }

            verify_module(module)?;

            let execution_engine = create_execution_engine(module)?;
            unsafe {
                LLVMAddGlobalMapping(
                    execution_engine,
                    runtime_pow,
                    exprion_runtime_pow as *mut c_void,
                );
                LLVMAddGlobalMapping(
                    execution_engine,
                    runtime_sqrt,
                    exprion_runtime_sqrt as *mut c_void,
                );
            }
            let address = unsafe {
                LLVMGetFunctionAddress(execution_engine, static_cstr(b"exprion_numeric_entry\0"))
            };
            if address == 0 {
                unsafe { LLVMDisposeExecutionEngine(execution_engine) };
                return Err(JitError::Codegen(
                    "LLVMGetFunctionAddress returned 0".to_string(),
                ));
            }

            let entry = unsafe { mem::transmute::<usize, NumericEntry>(address as usize) };
            Ok((execution_engine, entry))
        })();

        unsafe {
            LLVMDisposeBuilder(builder);
        }

        match compiled {
            Ok((execution_engine, entry)) => Ok(Self {
                _execution_engine: execution_engine,
                entry,
            }),
            Err(err) => {
                unsafe {
                    LLVMDisposeModule(module);
                    LLVMContextDispose(context);
                }
                Err(err)
            }
        }
    }
}

fn llvm_global_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lower_numeric(
    expr: &LoweredNumericExpr,
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    double_type: LLVMTypeRef,
    args_ptr: LLVMValueRef,
    runtime_pow: LLVMValueRef,
    runtime_sqrt: LLVMValueRef,
) -> Result<LLVMValueRef, JitError> {
    match expr {
        LoweredNumericExpr::Constant(number) => {
            Ok(unsafe { LLVMConstReal(double_type, number.to_float()) })
        }
        LoweredNumericExpr::Parameter(index) => lower_parameter(*index, builder, double_type, args_ptr),
        LoweredNumericExpr::Negation(inner) => {
            let value = lower_numeric(
                inner,
                builder,
                context,
                double_type,
                args_ptr,
                runtime_pow,
                runtime_sqrt,
            )?;
            Ok(unsafe { LLVMBuildFNeg(builder, value, static_cstr(b"neg\0")) })
        }
        LoweredNumericExpr::Addition(terms) => lower_numeric_terms(
            terms,
            builder,
            context,
            double_type,
            args_ptr,
            runtime_pow,
            runtime_sqrt,
            0.0,
            b"addtmp\0",
            LLVMBuildFAdd,
        ),
        LoweredNumericExpr::Multiplication(terms) => lower_numeric_terms(
            terms,
            builder,
            context,
            double_type,
            args_ptr,
            runtime_pow,
            runtime_sqrt,
            1.0,
            b"multmp\0",
            LLVMBuildFMul,
        ),
        LoweredNumericExpr::Power { base, exponent } => {
            let base = lower_numeric(
                base,
                builder,
                context,
                double_type,
                args_ptr,
                runtime_pow,
                runtime_sqrt,
            )?;
            if let LoweredNumericExpr::Constant(number) = exponent.as_ref() {
                if let Some(power) = number.to_integer() {
                    return lower_integer_power(builder, double_type, base, power);
                }
                if let Some(value) = lower_fractional_power(
                    builder,
                    double_type,
                    base,
                    number.to_float(),
                    runtime_sqrt,
                )? {
                    return Ok(value);
                }
            }
            let exponent = lower_numeric(
                exponent,
                builder,
                context,
                double_type,
                args_ptr,
                runtime_pow,
                runtime_sqrt,
            )?;
            let function_type = unsafe {
                let mut params = [double_type, double_type];
                LLVMFunctionType(double_type, params.as_mut_ptr(), 2, 0)
            };
            let mut args = [base, exponent];
            Ok(unsafe {
                LLVMBuildCall2(
                    builder,
                    function_type,
                    runtime_pow,
                    args.as_mut_ptr(),
                    2,
                    static_cstr(b"powtmp\0"),
                )
            })
        }
        LoweredNumericExpr::Piecewise { cases, otherwise } => {
            lower_piecewise_numeric(
                cases,
                otherwise.as_deref(),
                builder,
                context,
                double_type,
                args_ptr,
                runtime_pow,
                runtime_sqrt,
            )
        }
    }
}

fn simplify_piecewise_cases(
    cases: &[(LoweredLogicalExpr, LoweredNumericExpr)],
) -> Vec<(&LoweredLogicalExpr, &LoweredNumericExpr)> {
    let mut simplified = Vec::with_capacity(cases.len());
    for (condition, branch) in cases {
        match condition {
            LoweredLogicalExpr::Constant(false) => {}
            LoweredLogicalExpr::Constant(true) => {
                simplified.push((condition, branch));
                break;
            }
            _ => simplified.push((condition, branch)),
        }
    }
    simplified
}

#[allow(clippy::too_many_arguments)]
fn lower_piecewise_numeric(
    cases: &[(LoweredLogicalExpr, LoweredNumericExpr)],
    otherwise: Option<&LoweredNumericExpr>,
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    double_type: LLVMTypeRef,
    args_ptr: LLVMValueRef,
    runtime_pow: LLVMValueRef,
    runtime_sqrt: LLVMValueRef,
) -> Result<LLVMValueRef, JitError> {
    let optimized_cases = simplify_piecewise_cases(cases);
    if optimized_cases.is_empty() {
        return if let Some(otherwise) = otherwise {
            lower_numeric(
                otherwise,
                builder,
                context,
                double_type,
                args_ptr,
                runtime_pow,
                runtime_sqrt,
            )
        } else {
            Ok(unsafe { LLVMConstReal(double_type, f64::NAN) })
        };
    }

    let current_block = unsafe { LLVMGetInsertBlock(builder) };
    let function = unsafe { LLVMGetBasicBlockParent(current_block) };
    let merge_block =
        unsafe { LLVMAppendBasicBlockInContext(context, function, static_cstr(b"piece.merge\0")) };

    let mut incoming_values = Vec::with_capacity(optimized_cases.len() + 1);
    let mut incoming_blocks = Vec::with_capacity(optimized_cases.len() + 1);

    for (index, (condition, branch)) in optimized_cases.iter().enumerate() {
        let then_block = unsafe {
            LLVMAppendBasicBlockInContext(context, function, static_cstr(b"piece.then\0"))
        };
        let else_block = unsafe {
            LLVMAppendBasicBlockInContext(context, function, static_cstr(b"piece.else\0"))
        };

        let cond = lower_logical(condition, builder, context, args_ptr, runtime_pow, runtime_sqrt)?;
        unsafe { LLVMBuildCondBr(builder, cond, then_block, else_block) };

        unsafe { LLVMPositionBuilderAtEnd(builder, then_block) };
        let then_value = lower_numeric(
            branch,
            builder,
            context,
            double_type,
            args_ptr,
            runtime_pow,
            runtime_sqrt,
        )?;
        unsafe { LLVMBuildBr(builder, merge_block) };
        incoming_values.push(then_value);
        incoming_blocks.push(then_block);

        unsafe { LLVMPositionBuilderAtEnd(builder, else_block) };
        if index + 1 == optimized_cases.len() {
            let else_value = if let Some(otherwise) = otherwise {
                lower_numeric(
                    otherwise,
                    builder,
                    context,
                    double_type,
                    args_ptr,
                    runtime_pow,
                    runtime_sqrt,
                )?
            } else {
                unsafe { LLVMConstReal(double_type, f64::NAN) }
            };
            unsafe { LLVMBuildBr(builder, merge_block) };
            incoming_values.push(else_value);
            incoming_blocks.push(else_block);
        }
    }

    unsafe { LLVMPositionBuilderAtEnd(builder, merge_block) };
    let phi = unsafe { LLVMBuildPhi(builder, double_type, static_cstr(b"piece.phi\0")) };
    unsafe {
        LLVMAddIncoming(
            phi,
            incoming_values.as_mut_ptr(),
            incoming_blocks.as_mut_ptr(),
            incoming_values.len() as c_uint,
        );
    }
    Ok(phi)
}

fn lower_integer_power(
    builder: LLVMBuilderRef,
    double_type: LLVMTypeRef,
    base: LLVMValueRef,
    power: i64,
) -> Result<LLVMValueRef, JitError> {
    let one = unsafe { LLVMConstReal(double_type, 1.0) };

    if power == 0 {
        return Ok(one);
    }
    if power == 1 {
        return Ok(base);
    }
    if power == -1 {
        return Ok(unsafe { LLVMBuildFDiv(builder, one, base, static_cstr(b"invtmp\0")) });
    }
    if power == 2 {
        return Ok(unsafe { LLVMBuildFMul(builder, base, base, static_cstr(b"pow2\0")) });
    }
    if power == 3 {
        let square = unsafe { LLVMBuildFMul(builder, base, base, static_cstr(b"pow3sq\0")) };
        return Ok(unsafe { LLVMBuildFMul(builder, square, base, static_cstr(b"pow3\0")) });
    }
    if power == 4 {
        let square = unsafe { LLVMBuildFMul(builder, base, base, static_cstr(b"pow4sq\0")) };
        return Ok(unsafe { LLVMBuildFMul(builder, square, square, static_cstr(b"pow4\0")) });
    }
    if power == -2 {
        let square = unsafe { LLVMBuildFMul(builder, base, base, static_cstr(b"powm2sq\0")) };
        return Ok(unsafe { LLVMBuildFDiv(builder, one, square, static_cstr(b"powm2\0")) });
    }
    if power == -3 {
        let square = unsafe { LLVMBuildFMul(builder, base, base, static_cstr(b"powm3sq\0")) };
        let cube = unsafe { LLVMBuildFMul(builder, square, base, static_cstr(b"powm3cube\0")) };
        return Ok(unsafe { LLVMBuildFDiv(builder, one, cube, static_cstr(b"powm3\0")) });
    }

    let negative = power < 0;
    let mut exponent = power.unsigned_abs();
    let mut result = one;
    let mut factor = base;

    while exponent > 0 {
        if exponent & 1 == 1 {
            result = unsafe { LLVMBuildFMul(builder, result, factor, static_cstr(b"powimul\0")) };
        }
        exponent >>= 1;
        if exponent > 0 {
            factor = unsafe { LLVMBuildFMul(builder, factor, factor, static_cstr(b"powisq\0")) };
        }
    }

    if negative {
        Ok(unsafe { LLVMBuildFDiv(builder, one, result, static_cstr(b"powineg\0")) })
    } else {
        Ok(result)
    }
}

fn lower_fractional_power(
    builder: LLVMBuilderRef,
    double_type: LLVMTypeRef,
    base: LLVMValueRef,
    power: f64,
    runtime_sqrt: LLVMValueRef,
) -> Result<Option<LLVMValueRef>, JitError> {
    let one = unsafe { LLVMConstReal(double_type, 1.0) };
    let function_type = unsafe {
        let mut params = [double_type];
        LLVMFunctionType(double_type, params.as_mut_ptr(), 1, 0)
    };

    if (power - 0.5).abs() < 1e-12 {
        let mut args = [base];
        let value = unsafe {
            LLVMBuildCall2(
                builder,
                function_type,
                runtime_sqrt,
                args.as_mut_ptr(),
                1,
                static_cstr(b"sqrttmp\0"),
            )
        };
        return Ok(Some(value));
    }

    if (power + 0.5).abs() < 1e-12 {
        let mut args = [base];
        let sqrt_value = unsafe {
            LLVMBuildCall2(
                builder,
                function_type,
                runtime_sqrt,
                args.as_mut_ptr(),
                1,
                static_cstr(b"sqrtnegtmp\0"),
            )
        };
        let value =
            unsafe { LLVMBuildFDiv(builder, one, sqrt_value, static_cstr(b"invsqrttmp\0")) };
        return Ok(Some(value));
    }

    Ok(None)
}

fn lower_logical(
    expr: &LoweredLogicalExpr,
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    args_ptr: LLVMValueRef,
    runtime_pow: LLVMValueRef,
    runtime_sqrt: LLVMValueRef,
) -> Result<LLVMValueRef, JitError> {
    let bool_type = unsafe { LLVMInt1TypeInContext(context) };
    match expr {
        LoweredLogicalExpr::Constant(value) => Ok(unsafe {
            LLVMConstInt(bool_type, if *value { 1 } else { 0 }, 0)
        }),
        LoweredLogicalExpr::Not(inner) => {
            let value = lower_logical(inner, builder, context, args_ptr, runtime_pow, runtime_sqrt)?;
            let true_value = unsafe { LLVMConstInt(bool_type, 1, 0) };
            Ok(unsafe { LLVMBuildXor(builder, value, true_value, static_cstr(b"nottmp\0")) })
        }
        LoweredLogicalExpr::And(terms) => {
            if terms.iter().any(|term| matches!(term, LoweredLogicalExpr::Constant(false))) {
                return Ok(unsafe { LLVMConstInt(bool_type, 0, 0) });
            }
            lower_logical_terms(
                terms,
                builder,
                context,
                args_ptr,
                true,
                b"andtmp\0",
                runtime_pow,
                runtime_sqrt,
                LLVMBuildAnd,
            )
        }
        LoweredLogicalExpr::Or(terms) => {
            if terms.iter().any(|term| matches!(term, LoweredLogicalExpr::Constant(true))) {
                return Ok(unsafe { LLVMConstInt(bool_type, 1, 0) });
            }
            lower_logical_terms(
                terms,
                builder,
                context,
                args_ptr,
                false,
                b"ortmp\0",
                runtime_pow,
                runtime_sqrt,
                LLVMBuildOr,
            )
        }
        LoweredLogicalExpr::Relation { left, operator, right } => {
            let double_type = unsafe { LLVMDoubleTypeInContext(context) };
            if let (
                LoweredNumericExpr::Constant(left_num),
                LoweredNumericExpr::Constant(right_num),
            ) = (left.as_ref(), right.as_ref())
            {
                let result = match operator {
                    Relation::Equal => {
                        (left_num.to_float() - right_num.to_float()).abs() < 1e-9
                    }
                    Relation::NotEqual => {
                        (left_num.to_float() - right_num.to_float()).abs() >= 1e-9
                    }
                    Relation::LessThan => left_num.to_float() < right_num.to_float(),
                    Relation::GreaterThan => left_num.to_float() > right_num.to_float(),
                    Relation::LessEqual => left_num.to_float() <= right_num.to_float(),
                    Relation::GreaterEqual => {
                        left_num.to_float() >= right_num.to_float()
                    }
                };
                return Ok(unsafe { LLVMConstInt(bool_type, if result { 1 } else { 0 }, 0) });
            }
            let lhs = lower_numeric(
                left,
                builder,
                context,
                double_type,
                args_ptr,
                ptr::null_mut(),
                ptr::null_mut(),
            )?;
            let rhs = lower_numeric(
                right,
                builder,
                context,
                double_type,
                args_ptr,
                ptr::null_mut(),
                ptr::null_mut(),
            )?;
            let predicate = real_predicate(&Symbol::Relation(*operator))?;
            Ok(unsafe { LLVMBuildFCmp(builder, predicate, lhs, rhs, static_cstr(b"cmptmp\0")) })
        }
    }
}

fn lower_parameter(
    index: usize,
    builder: LLVMBuilderRef,
    double_type: LLVMTypeRef,
    args_ptr: LLVMValueRef,
) -> Result<LLVMValueRef, JitError> {
    let index_value = unsafe { LLVMConstInt(LLVMInt64Type(), index as u64, 0) };
    let mut indices = [index_value];
    let value_ptr = unsafe {
        LLVMBuildGEP2(
            builder,
            double_type,
            args_ptr,
            indices.as_mut_ptr(),
            1,
            static_cstr(b"var_ptr\0"),
        )
    };
    Ok(unsafe { LLVMBuildLoad2(builder, double_type, value_ptr, static_cstr(b"var\0")) })
}

#[allow(clippy::too_many_arguments)]
fn lower_numeric_terms(
    terms: &[LoweredNumericExpr],
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    double_type: LLVMTypeRef,
    args_ptr: LLVMValueRef,
    runtime_pow: LLVMValueRef,
    runtime_sqrt: LLVMValueRef,
    empty_value: f64,
    temp_name: &'static [u8],
    combine: unsafe extern "C" fn(
        LLVMBuilderRef,
        LLVMValueRef,
        LLVMValueRef,
        *const c_char,
    ) -> LLVMValueRef,
) -> Result<LLVMValueRef, JitError> {
    let mut acc = None;

    for expression in terms {
        let value = lower_numeric(
            expression,
            builder,
            context,
            double_type,
            args_ptr,
            runtime_pow,
            runtime_sqrt,
        )?;
        acc = Some(match acc {
            Some(lhs) => {
                unsafe { combine(builder, lhs, value, static_cstr(temp_name)) }
            }
            None => value,
        });
    }

    Ok(acc.unwrap_or_else(|| unsafe { LLVMConstReal(double_type, empty_value) }))
}

#[allow(clippy::too_many_arguments)]
fn lower_logical_terms(
    terms: &[LoweredLogicalExpr],
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    args_ptr: LLVMValueRef,
    empty_value: bool,
    temp_name: &'static [u8],
    runtime_pow: LLVMValueRef,
    runtime_sqrt: LLVMValueRef,
    combine: unsafe extern "C" fn(
        LLVMBuilderRef,
        LLVMValueRef,
        LLVMValueRef,
        *const c_char,
    ) -> LLVMValueRef,
) -> Result<LLVMValueRef, JitError> {
    let bool_type = unsafe { LLVMInt1TypeInContext(context) };
    let mut acc = None;

    for expression in terms {
        let value = lower_logical(expression, builder, context, args_ptr, runtime_pow, runtime_sqrt)?;
        acc = Some(match acc {
            Some(lhs) => {
                unsafe { combine(builder, lhs, value, static_cstr(temp_name)) }
            }
            None => value,
        });
    }

    Ok(acc.unwrap_or_else(|| unsafe {
        LLVMConstInt(bool_type, if empty_value { 1 } else { 0 }, 0)
    }))
}

fn real_predicate(operator: &Symbol) -> Result<LLVMRealPredicate, JitError> {
    match operator {
        Symbol::Relation(Relation::Equal) => Ok(LLVM_REAL_OEQ),
        Symbol::Relation(Relation::NotEqual) => Ok(LLVM_REAL_ONE),
        Symbol::Relation(Relation::LessThan) => Ok(LLVM_REAL_OLT),
        Symbol::Relation(Relation::GreaterThan) => Ok(LLVM_REAL_OGT),
        Symbol::Relation(Relation::LessEqual) => Ok(LLVM_REAL_OLE),
        Symbol::Relation(Relation::GreaterEqual) => Ok(LLVM_REAL_OGE),
        _ => Err(JitError::UnsupportedExpression(format!(
            "unsupported relation operator `{operator}`"
        ))),
    }
}

fn declare_runtime_pow(
    module: LLVMModuleRef,
    context: LLVMContextRef,
) -> Result<LLVMValueRef, JitError> {
    let double_type = unsafe { LLVMDoubleTypeInContext(context) };
    let mut params = [double_type, double_type];
    let function_type = unsafe { LLVMFunctionType(double_type, params.as_mut_ptr(), 2, 0) };
    Ok(unsafe { LLVMAddFunction(module, static_cstr(b"exprion_runtime_pow\0"), function_type) })
}

extern "C" fn exprion_runtime_pow(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
}

fn declare_runtime_sqrt(
    module: LLVMModuleRef,
    context: LLVMContextRef,
) -> Result<LLVMValueRef, JitError> {
    let double_type = unsafe { LLVMDoubleTypeInContext(context) };
    let mut params = [double_type];
    let function_type = unsafe { LLVMFunctionType(double_type, params.as_mut_ptr(), 1, 0) };
    Ok(unsafe { LLVMAddFunction(module, static_cstr(b"exprion_runtime_sqrt\0"), function_type) })
}

extern "C" fn exprion_runtime_sqrt(value: f64) -> f64 {
    value.sqrt()
}

fn verify_module(module: LLVMModuleRef) -> Result<(), JitError> {
    let mut message = ptr::null_mut();
    let failed = unsafe { LLVMVerifyModule(module, LLVM_RETURN_STATUS_ACTION, &mut message) };
    if failed == 0 {
        return Ok(());
    }
    Err(JitError::Codegen(take_message(message)))
}

fn create_execution_engine(module: LLVMModuleRef) -> Result<LLVMExecutionEngineRef, JitError> {
    let mut engine = ptr::null_mut();
    let mut options = LLVMMCJITCompilerOptions {
        OptLevel: 3,
        CodeModel: LLVM_CODE_MODEL_DEFAULT,
        NoFramePointerElim: 0,
        EnableFastISel: 1,
        MCJMM: ptr::null_mut(),
    };
    unsafe {
        LLVMInitializeMCJITCompilerOptions(
            &mut options,
            mem::size_of::<LLVMMCJITCompilerOptions>(),
        );
    }

    let mut message = ptr::null_mut();
    let failed = unsafe {
        LLVMCreateMCJITCompilerForModule(
            &mut engine,
            module,
            &mut options,
            mem::size_of::<LLVMMCJITCompilerOptions>(),
            &mut message,
        )
    };

    if failed == 0 {
        Ok(engine)
    } else {
        Err(JitError::Codegen(take_message(message)))
    }
}

fn initialize_native_target() -> Result<(), JitError> {
    static INIT: Once = Once::new();
    static mut ERROR: Option<&'static str> = None;

    INIT.call_once(|| unsafe {
        LLVMLinkInMCJIT();

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            LLVMInitializeX86TargetInfo();
            LLVMInitializeX86Target();
            LLVMInitializeX86TargetMC();
            LLVMInitializeX86AsmPrinter();
        }

        #[cfg(target_arch = "aarch64")]
        {
            LLVMInitializeAArch64TargetInfo();
            LLVMInitializeAArch64Target();
            LLVMInitializeAArch64TargetMC();
            LLVMInitializeAArch64AsmPrinter();
        }

        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        {
            ERROR = Some("LLVM JIT backend does not support this target architecture yet");
        }
    });

    unsafe {
        if let Some(message) = ERROR {
            Err(JitError::UnsupportedPlatform(message.to_string()))
        } else {
            Ok(())
        }
    }
}

fn take_message(message: *mut c_char) -> String {
    if message.is_null() {
        return "LLVM returned an empty error message".to_string();
    }

    let text = unsafe { std::ffi::CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    unsafe { LLVMDisposeMessage(message) };
    text
}

fn static_cstr(value: &'static [u8]) -> *const c_char {
    value.as_ptr().cast()
}

#[repr(C)]
#[allow(non_snake_case)]
struct LLVMMCJITCompilerOptions {
    OptLevel: c_uint,
    CodeModel: LLVMCodeModel,
    NoFramePointerElim: c_uint,
    EnableFastISel: c_uint,
    MCJMM: *mut c_void,
}

#[link(name = "LLVM-C")]
extern "C" {
    fn LLVMContextCreate() -> LLVMContextRef;
    fn LLVMContextDispose(C: LLVMContextRef);

    fn LLVMModuleCreateWithNameInContext(ModuleID: *const c_char, C: LLVMContextRef)
        -> LLVMModuleRef;
    fn LLVMDisposeModule(M: LLVMModuleRef);

    fn LLVMDoubleTypeInContext(C: LLVMContextRef) -> LLVMTypeRef;
    fn LLVMInt1TypeInContext(C: LLVMContextRef) -> LLVMTypeRef;
    fn LLVMInt8TypeInContext(C: LLVMContextRef) -> LLVMTypeRef;
    fn LLVMInt64Type() -> LLVMTypeRef;
    fn LLVMPointerType(ElementType: LLVMTypeRef, AddressSpace: c_uint) -> LLVMTypeRef;
    fn LLVMFunctionType(
        ReturnType: LLVMTypeRef,
        ParamTypes: *mut LLVMTypeRef,
        ParamCount: c_uint,
        IsVarArg: c_uint,
    ) -> LLVMTypeRef;

    fn LLVMAddFunction(M: LLVMModuleRef, Name: *const c_char, FunctionTy: LLVMTypeRef)
        -> LLVMValueRef;
    fn LLVMGetParam(Fn: LLVMValueRef, Index: c_uint) -> LLVMValueRef;

    fn LLVMAppendBasicBlockInContext(
        C: LLVMContextRef,
        Fn: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMBasicBlockRef;
    fn LLVMGetInsertBlock(Builder: LLVMBuilderRef) -> LLVMBasicBlockRef;
    fn LLVMGetBasicBlockParent(BB: LLVMBasicBlockRef) -> LLVMValueRef;

    fn LLVMCreateBuilderInContext(C: LLVMContextRef) -> LLVMBuilderRef;
    fn LLVMPositionBuilderAtEnd(Builder: LLVMBuilderRef, Block: LLVMBasicBlockRef);
    fn LLVMDisposeBuilder(Builder: LLVMBuilderRef);

    fn LLVMConstReal(RealTy: LLVMTypeRef, N: f64) -> LLVMValueRef;
    fn LLVMConstInt(IntTy: LLVMTypeRef, N: u64, SignExtend: c_uint) -> LLVMValueRef;

    fn LLVMBuildRet(Builder: LLVMBuilderRef, V: LLVMValueRef) -> LLVMValueRef;
    fn LLVMBuildBr(Builder: LLVMBuilderRef, Dest: LLVMBasicBlockRef) -> LLVMValueRef;
    fn LLVMBuildCondBr(
        Builder: LLVMBuilderRef,
        If: LLVMValueRef,
        Then: LLVMBasicBlockRef,
        Else: LLVMBasicBlockRef,
    ) -> LLVMValueRef;
    fn LLVMBuildCall2(
        Builder: LLVMBuilderRef,
        Ty: LLVMTypeRef,
        Fn: LLVMValueRef,
        Args: *mut LLVMValueRef,
        NumArgs: c_uint,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildFAdd(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildFMul(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildFDiv(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildFNeg(
        Builder: LLVMBuilderRef,
        V: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildLoad2(
        Builder: LLVMBuilderRef,
        Ty: LLVMTypeRef,
        PointerVal: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildGEP2(
        Builder: LLVMBuilderRef,
        Ty: LLVMTypeRef,
        Pointer: LLVMValueRef,
        Indices: *mut LLVMValueRef,
        NumIndices: c_uint,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildFCmp(
        Builder: LLVMBuilderRef,
        Op: LLVMRealPredicate,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildSelect(
        Builder: LLVMBuilderRef,
        If: LLVMValueRef,
        Then: LLVMValueRef,
        Else: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildPhi(
        Builder: LLVMBuilderRef,
        Ty: LLVMTypeRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMAddIncoming(
        PhiNode: LLVMValueRef,
        IncomingValues: *mut LLVMValueRef,
        IncomingBlocks: *mut LLVMBasicBlockRef,
        Count: c_uint,
    );
    fn LLVMBuildAnd(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildOr(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildXor(
        Builder: LLVMBuilderRef,
        LHS: LLVMValueRef,
        RHS: LLVMValueRef,
        Name: *const c_char,
    ) -> LLVMValueRef;
    fn LLVMBuildZExt(
        Builder: LLVMBuilderRef,
        Val: LLVMValueRef,
        DestTy: LLVMTypeRef,
        Name: *const c_char,
    ) -> LLVMValueRef;

    fn LLVMVerifyModule(
        M: LLVMModuleRef,
        Action: i32,
        OutMessage: *mut *mut c_char,
    ) -> c_uint;
    fn LLVMDisposeMessage(Message: *mut c_char);

    fn LLVMLinkInMCJIT();
    fn LLVMInitializeMCJITCompilerOptions(
        Options: *mut LLVMMCJITCompilerOptions,
        SizeOfOptions: usize,
    );
    fn LLVMCreateMCJITCompilerForModule(
        OutJIT: *mut LLVMExecutionEngineRef,
        M: LLVMModuleRef,
        Options: *mut LLVMMCJITCompilerOptions,
        SizeOfOptions: usize,
        OutError: *mut *mut c_char,
    ) -> c_uint;
    fn LLVMDisposeExecutionEngine(EE: LLVMExecutionEngineRef);
    fn LLVMGetFunctionAddress(EE: LLVMExecutionEngineRef, Name: *const c_char) -> u64;
    fn LLVMAddGlobalMapping(
        EE: LLVMExecutionEngineRef,
        Global: LLVMValueRef,
        Addr: *mut c_void,
    );

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn LLVMInitializeX86TargetInfo();
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn LLVMInitializeX86Target();
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn LLVMInitializeX86TargetMC();
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    fn LLVMInitializeX86AsmPrinter();

    #[cfg(target_arch = "aarch64")]
    fn LLVMInitializeAArch64TargetInfo();
    #[cfg(target_arch = "aarch64")]
    fn LLVMInitializeAArch64Target();
    #[cfg(target_arch = "aarch64")]
    fn LLVMInitializeAArch64TargetMC();
    #[cfg(target_arch = "aarch64")]
    fn LLVMInitializeAArch64AsmPrinter();
}
