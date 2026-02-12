use proc_macro::TokenStream;
use symbolix_core::{
    context::compile::CompileContext,
    lexer::{symbol::Precedence, Lexer},
    new_compile_context,
    optimizer::optimize,
    parser::pratt_parsing,
    semantic::{semantic_without_ctx, variable::VariableType},
    with_compile_context,
};
use syn::{parse_macro_input, LitStr};

use crate::codegen::codegen_semantic;

mod codegen;

use quote::quote;

#[proc_macro]
pub fn compile(input: TokenStream) -> TokenStream {
    new_compile_context! {
        let input_lit = parse_macro_input!(input as LitStr);
        let expr_str = input_lit.value();

        let mut lexer = Lexer::new(&expr_str);
        let expression = pratt_parsing(&mut lexer, Precedence::Lowest);
        let mut semantic_expression = semantic_without_ctx(&expression, true);
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
                VariableType::Float | VariableType::Fraction | VariableType::Integer => quote! { f64 },
                VariableType::Boolean => quote! { bool },
                _ => panic!("invalid variable type"),
            })
            .collect();

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
                    pub fn calculate(&self, #(#var_names: #var_types),*) -> f64 {
                        #code
                    }

                    pub fn to_closure(&self) -> Box<dyn Fn(#(#var_types),*) -> f64> {
                        #[doc = #doc_comment]
                        Box::new(|#(#var_names: #var_types),*| -> f64 {
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
