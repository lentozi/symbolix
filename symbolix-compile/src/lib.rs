use std::collections::HashMap;

use proc_macro::TokenStream;
use symbolix_core::{
    lexer::Lexer,
    new_compile_context,
    optimizer::optimize,
    parser::Parser,
    semantic::{
        semantic_ir::SemanticExpression,
        variable::{Variable, VariableType},
    },
    with_compile_context,
};
use syn::{parse_macro_input, LitStr};

use crate::{
    codegen::codegen_semantic,
    rust_expr::{convert_expr, convert_expr_},
};

mod codegen;
mod rust_expr;

use quote::quote;
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

        // 获取上下文中的变量
        let mut variables = with_compile_context!(ctx, ctx.collect_variables());
        variables.sort_by(|a, b| a.name.cmp(&b.name));

        let var_names: Vec<_> = variables
            .iter()
            .map(|variable| syn::Ident::new(&variable.name, proc_macro2::Span::call_site()))
            .collect();

        let var_types: Vec<_> = variables
            .iter()
            .map(|variable| match variable.var_type {
                VariableType::Float | VariableType::Fraction => quote! { f64 },
                VariableType::Integer => quote! { i32 },
                VariableType::Boolean => quote! { bool },
                _ => panic!("invalid variable type"),
            })
            .collect();

        let return_type = if analyzer.is_numeric() {
            quote! { f64 }
        } else {
            quote! { bool }
        };

        let doc_comment = format!(
            "Compiled Formula\n\nArguments in order: ({})",
            variables
                .iter()
                .map(|v| v.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );

        let expanded = quote! {
            {
                #[derive(Clone, Copy)]
                #[doc = #doc_comment]
                struct CompiledFormula;

                impl CompiledFormula {
                    pub fn calculate(&self, #(#var_names: #var_types),*) -> #return_type {
                        #code
                    }

                    pub fn to_closure(&self) -> Box<dyn Fn(#(#var_types),*) -> #return_type> {
                        #[doc = #doc_comment]
                        Box::new(|#(#var_names: #var_types),*| -> #return_type {
                            #code
                        })
                    }
                }

                CompiledFormula
            }
        };

        expanded.into()
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

        for stmt in &block.stmts {
            // println!("{:#?}", stmt);
            match stmt {
                // let 赋值
                syn::Stmt::Local(local) => {
                    // println!("{:#?}", local);
                    let pat = local.pat.clone();

                    let var_name = match &pat {
                        syn::Pat::Ident(ident) => ident.ident.to_string(),
                        _ => panic!("invalid pat"),
                    };

                    // expr 是等号右侧的元数据
                    let expr_token = local.init.as_ref().unwrap().clone();

                    // 右侧可能出现的：宏调用、方法调用、二元表达式
                    let expr = convert_expr(expr_token.expr.as_ref(), &mut expr_table);
                    // convert_expr(expr.expr.as_ref(), &mut expr_table);
                    // println!("{:#?}", expr);

                    expr_table.insert(var_name, expr);
                }
                syn::Stmt::Item(_) => unreachable!(),
                // expr 只能作为返回值出现
                syn::Stmt::Expr(expr, semi) => println!("expression: {:#?}, {:#?}", expr, semi),
                _ => {}
            }
        }

        TokenStream::new()
    }
}
