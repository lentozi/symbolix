use std::{
    collections::HashMap,
    ffi::{c_char, c_uint, c_void, CString},
    mem,
    ptr,
    sync::Once,
};

use exprion_core::{
    semantic::semantic_ir::numeric::NumericExpression,
};

use crate::JitError;

type JitEntry = unsafe extern "C" fn(*const f64) -> f64;

type LLVMContextRef = *mut c_void;
type LLVMModuleRef = *mut c_void;
type LLVMBuilderRef = *mut c_void;
type LLVMTypeRef = *mut c_void;
type LLVMValueRef = *mut c_void;
type LLVMBasicBlockRef = *mut c_void;
type LLVMExecutionEngineRef = *mut c_void;

pub(crate) struct LlvmNumericKernel {
    _execution_engine: LLVMExecutionEngineRef,
    entry: JitEntry,
}

impl LlvmNumericKernel {
    pub(crate) fn compile(
        expr: &NumericExpression,
        variable_names: &[String],
    ) -> Result<Self, JitError> {
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

            let block_name = cstring("entry")?;
            let entry_block =
                unsafe { LLVMAppendBasicBlockInContext(context, function, block_name.as_ptr()) };
            unsafe { LLVMPositionBuilderAtEnd(builder, entry_block) };

            let args_ptr = unsafe { LLVMGetParam(function, 0) };
            let variable_slots = variable_names
                .iter()
                .enumerate()
                .map(|(index, name)| (name.clone(), index))
                .collect::<HashMap<_, _>>();

            let return_value =
                lower_numeric(expr, builder, double_type, args_ptr, &variable_slots)?;
            unsafe {
                LLVMBuildRet(builder, return_value);
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

            let entry = unsafe { mem::transmute::<usize, JitEntry>(address as usize) };
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

    pub(crate) fn call(&self, arguments: &[f64]) -> f64 {
        unsafe { (self.entry)(arguments.as_ptr()) }
    }
}

fn lower_numeric(
    expr: &NumericExpression,
    builder: LLVMBuilderRef,
    double_type: LLVMTypeRef,
    args_ptr: LLVMValueRef,
    variable_slots: &HashMap<String, usize>,
) -> Result<LLVMValueRef, JitError> {
    match expr {
        NumericExpression::Constant(number) => {
            Ok(unsafe { LLVMConstReal(double_type, number.to_float()) })
        }
        NumericExpression::Variable(variable) => {
            let index = variable_slots.get(&variable.name).ok_or_else(|| {
                JitError::Codegen(format!("missing variable slot for `{}`", variable.name))
            })?;

            let index_value =
                unsafe { LLVMConstInt(LLVMInt64Type(), *index as u64, 0) };
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
            let value = lower_numeric(inner, builder, double_type, args_ptr, variable_slots)?;
            let name = cstring("neg")?;
            Ok(unsafe { LLVMBuildFNeg(builder, value, name.as_ptr()) })
        }
        NumericExpression::Addition(bucket) => {
            let mut iter = bucket.iter();
            let Some(first) = iter.next() else {
                return Ok(unsafe { LLVMConstReal(double_type, 0.0) });
            };

            let mut acc = lower_numeric(&first, builder, double_type, args_ptr, variable_slots)?;
            for term in iter {
                let rhs = lower_numeric(&term, builder, double_type, args_ptr, variable_slots)?;
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

            let mut acc = lower_numeric(&first, builder, double_type, args_ptr, variable_slots)?;
            for term in iter {
                let rhs = lower_numeric(&term, builder, double_type, args_ptr, variable_slots)?;
                let name = cstring("multmp")?;
                acc = unsafe { LLVMBuildFMul(builder, acc, rhs, name.as_ptr()) };
            }
            Ok(acc)
        }
        NumericExpression::Power { .. } => Err(JitError::UnsupportedExpression(
            "LLVM JIT does not yet lower power expressions".to_string(),
        )),
        NumericExpression::Piecewise { .. } => Err(JitError::UnsupportedExpression(
            "LLVM JIT does not yet lower piecewise expressions".to_string(),
        )),
    }
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

type LLVMCodeModel = i32;
const LLVM_CODE_MODEL_DEFAULT: LLVMCodeModel = 0;
const LLVM_RETURN_STATUS_ACTION: i32 = 2;

#[link(name = "LLVM-C")]
extern "C" {
    fn LLVMContextCreate() -> LLVMContextRef;
    fn LLVMContextDispose(C: LLVMContextRef);

    fn LLVMModuleCreateWithNameInContext(ModuleID: *const c_char, C: LLVMContextRef)
    -> LLVMModuleRef;
    fn LLVMDisposeModule(M: LLVMModuleRef);

    fn LLVMDoubleTypeInContext(C: LLVMContextRef) -> LLVMTypeRef;
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
