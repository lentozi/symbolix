use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::Ident;
use symbolix_core::{
    lexer::Lexer, new_compile_context, optimizer::optimize, parser::Parser,
    semantic::semantic_ir::SemanticExpression,
};
use syn::{parse_macro_input, Expr, LitStr};

use crate::{codegen::codegen_semantic, rust_expr::convert_expr};

mod codegen;
mod rust_expr;

use crate::codegen::{generate_struct, get_func_arguments, get_func_return_type, multi_codegen_semantic};
use crate::rust_expr::convert_block;
use quote::{format_ident, quote};
use symbolix_core::semantic::Analyzer;

#[proc_macro]
pub fn compile(input: TokenStream) -> TokenStream {
    new_compile_context! {
        // 将输入转化为字符串
        let input_lit = parse_macro_input!(input as LitStr);
        let expr_str = input_lit.value();

        let mut lexer = Lexer::new(&expr_str);
        let expression = Parser::pratt(&mut lexer);
        let mut analyzer = Analyzer::new();
        let mut semantic_expression = analyzer.analyze_with_ctx(&expression);
        optimize(&mut semantic_expression);
        let code = codegen_semantic(&semantic_expression);

        let (var_names, var_types) = get_func_arguments();
        let return_type = get_func_return_type(&semantic_expression);

        generate_struct(var_names, var_types, return_type, code).into()
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
pub fn symbolix_rust(input: TokenStream) -> TokenStream {
    new_compile_context! {

        let input: proc_macro2::TokenStream = input.into();

        let wrapped = quote::quote!({
            #input
        });

        let block: syn::Block = syn::parse2(wrapped).unwrap();

        // 创建变量表和表达式表
        let mut expr_table: HashMap<String, SemanticExpression> = HashMap::new();
        let (expr_list, return_name_list): (Vec<SemanticExpression>, Vec<Ident>) = convert_block(&block, &mut expr_table);

        let (code, return_type): (proc_macro2::TokenStream, proc_macro2::TokenStream) = if expr_list.len() == 1 {
            let expr = expr_list.into_iter().next().unwrap();
            let code = codegen_semantic(&expr);

            let return_type = get_func_return_type(&expr);
            (code, return_type)
        } else {
            let code = multi_codegen_semantic(&expr_list, &return_name_list);

            let return_types = expr_list
                    .iter()
                    .map(get_func_return_type)
                    .collect::<Vec<_>>();

            let return_type = quote! {
                ( #(#return_types),* )
            };

            (code, return_type)
        };

        let (var_names, var_types) = get_func_arguments();

        generate_struct(var_names, var_types, return_type, code).into()
    }
}
