use std::{
    collections::HashMap,
    ffi::{c_char, c_uint, c_void, CString},
    mem,
    ptr,
    sync::{Mutex, Once, OnceLock},
};

use exprion_core::{
    lexer::symbol::{Relation, Symbol},
    semantic::semantic_ir::{logic::LogicalExpression, numeric::NumericExpression},
};

use crate::{
    backend::{Backend, CompiledLogicalKernel, CompiledNumericKernel},
    JitError, ParameterInfo,
};

type NumericEntry = unsafe extern "C" fn(*const f64) -> f64;
type LogicalEntry = unsafe extern "C" fn(*const f64) -> u8;

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

pub(crate) struct LlvmLogicalKernel {
    _execution_engine: LLVMExecutionEngineRef,
    entry: LogicalEntry,
}

impl Backend for McjitBackend {
    fn compile_numeric(
        expr: &NumericExpression,
        parameters: &[ParameterInfo],
    ) -> Result<Box<dyn CompiledNumericKernel>, JitError> {
        Ok(Box::new(LlvmNumericKernel::compile(expr, parameters)?))
    }

    fn compile_logical(
        expr: &LogicalExpression,
        parameters: &[ParameterInfo],
    ) -> Result<Box<dyn CompiledLogicalKernel>, JitError> {
        Ok(Box::new(LlvmLogicalKernel::compile(expr, parameters)?))
    }
}

impl CompiledNumericKernel for LlvmNumericKernel {
    fn call(&self, arguments: &[f64]) -> f64 {
        unsafe { (self.entry)(arguments.as_ptr()) }
    }
}

impl CompiledLogicalKernel for LlvmLogicalKernel {
    fn call(&self, arguments: &[f64]) -> bool {
        unsafe { (self.entry)(arguments.as_ptr()) != 0 }
    }
}

impl LlvmNumericKernel {
    fn compile(
        expr: &NumericExpression,
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

        let module_name = cstring("exprion_engine_module")?;
        let module = unsafe { LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context) };
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
            let function_name = cstring("exprion_numeric_entry")?;
            let function =
                unsafe { LLVMAddFunction(module, function_name.as_ptr(), function_type) };
            let runtime_pow = declare_runtime_pow(module, context)?;

            let block_name = cstring("entry")?;
            let entry_block =
                unsafe { LLVMAppendBasicBlockInContext(context, function, block_name.as_ptr()) };
            unsafe { LLVMPositionBuilderAtEnd(builder, entry_block) };

            let args_ptr = unsafe { LLVMGetParam(function, 0) };
            let variable_slots = parameter_slots(parameters);

            let return_value = lower_numeric(
                expr,
                builder,
                context,
                double_type,
                args_ptr,
                &variable_slots,
                runtime_pow,
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
            }
            let address =
                unsafe { LLVMGetFunctionAddress(execution_engine, function_name.as_ptr()) };
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

impl LlvmLogicalKernel {
    fn compile(
        expr: &LogicalExpression,
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

        let module_name = cstring("exprion_engine_logical_module")?;
        let module = unsafe { LLVMModuleCreateWithNameInContext(module_name.as_ptr(), context) };
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
            let bool_type = unsafe { LLVMInt1TypeInContext(context) };
            let return_type = unsafe { LLVMInt8TypeInContext(context) };
            let double_type = unsafe { LLVMDoubleTypeInContext(context) };
            let pointer_type = unsafe { LLVMPointerType(double_type, 0) };
            let mut params = [pointer_type];
            let function_type =
                unsafe { LLVMFunctionType(return_type, params.as_mut_ptr(), 1, 0) };
            let function_name = cstring("exprion_logical_entry")?;
            let function =
                unsafe { LLVMAddFunction(module, function_name.as_ptr(), function_type) };

            let block_name = cstring("entry")?;
            let entry_block =
                unsafe { LLVMAppendBasicBlockInContext(context, function, block_name.as_ptr()) };
            unsafe { LLVMPositionBuilderAtEnd(builder, entry_block) };

            let args_ptr = unsafe { LLVMGetParam(function, 0) };
            let variable_slots = parameter_slots(parameters);

            let result = lower_logical(expr, builder, context, args_ptr, &variable_slots)?;
            let extend_name = cstring("logical_ret")?;
            let widened =
                unsafe { LLVMBuildZExt(builder, result, return_type, extend_name.as_ptr()) };
            let _ = bool_type;
            unsafe {
                LLVMBuildRet(builder, widened);
            }

            verify_module(module)?;
            let execution_engine = create_execution_engine(module)?;
            let address =
                unsafe { LLVMGetFunctionAddress(execution_engine, function_name.as_ptr()) };
            if address == 0 {
                unsafe { LLVMDisposeExecutionEngine(execution_engine) };
                return Err(JitError::Codegen(
                    "LLVMGetFunctionAddress returned 0".to_string(),
                ));
            }

            let entry = unsafe { mem::transmute::<usize, LogicalEntry>(address as usize) };
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

fn parameter_slots(parameters: &[ParameterInfo]) -> HashMap<String, usize> {
    parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.index))
        .collect()
}

fn llvm_global_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lower_numeric(
    expr: &NumericExpression,
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    double_type: LLVMTypeRef,
    args_ptr: LLVMValueRef,
    variable_slots: &HashMap<String, usize>,
    runtime_pow: LLVMValueRef,
) -> Result<LLVMValueRef, JitError> {
    match expr {
        NumericExpression::Constant(number) => {
            Ok(unsafe { LLVMConstReal(double_type, number.to_float()) })
        }
        NumericExpression::Variable(variable) => {
            let index = variable_slots.get(&variable.name).ok_or_else(|| {
                JitError::Codegen(format!("missing variable slot for `{}`", variable.name))
            })?;

            let index_value = unsafe { LLVMConstInt(LLVMInt64Type(), *index as u64, 0) };
            let mut indices = [index_value];
            let gep_name = cstring(&format!("{}_ptr", variable.name))?;
            let value_name = cstring(&variable.name)?;
            let value_ptr = unsafe {
                LLVMBuildGEP2(
                    builder,
                    double_type,
                    args_ptr,
                    indices.as_mut_ptr(),
                    1,
                    gep_name.as_ptr(),
                )
            };
            Ok(unsafe { LLVMBuildLoad2(builder, double_type, value_ptr, value_name.as_ptr()) })
        }
        NumericExpression::Negation(inner) => {
            let value = lower_numeric(
                inner,
                builder,
                context,
                double_type,
                args_ptr,
                variable_slots,
                runtime_pow,
            )?;
            let name = cstring("neg")?;
            Ok(unsafe { LLVMBuildFNeg(builder, value, name.as_ptr()) })
        }
        NumericExpression::Addition(bucket) => {
            let mut iter = bucket.iter();
            let Some(first) = iter.next() else {
                return Ok(unsafe { LLVMConstReal(double_type, 0.0) });
            };
            let mut acc = lower_numeric(
                &first,
                builder,
                context,
                double_type,
                args_ptr,
                variable_slots,
                runtime_pow,
            )?;
            for term in iter {
                let rhs = lower_numeric(
                    &term,
                    builder,
                    context,
                    double_type,
                    args_ptr,
                    variable_slots,
                    runtime_pow,
                )?;
                let name = cstring("addtmp")?;
                acc = unsafe { LLVMBuildFAdd(builder, acc, rhs, name.as_ptr()) };
            }
            Ok(acc)
        }
        NumericExpression::Multiplication(bucket) => {
            let mut iter = bucket.iter();
            let Some(first) = iter.next() else {
                return Ok(unsafe { LLVMConstReal(double_type, 1.0) });
            };
            let mut acc = lower_numeric(
                &first,
                builder,
                context,
                double_type,
                args_ptr,
                variable_slots,
                runtime_pow,
            )?;
            for term in iter {
                let rhs = lower_numeric(
                    &term,
                    builder,
                    context,
                    double_type,
                    args_ptr,
                    variable_slots,
                    runtime_pow,
                )?;
                let name = cstring("multmp")?;
                acc = unsafe { LLVMBuildFMul(builder, acc, rhs, name.as_ptr()) };
            }
            Ok(acc)
        }
        NumericExpression::Power { base, exponent } => {
            let base = lower_numeric(
                base,
                builder,
                context,
                double_type,
                args_ptr,
                variable_slots,
                runtime_pow,
            )?;
            let exponent = lower_numeric(
                exponent,
                builder,
                context,
                double_type,
                args_ptr,
                variable_slots,
                runtime_pow,
            )?;
            let call_name = cstring("powtmp")?;
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
                    call_name.as_ptr(),
                )
            })
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            let mut current = if let Some(otherwise) = otherwise {
                lower_numeric(
                    otherwise,
                    builder,
                    context,
                    double_type,
                    args_ptr,
                    variable_slots,
                    runtime_pow,
                )?
            } else {
                unsafe { LLVMConstReal(double_type, f64::NAN) }
            };

            for (condition, branch) in cases.iter().rev() {
                let cond = lower_logical(condition, builder, context, args_ptr, variable_slots)?;
                let then_value = lower_numeric(
                    branch,
                    builder,
                    context,
                    double_type,
                    args_ptr,
                    variable_slots,
                    runtime_pow,
                )?;
                let select_name = cstring("piecewise")?;
                current = unsafe {
                    LLVMBuildSelect(builder, cond, then_value, current, select_name.as_ptr())
                };
            }
            Ok(current)
        }
    }
}

fn lower_logical(
    expr: &LogicalExpression,
    builder: LLVMBuilderRef,
    context: LLVMContextRef,
    args_ptr: LLVMValueRef,
    variable_slots: &HashMap<String, usize>,
) -> Result<LLVMValueRef, JitError> {
    let bool_type = unsafe { LLVMInt1TypeInContext(context) };
    match expr {
        LogicalExpression::Constant(value) => Ok(unsafe {
            LLVMConstInt(bool_type, if *value { 1 } else { 0 }, 0)
        }),
        LogicalExpression::Variable(variable) => {
            Err(JitError::UnsupportedLogicalVariable(variable.name.clone()))
        }
        LogicalExpression::Not(inner) => {
            let value = lower_logical(inner, builder, context, args_ptr, variable_slots)?;
            let true_value = unsafe { LLVMConstInt(bool_type, 1, 0) };
            let name = cstring("nottmp")?;
            Ok(unsafe { LLVMBuildXor(builder, value, true_value, name.as_ptr()) })
        }
        LogicalExpression::And(bucket) => {
            let mut iter = bucket.iter();
            let Some(first) = iter.next() else {
                return Ok(unsafe { LLVMConstInt(bool_type, 1, 0) });
            };
            let mut acc = lower_logical(&first, builder, context, args_ptr, variable_slots)?;
            for term in iter {
                let rhs = lower_logical(&term, builder, context, args_ptr, variable_slots)?;
                let name = cstring("andtmp")?;
                acc = unsafe { LLVMBuildAnd(builder, acc, rhs, name.as_ptr()) };
            }
            Ok(acc)
        }
        LogicalExpression::Or(bucket) => {
            let mut iter = bucket.iter();
            let Some(first) = iter.next() else {
                return Ok(unsafe { LLVMConstInt(bool_type, 0, 0) });
            };
            let mut acc = lower_logical(&first, builder, context, args_ptr, variable_slots)?;
            for term in iter {
                let rhs = lower_logical(&term, builder, context, args_ptr, variable_slots)?;
                let name = cstring("ortmp")?;
                acc = unsafe { LLVMBuildOr(builder, acc, rhs, name.as_ptr()) };
            }
            Ok(acc)
        }
        LogicalExpression::Relation { left, operator, right } => {
            let double_type = unsafe { LLVMDoubleTypeInContext(context) };
            let lhs = lower_numeric(
                left,
                builder,
                context,
                double_type,
                args_ptr,
                variable_slots,
                ptr::null_mut(),
            )?;
            let rhs = lower_numeric(
                right,
                builder,
                context,
                double_type,
                args_ptr,
                variable_slots,
                ptr::null_mut(),
            )?;
            let predicate = real_predicate(operator)?;
            let name = cstring("cmptmp")?;
            Ok(unsafe { LLVMBuildFCmp(builder, predicate, lhs, rhs, name.as_ptr()) })
        }
    }
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
    let name = cstring("exprion_runtime_pow")?;
    Ok(unsafe { LLVMAddFunction(module, name.as_ptr(), function_type) })
}

extern "C" fn exprion_runtime_pow(base: f64, exponent: f64) -> f64 {
    base.powf(exponent)
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

fn cstring(value: &str) -> Result<CString, JitError> {
    CString::new(value)
        .map_err(|_| JitError::Codegen(format!("string contains interior null byte: {value:?}")))
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

    fn LLVMCreateBuilderInContext(C: LLVMContextRef) -> LLVMBuilderRef;
    fn LLVMPositionBuilderAtEnd(Builder: LLVMBuilderRef, Block: LLVMBasicBlockRef);
    fn LLVMDisposeBuilder(Builder: LLVMBuilderRef);

    fn LLVMConstReal(RealTy: LLVMTypeRef, N: f64) -> LLVMValueRef;
    fn LLVMConstInt(IntTy: LLVMTypeRef, N: u64, SignExtend: c_uint) -> LLVMValueRef;

    fn LLVMBuildRet(Builder: LLVMBuilderRef, V: LLVMValueRef) -> LLVMValueRef;
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
