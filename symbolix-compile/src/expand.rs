use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use symbolix_core::{
    lexer::Lexer, new_compile_context, optimizer::optimize, parser::Parser,
};
use syn::{spanned::Spanned, LitStr};

use crate::codegen::{
    codegen_semantic, codegen_value, generate_struct, get_func_arguments, get_func_return_type,
    multi_codegen_values,
};
use crate::rust_expr::convert_block;
use crate::CompileValue;
use symbolix_core::semantic::Analyzer;

pub(crate) fn compile_formula(input: TokenStream) -> syn::Result<TokenStream> {
    new_compile_context! {
        let input_lit = syn::parse::<LitStr>(input)?;
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

        Ok(generate_struct(var_names, var_types, return_type, code).into())
    }
}

pub(crate) fn compile_symbolix(input: TokenStream) -> syn::Result<TokenStream> {
    new_compile_context! {
        let input: TokenStream2 = input.into();

        let wrapped = quote!({
            #input
        });

        let block: syn::Block = syn::parse2(wrapped)?;

        let mut expr_table: HashMap<String, CompileValue> = HashMap::new();
        let (expr_list, return_name_list): (Vec<CompileValue>, Vec<Ident>) =
            convert_block(&block, &mut expr_table)?;

        let (var_names, var_types) = get_func_arguments(&expr_list);

        let (code, return_type): (TokenStream2, TokenStream2) = if expr_list.len() == 1 {
            let expr = expr_list.into_iter().next().ok_or_else(|| {
                syn::Error::new(
                    block.span(),
                    "symbolix! block must end with a return expression",
                )
            })?;
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
    }
}

pub(crate) fn normalize_formula_input(input: TokenStream) -> syn::Result<String> {
    let input_lit = syn::parse::<LitStr>(input)?;
    Ok(input_lit.value())
}

pub(crate) fn normalize_symbolix_input(input: TokenStream) -> String {
    let input: TokenStream2 = input.into();
    input.to_string()
}

pub(crate) fn panic_to_compile_error(payload: Box<dyn std::any::Any + Send>) -> TokenStream {
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
