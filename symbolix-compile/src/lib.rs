use std::panic::{catch_unwind, AssertUnwindSafe};

use proc_macro::TokenStream;

mod cache;
mod codegen;
mod expand;
mod rust_expr;

use crate::cache::expand_with_cache;
use crate::expand::{
    compile_formula, compile_symbolix, normalize_formula_input, normalize_symbolix_input,
    panic_to_compile_error,
};
use symbolix_core::{equation::SolutionSet, semantic::semantic_ir::SemanticExpression};

#[derive(Debug, Clone)]
pub(crate) enum CompileValue {
    Semantic(SemanticExpression),
    SolutionSet(SolutionSet),
}

#[proc_macro]
pub fn formula(input: TokenStream) -> TokenStream {
    match catch_unwind(AssertUnwindSafe(|| {
        let normalized_input = match normalize_formula_input(input.clone()) {
            Ok(normalized) => normalized,
            Err(err) => return err.to_compile_error().into(),
        };

        expand_with_cache("formula", &normalized_input, || compile_formula(input))
            .unwrap_or_else(|err| err.to_compile_error().into())
    })) {
        Ok(tokens) => tokens,
        Err(payload) => panic_to_compile_error(payload),
    }
}

/// 规范一下宏里面的内容
/// 1. 变量定义：var!("var_name")
/// 2. 表达式定义：expr!("expr")
/// 3. 表达式调用方法创建关系：greater_than、less_than、equals、not_equals、greater_equal、less_equal
/// 4. 表达式运算
/// 5. 方程求解创建表达式
/// 6. if 分支语句
/// 7. 只有表达式或变量可以作为返回值，必须有返回值，返回值可以是元组
#[proc_macro]
pub fn symbolix(input: TokenStream) -> TokenStream {
    match catch_unwind(AssertUnwindSafe(|| {
        let normalized_input = normalize_symbolix_input(input.clone());
        expand_with_cache("symbolix", &normalized_input, || compile_symbolix(input))
            .unwrap_or_else(|err| err.to_compile_error().into())
    })) {
        Ok(tokens) => tokens,
        Err(payload) => panic_to_compile_error(payload),
    }
}
