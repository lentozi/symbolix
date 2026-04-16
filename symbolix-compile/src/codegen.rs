use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use symbolix_core::equation::{BranchResult, SolutionSet};
use symbolix_core::lexer::symbol::{Relation, Symbol};
use symbolix_core::semantic::semantic_ir::{
    logic::LogicalExpression, numeric::NumericExpression, SemanticExpression,
};
use symbolix_core::semantic::variable::{Variable, VariableType};
use crate::CompileValue;

pub fn get_func_arguments(values: &[CompileValue]) -> (Vec<Ident>, Vec<TokenStream>) {
    let mut variables = Vec::new();
    for value in values {
        collect_value_variables(value, &mut variables);
    }
    variables.sort_by(|a, b| a.name.cmp(&b.name));
    variables.dedup_by(|a, b| a.name == b.name);

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

    (var_names, var_types)
}

fn collect_value_variables(value: &CompileValue, variables: &mut Vec<Variable>) {
    match value {
        CompileValue::Semantic(expr) => collect_semantic_variables(expr, variables),
        CompileValue::Variable(variable) => push_variable(variable, variables),
        CompileValue::SolutionSet(solution_set) => collect_solution_set_variables(solution_set, variables),
    }
}

fn collect_semantic_variables(expr: &SemanticExpression, variables: &mut Vec<Variable>) {
    match expr {
        SemanticExpression::Numeric(numeric) => collect_numeric_variables(numeric, variables),
        SemanticExpression::Logical(logical) => collect_logical_variables(logical, variables),
    }
}

fn collect_numeric_variables(expr: &NumericExpression, variables: &mut Vec<Variable>) {
    match expr {
        NumericExpression::Constant(_) => {}
        NumericExpression::Variable(variable) => push_variable(variable, variables),
        NumericExpression::Negation(inner) => collect_numeric_variables(inner, variables),
        NumericExpression::Addition(bucket) | NumericExpression::Multiplication(bucket) => {
            for expr in bucket.iter() {
                collect_numeric_variables(&expr, variables);
            }
        }
        NumericExpression::Power { base, exponent } => {
            collect_numeric_variables(base, variables);
            collect_numeric_variables(exponent, variables);
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            for (condition, expr) in cases {
                collect_logical_variables(condition, variables);
                collect_numeric_variables(expr, variables);
            }
            if let Some(expr) = otherwise {
                collect_numeric_variables(expr, variables);
            }
        }
    }
}

fn collect_logical_variables(expr: &LogicalExpression, variables: &mut Vec<Variable>) {
    match expr {
        LogicalExpression::Constant(_) => {}
        LogicalExpression::Variable(variable) => push_variable(variable, variables),
        LogicalExpression::Not(inner) => collect_logical_variables(inner, variables),
        LogicalExpression::And(bucket) | LogicalExpression::Or(bucket) => {
            for expr in bucket.iter() {
                collect_logical_variables(&expr, variables);
            }
        }
        LogicalExpression::Relation { left, right, .. } => {
            collect_numeric_variables(left, variables);
            collect_numeric_variables(right, variables);
        }
    }
}

fn collect_solution_set_variables(solution_set: &SolutionSet, variables: &mut Vec<Variable>) {
    for branch in &solution_set.branches {
        collect_logical_variables(&branch.constraint, variables);
        if let BranchResult::Finite(solutions) = &branch.result {
            for solution in solutions {
                collect_numeric_variables(solution, variables);
            }
        }
    }
}

fn push_variable(variable: &Variable, variables: &mut Vec<Variable>) {
    if !variables.iter().any(|existing| existing.name == variable.name) {
        variables.push(variable.clone());
    }
}

pub fn get_func_return_type(value: &CompileValue) -> TokenStream {
    match value {
        CompileValue::Semantic(expr) => {
            if expr.is_numeric() {
                quote! { f64 }
            } else {
                quote! { bool }
            }
        }
        CompileValue::Variable(variable) => match variable.var_type {
            VariableType::Float | VariableType::Fraction => quote! { f64 },
            VariableType::Integer => quote! { i32 },
            VariableType::Boolean => quote! { bool },
            VariableType::Unknown => panic!("invalid variable type"),
        },
        CompileValue::SolutionSet(solution_set) => solution_set_return_type(solution_set),
    }
}

pub fn multi_codegen_values(
    expr_list: &Vec<CompileValue>,
    name_list: &Vec<Ident>,
) -> TokenStream {
    let code_list = expr_list.iter().map(codegen_value).collect::<Vec<_>>();

    let lets = name_list
        .iter()
        .zip(code_list.iter())
        .map(|(name, code)| {
            quote! {
                let #name = #code;
            }
        });

    quote! {
        #(#lets)*
        (#(#name_list),*)
    }
}

pub fn codegen_value(value: &CompileValue) -> TokenStream {
    match value {
        CompileValue::Semantic(expr) => codegen_semantic(expr),
        CompileValue::Variable(variable) => codegen_semantic(&variable.as_expression()),
        CompileValue::SolutionSet(solution_set) => codegen_solution_set(solution_set),
    }
}

pub fn codegen_semantic(expr: &SemanticExpression) -> TokenStream {
    match expr {
        SemanticExpression::Numeric(n) => codegen_numeric(n),
        SemanticExpression::Logical(l) => codegen_logical(l),
    }
}

fn codegen_solution_set(solution_set: &SolutionSet) -> TokenStream {
    let shape = solution_set_shape(solution_set);
    let branches = solution_set
        .branches
        .iter()
        .map(|branch| {
            let constraint = codegen_logical(&branch.constraint);
            let value = match (&shape, &branch.result) {
                (SolutionSetShape::Scalar, BranchResult::Finite(solutions)) => {
                    codegen_numeric(&solutions[0])
                }
                (SolutionSetShape::Vector, BranchResult::Finite(solutions)) => {
                    let values = solutions.iter().map(codegen_numeric).collect::<Vec<_>>();
                    quote! { vec![#(#values),*] }
                }
                (_, BranchResult::Identity) => unreachable!("identity branches are rejected earlier"),
            };
            (constraint, value)
        })
        .collect::<Vec<_>>();

    if branches.is_empty() {
        return match shape {
            SolutionSetShape::Scalar => {
                quote! { panic!("solution set has no branches and cannot lower to a scalar") }
            }
            SolutionSetShape::Vector => quote! { Vec::<f64>::new() },
        };
    }

    let else_block = match shape {
        SolutionSetShape::Scalar => {
            quote! { panic!("no condition in solution set matched for scalar result") }
        }
        SolutionSetShape::Vector => quote! { Vec::<f64>::new() },
    };

    let mut stream = else_block;
    for (constraint, value) in branches.into_iter().rev() {
        stream = quote! {
            if #constraint {
                #value
            } else {
                #stream
            }
        };
    }

    stream
}

enum SolutionSetShape {
    Scalar,
    Vector,
}

fn solution_set_shape(solution_set: &SolutionSet) -> SolutionSetShape {
    let mut saw_non_singleton = solution_set.branches.is_empty();
    for branch in &solution_set.branches {
        match &branch.result {
            BranchResult::Finite(solutions) => {
                if solutions.len() != 1 {
                    saw_non_singleton = true;
                }
            }
            BranchResult::Identity => {
                panic!(
                    "symbolix! cannot lower solution sets with identity branches to runtime values"
                );
            }
        }
    }

    if saw_non_singleton {
        SolutionSetShape::Vector
    } else {
        SolutionSetShape::Scalar
    }
}

fn solution_set_return_type(solution_set: &SolutionSet) -> TokenStream {
    match solution_set_shape(solution_set) {
        SolutionSetShape::Scalar => quote! { f64 },
        SolutionSetShape::Vector => quote! { Vec<f64> },
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
                terms.push(quote! { (#val) });
            }
            for v in &bucket.variables {
                let name = format_ident!("{}", v.name);
                terms.push(quote! { (#name) });
            }
            for e in &bucket.expressions {
                let expr = codegen_numeric(e);
                terms.push(quote! { (#expr) });
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
                terms.push(quote! { (#val) });
            }
            for v in &bucket.variables {
                let name = format_ident!("{}", v.name);
                terms.push(quote! { (#name) });
            }
            for e in &bucket.expressions {
                let expr = codegen_numeric(e);
                terms.push(quote! { (#expr) });
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
                terms.push(quote! { (#c) });
            }
            for v in &bucket.variables {
                let name = format_ident!("{}", v.name);
                terms.push(quote! { (#name) });
            }
            for e in &bucket.expressions {
                let expr = codegen_logical(e);
                terms.push(quote! { (#expr) });
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
                terms.push(quote! { (#c) });
            }
            for v in &bucket.variables {
                let name = format_ident!("{}", v.name);
                terms.push(quote! { (#name) });
            }
            for e in &bucket.expressions {
                let expr = codegen_logical(e);
                terms.push(quote! { (#expr) });
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

pub fn generate_struct(
    var_names: Vec<Ident>,
    var_types: Vec<TokenStream>,
    return_type: TokenStream,
    code: TokenStream) -> TokenStream {

    let doc_comment = format!(
        "Compiled Formula\n\nArguments in order: ({})",
        var_names
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );

    quote! {
        (|| {
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
        })()
    }.into()
}


