use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use symbolix_core::{
    lexer::Lexer, new_compile_context, optimizer::optimize, parser::Parser,
    equation::SolutionSet,
    semantic::semantic_ir::SemanticExpression,
};
use syn::{parse_macro_input, LitStr};

use crate::codegen::codegen_semantic;

mod codegen;
mod rust_expr;

use crate::codegen::{
    codegen_value, generate_struct, get_func_arguments, get_func_return_type, multi_codegen_values,
};
use crate::rust_expr::convert_block;
use symbolix_core::semantic::Analyzer;

#[derive(Debug, Clone)]
pub(crate) enum CompileValue {
    Semantic(SemanticExpression),
    SolutionSet(SolutionSet),
}

#[proc_macro]
pub fn formula(input: TokenStream) -> TokenStream {
    match catch_unwind(AssertUnwindSafe(|| {
        new_compile_context! {
            let input_lit = parse_macro_input!(input as LitStr);
            let expr_str = input_lit.value();

            let mut lexer = Lexer::new(&expr_str);
            let expression = Parser::pratt(&mut lexer);
            let mut analyzer = Analyzer::new();
            let mut semantic_expression = analyzer.analyze_with_ctx(&expression);
            optimize(&mut semantic_expression);
            let code = codegen_semantic(&semantic_expression);

            let compiled_value = CompileValue::Semantic(semantic_expression.clone());
            let (var_names, var_types) = get_func_arguments(&[compiled_value.clone()]);
            let return_type = get_func_return_type(&compiled_value);

            generate_struct(var_names, var_types, return_type, code).into()
        }
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
        let result: syn::Result<TokenStream> = new_compile_context! {
            let input: proc_macro2::TokenStream = input.into();

            let wrapped = quote::quote!({
                #input
            });

            let block: syn::Block = syn::parse2(wrapped).unwrap();

            let mut expr_table: HashMap<String, CompileValue> = HashMap::new();
            let (expr_list, return_name_list): (Vec<CompileValue>, Vec<Ident>) =
                convert_block(&block, &mut expr_table)?;

            let (var_names, var_types) = get_func_arguments(&expr_list);

            let (code, return_type): (proc_macro2::TokenStream, proc_macro2::TokenStream) = if expr_list.len() == 1 {
                let expr = expr_list.into_iter().next().unwrap();
                let code = codegen_value(&expr);

                let return_type = get_func_return_type(&expr);
                (code, return_type)
            } else {
                let code = multi_codegen_values(&expr_list, &return_name_list);

                let return_types = expr_list
                        .iter()
                        .map(get_func_return_type)
                        .collect::<Vec<_>>();

                let return_type = quote! {
                    ( #(#return_types),* )
                };

                (code, return_type)
            };

            Ok(generate_struct(var_names, var_types, return_type, code).into())
        };

        result.unwrap_or_else(|err| err.to_compile_error().into())
    })) {
        Ok(tokens) => tokens,
        Err(payload) => panic_to_compile_error(payload),
    }
}

fn panic_to_compile_error(payload: Box<dyn std::any::Any + Send>) -> TokenStream {
    let message = if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else {
        "symbolix-compile internal error".to_string()
    };

    quote! {
        compile_error!(#message);
    }
    .into()
}

