use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use symbolix_core::lexer::symbol::{Relation, Symbol};

use symbolix_core::semantic::semantic_ir::{
    logic::LogicalExpression, numeric::NumericExpression, SemanticExpression,
};

pub fn codegen_semantic(expr: &SemanticExpression) -> TokenStream {
    match expr {
        SemanticExpression::Numeric(n) => codegen_numeric(n),
        SemanticExpression::Logical(l) => codegen_logical(l),
    }
}

pub fn codegen_numeric(expr: &NumericExpression) -> TokenStream {
    match expr {
        NumericExpression::Constant(n) => {
            let val = n.to_float();
            quote! { #val }
        }
        NumericExpression::Variable(v) => {
            let name = format_ident!("{}", v.name);
            quote! { #name }
        }
        NumericExpression::Negation(inner) => {
            let inner_code = codegen_numeric(inner);
            quote! { (-#inner_code) }
        }
        NumericExpression::Addition(bucket) => {
            let mut terms = Vec::new();
            for c in &bucket.constants {
                let val = c.to_float();
                terms.push(quote! { #val });
            }
            for v in &bucket.variables {
                let name = format_ident!("{}", v.name);
                terms.push(quote! { #name });
            }
            for e in &bucket.expressions {
                terms.push(codegen_numeric(e));
            }

            if terms.is_empty() {
                quote! { 0.0 }
            } else {
                quote! { #(#terms)+* }
            }
        }
        NumericExpression::Multiplication(bucket) => {
            let mut terms = Vec::new();
            for c in &bucket.constants {
                let val = c.to_float();
                terms.push(quote! { #val });
            }
            for v in &bucket.variables {
                let name = format_ident!("{}", v.name);
                terms.push(quote! { #name });
            }
            for e in &bucket.expressions {
                terms.push(codegen_numeric(e));
            }

            if terms.is_empty() {
                quote! { 1.0 }
            } else {
                quote! { 1.0 #(* #terms)* }
            }
        }
        NumericExpression::Power { base, exponent } => {
            let b = codegen_numeric(base);
            let e = codegen_numeric(exponent);
            quote! { f64::powf(#b, #e) }
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            let else_block = if let Some(other) = otherwise {
                codegen_numeric(other)
            } else {
                quote! { f64::NAN }
            };

            let mut stream = else_block;

            // Iterate in reverse to wrap: if c1 { v1 } else { if c2 { v2 } else { ... } }
            for (cond, val) in cases.iter().rev() {
                let val_code = codegen_numeric(val);
                let cond_code = codegen_logical(cond);

                stream = quote! {
                    if #cond_code {
                        #val_code
                    } else {
                        #stream
                    }
                };
            }
            stream
        }
    }
}

pub fn codegen_logical(expr: &LogicalExpression) -> TokenStream {
    match expr {
        LogicalExpression::Constant(c) => {
            quote! { #c }
        }
        LogicalExpression::Variable(v) => {
            let name = syn::Ident::new(&v.name, proc_macro2::Span::call_site());
            quote! { #name }
        }
        LogicalExpression::Not(inner) => {
            let inner_code = codegen_logical(inner);
            quote! { !#inner_code }
        }
        LogicalExpression::And(bucket) => {
            let mut terms = Vec::new();
            for c in &bucket.constants {
                terms.push(quote! { #c });
            }
            for v in &bucket.variables {
                let name = format_ident!("{}", v.name);
                terms.push(quote! { #name });
            }
            for e in &bucket.expressions {
                terms.push(codegen_logical(e));
            }

            if terms.is_empty() {
                quote! { true }
            } else {
                quote! { #(#terms)&&* }
            }
        }
        LogicalExpression::Or(bucket) => {
            let mut terms = Vec::new();
            for c in &bucket.constants {
                terms.push(quote! { #c });
            }
            for v in &bucket.variables {
                let name = format_ident!("{}", v.name);
                terms.push(quote! { #name });
            }
            for e in &bucket.expressions {
                terms.push(codegen_logical(e));
            }

            if terms.is_empty() {
                quote! { false }
            } else {
                quote! { #(#terms)||* }
            }
        }
        LogicalExpression::Relation {
            left,
            operator,
            right,
        } => {
            let l = codegen_numeric(left);
            let r = codegen_numeric(right);
            match operator {
                Symbol::Relation(rel) => match rel {
                    Relation::Equal => quote! { #l == #r },
                    Relation::NotEqual => quote! { #l != #r },
                    Relation::LessThan => quote! { #l < #r },
                    Relation::GreaterThan => quote! { #l > #r },
                    Relation::LessEqual => quote! { #l <= #r },
                    Relation::GreaterEqual => quote! { #l >= #r },
                },
                _ => quote! { compile_error!("Unsupported relation operator") },
            }
        }
    }
}

#[test]
fn test_codegen_arithmetic() {
    use symbolix_core::context::compile::CompileContext;
    use symbolix_core::lexer::symbol::Precedence;
    use symbolix_core::lexer::Lexer;
    use symbolix_core::optimizer::optimize;
    use symbolix_core::parser::expression::Expression;
    use symbolix_core::parser::pratt_parsing;
    use symbolix_core::semantic::semantic_without_ctx;
    use symbolix_core::semantic::variable::VariableType;

    let mut ctx = CompileContext::new();
    let input = "-x + y + 123 + 45.67 * ((89 - 0.1) ^ x) ^ x + 0";
    let mut lexer: Lexer = Lexer::new(input);
    let expression: Expression = pratt_parsing(&mut lexer, Precedence::Lowest);
    let mut semantic = semantic_without_ctx(&expression, true, &mut ctx);
    optimize(&mut semantic);
    let code = codegen_semantic(&semantic);
    println!("{}", code);

    let variables = ctx.collect_variables();
    let var_names = variables
        .iter()
        .map(|variable| syn::Ident::new(&variable.name, proc_macro2::Span::call_site()));
    let var_types = variables.iter().map(|variable| match variable.var_type {
        VariableType::Float | VariableType::Fraction | VariableType::Integer => quote! { f64 },
        VariableType::Boolean => quote! { bool },
        _ => panic!("invalid variable type"),
    });

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
            struct CompiledFormula;

            impl CompiledFormula {
                #[doc = #doc_comment]
                pub fn calculate(&self, #(#var_names: #var_types),*) -> f64 {
                    #code
                }
            }

            CompiledFormula
        }
    };

    println!("{}", expanded.to_string());
}
