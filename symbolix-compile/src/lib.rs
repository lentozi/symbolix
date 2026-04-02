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
use syn::{parse_macro_input, Expr, LitStr};

use crate::{
    codegen::codegen_semantic,
    rust_expr::{convert_expr, convert_expr_},
};

mod codegen;
mod rust_expr;

use quote::{format_ident, quote};
use symbolix_core::semantic::Analyzer;
use crate::codegen::{get_func_arguments, get_func_return_type};

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

        let doc_comment = format!(
            "Compiled Formula\n\nArguments in order: ({})",
            var_names
                .iter()
                .map(|v| v.to_string())
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

                    expr_table.insert(var_name, expr);
                }
                syn::Stmt::Item(_) => unreachable!(),
                // expr 只能作为返回值出现，可能是普通表达式，可能是元组
                syn::Stmt::Expr(expr, semi) => {
                    if semi.is_some() {
                        panic!("unexpected ';'");
                    }

                    let (code, return_type): (proc_macro2::TokenStream, proc_macro2::TokenStream) = match expr {
                        Expr::Tuple(tuple_expr) => {
                            let expr_list = tuple_expr.elems.iter().map(|x| {
                                convert_expr(x, &mut expr_table)
                            }).collect::<Vec<_>>();

                            let code_list = expr_list.iter().map(codegen_semantic).collect::<Vec<_>>();

                            let return_name_list = tuple_expr.elems.iter().enumerate().map(|(i, x)| {
                                match x {
                                    Expr::Path(path) => path.path.get_ident().unwrap().clone(),
                                    _ => format_ident!("_{}", i),
                                }
                            }).collect::<Vec<_>>();

                            let lets = return_name_list.iter().zip(code_list.iter()).map(|(name, code)| {
                                quote! {
                                    let #name = #code;
                                }
                            });

                            let code = quote! {
                                #(#lets)*

                                (#(#return_name_list),*)
                            };

                            let return_types = expr_list.iter().map(get_func_return_type).collect::<Vec<_>>();

                            let return_type = quote! {
                                ( #(#return_types),* )
                            };

                            (code, return_type)
                        }
                        _ => {
                            let expr = convert_expr(expr, &mut expr_table);
                            let code = codegen_semantic(&expr);

                            let return_type = get_func_return_type(&expr);

                            (code, return_type)
                        }
                    };

                    let (var_names, var_types) = get_func_arguments();

                    let doc_comment = format!(
                        "Compiled Formula\n\nArguments in order: ({})",
                        var_names
                            .iter()
                            .map(|v| v.to_string())
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

                    return expanded.into();
                },
                _ => {}
            }
        }

        panic!("need return values");

    }
}
