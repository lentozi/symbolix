use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use symbolix_core::equation::{BranchResult, SolutionBranch, SolutionSet};
use symbolix_core::lexer::constant::Number;
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
        CompileValue::SolutionSet(_) => quote! { symbolix_core::equation::SolutionSet },
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
    let target = codegen_variable_literal(&solution_set.target);
    let branches = solution_set
        .branches
        .iter()
        .map(codegen_solution_branch)
        .collect::<Vec<_>>();

    quote! {
        symbolix_core::equation::SolutionSet {
            target: #target,
            branches: vec![#(#branches),*],
        }
    }
}

fn codegen_solution_branch(branch: &SolutionBranch) -> TokenStream {
    let constraint = codegen_logical_ast(&branch.constraint);
    let result = match &branch.result {
        BranchResult::Identity => {
            quote! { symbolix_core::equation::BranchResult::Identity }
        }
        BranchResult::Finite(solutions) => {
            let values = solutions.iter().map(codegen_numeric_ast).collect::<Vec<_>>();
            quote! { symbolix_core::equation::BranchResult::Finite(vec![#(#values),*]) }
        }
    };

    quote! {
        symbolix_core::equation::SolutionBranch {
            constraint: #constraint,
            result: #result,
        }
    }
}

fn codegen_variable_literal(variable: &Variable) -> TokenStream {
    let name = &variable.name;
    let var_type = match variable.var_type {
        VariableType::Integer => quote! { symbolix_core::semantic::variable::VariableType::Integer },
        VariableType::Float => quote! { symbolix_core::semantic::variable::VariableType::Float },
        VariableType::Fraction => quote! { symbolix_core::semantic::variable::VariableType::Fraction },
        VariableType::Boolean => quote! { symbolix_core::semantic::variable::VariableType::Boolean },
        VariableType::Unknown => quote! { symbolix_core::semantic::variable::VariableType::Unknown },
    };

    quote! {
        symbolix_core::semantic::variable::Variable {
            name: #name.to_string(),
            var_type: #var_type,
            value: None,
        }
    }
}

fn codegen_number_literal(number: &Number) -> TokenStream {
    match number {
        Number::Integer(value) => quote! { symbolix_core::lexer::constant::Number::integer(#value) },
        Number::Float(value) => {
            let inner = value.0;
            quote! { symbolix_core::lexer::constant::Number::float(#inner) }
        }
        Number::Fraction(frac) => {
            let numerator = frac.numerator;
            let denominator = frac.denominator;
            quote! { symbolix_core::lexer::constant::Number::fraction(#numerator, #denominator) }
        }
    }
}

fn codegen_numeric_ast(expr: &NumericExpression) -> TokenStream {
    match expr {
        NumericExpression::Constant(number) => {
            let number = codegen_number_literal(number);
            quote! { symbolix_core::semantic::semantic_ir::numeric::NumericExpression::constant(#number) }
        }
        NumericExpression::Variable(variable) => {
            let variable = codegen_variable_literal(variable);
            quote! { symbolix_core::semantic::semantic_ir::numeric::NumericExpression::variable(#variable) }
        }
        NumericExpression::Negation(inner) => {
            let inner = codegen_numeric_ast(inner);
            quote! { -#inner }
        }
        NumericExpression::Addition(bucket) => {
            let terms = bucket.iter().map(|expr| codegen_numeric_ast(&expr)).collect::<Vec<_>>();
            quote! {
                {
                    let mut iter = vec![#(#terms),*].into_iter();
                    let first = iter.next().unwrap_or_else(|| symbolix_core::semantic::semantic_ir::numeric::NumericExpression::constant(symbolix_core::lexer::constant::Number::integer(0)));
                    iter.fold(first, |acc, expr| acc + expr)
                }
            }
        }
        NumericExpression::Multiplication(bucket) => {
            let factors = bucket.iter().map(|expr| codegen_numeric_ast(&expr)).collect::<Vec<_>>();
            quote! {
                {
                    let mut iter = vec![#(#factors),*].into_iter();
                    let first = iter.next().unwrap_or_else(|| symbolix_core::semantic::semantic_ir::numeric::NumericExpression::constant(symbolix_core::lexer::constant::Number::integer(1)));
                    iter.fold(first, |acc, expr| acc * expr)
                }
            }
        }
        NumericExpression::Power { base, exponent } => {
            let base = codegen_numeric_ast(base);
            let exponent = codegen_numeric_ast(exponent);
            quote! {
                symbolix_core::semantic::semantic_ir::numeric::NumericExpression::power(&#base, &#exponent)
            }
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            let cases = cases
                .iter()
                .map(|(condition, expr)| {
                    let condition = codegen_logical_ast(condition);
                    let expr = codegen_numeric_ast(expr);
                    quote! { (#condition, #expr) }
                })
                .collect::<Vec<_>>();
            let otherwise = if let Some(expr) = otherwise {
                let expr = codegen_numeric_ast(expr);
                quote! { Some(#expr) }
            } else {
                quote! { None }
            };

            quote! {
                symbolix_core::semantic::semantic_ir::numeric::NumericExpression::piecewise(
                    vec![#(#cases),*],
                    #otherwise,
                )
            }
        }
    }
}

fn codegen_logical_ast(expr: &LogicalExpression) -> TokenStream {
    match expr {
        LogicalExpression::Constant(value) => {
            quote! { symbolix_core::semantic::semantic_ir::logic::LogicalExpression::constant(#value) }
        }
        LogicalExpression::Variable(variable) => {
            let variable = codegen_variable_literal(variable);
            quote! { symbolix_core::semantic::semantic_ir::logic::LogicalExpression::variable(#variable) }
        }
        LogicalExpression::Not(inner) => {
            let inner = codegen_logical_ast(inner);
            quote! { !#inner }
        }
        LogicalExpression::And(bucket) => {
            let terms = bucket.iter().map(|expr| codegen_logical_ast(&expr)).collect::<Vec<_>>();
            quote! {
                {
                    let mut iter = vec![#(#terms),*].into_iter();
                    let first = iter.next().unwrap_or_else(|| symbolix_core::semantic::semantic_ir::logic::LogicalExpression::constant(true));
                    iter.fold(first, |acc, expr| acc & expr)
                }
            }
        }
        LogicalExpression::Or(bucket) => {
            let terms = bucket.iter().map(|expr| codegen_logical_ast(&expr)).collect::<Vec<_>>();
            quote! {
                {
                    let mut iter = vec![#(#terms),*].into_iter();
                    let first = iter.next().unwrap_or_else(|| symbolix_core::semantic::semantic_ir::logic::LogicalExpression::constant(false));
                    iter.fold(first, |acc, expr| acc | expr)
                }
            }
        }
        LogicalExpression::Relation { left, operator, right } => {
            let left = codegen_numeric_ast(left);
            let right = codegen_numeric_ast(right);
            let operator = match operator {
                Symbol::Relation(Relation::Equal) => quote! { symbolix_core::lexer::symbol::Symbol::Relation(symbolix_core::lexer::symbol::Relation::Equal) },
                Symbol::Relation(Relation::NotEqual) => quote! { symbolix_core::lexer::symbol::Symbol::Relation(symbolix_core::lexer::symbol::Relation::NotEqual) },
                Symbol::Relation(Relation::LessThan) => quote! { symbolix_core::lexer::symbol::Symbol::Relation(symbolix_core::lexer::symbol::Relation::LessThan) },
                Symbol::Relation(Relation::GreaterThan) => quote! { symbolix_core::lexer::symbol::Symbol::Relation(symbolix_core::lexer::symbol::Relation::GreaterThan) },
                Symbol::Relation(Relation::LessEqual) => quote! { symbolix_core::lexer::symbol::Symbol::Relation(symbolix_core::lexer::symbol::Relation::LessEqual) },
                Symbol::Relation(Relation::GreaterEqual) => quote! { symbolix_core::lexer::symbol::Symbol::Relation(symbolix_core::lexer::symbol::Relation::GreaterEqual) },
                _ => unreachable!("solution constraints must be relation operators"),
            };
            quote! {
                symbolix_core::semantic::semantic_ir::logic::LogicalExpression::relation(&#left, &#operator, &#right)
            }
        }
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
    }.into()
}

#[test]
fn test_codegen_arithmetic() {
    use symbolix_core::lexer::Lexer;
    use symbolix_core::optimizer::optimize;
    use symbolix_core::parser::expression::Expression;
    use symbolix_core::parser::Parser;
    use symbolix_core::semantic::variable::VariableType;
    use symbolix_core::semantic::Analyzer;
    use symbolix_core::{new_compile_context, with_compile_context};

    new_compile_context! {
        let input = "x == 50 ? 4 : 5";
        let mut lexer: Lexer = Lexer::new(input);
        let expression: Expression = Parser::pratt(&mut lexer);
        let mut analyzer = Analyzer::new();
        let mut semantic = analyzer.analyze_with_ctx(&expression);
        optimize(&mut semantic);
        let code = codegen_semantic(&semantic);
        println!("{}", code);

        let variables = with_compile_context!(ctx, ctx.collect_variables());
        let var_names = variables
            .iter()
            .map(|variable| syn::Ident::new(&variable.name, proc_macro2::Span::call_site()));
        let var_types = variables.iter().map(|variable| match variable.var_type {
            VariableType::Float | VariableType::Fraction | VariableType::Integer => quote! { f64 },
            VariableType::Boolean => quote! { bool },
            _ => panic!("invalid variable type"),
        });
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
                struct CompiledFormula;

                impl CompiledFormula {
                    #[doc = #doc_comment]
                    pub fn calculate(&self, #(#var_names: #var_types),*) -> #return_type {
                        #code
                    }
                }

                CompiledFormula
            }
        };

        println!("{}", expanded.to_string());
    }
}

#[test]
fn solution_set_arguments_do_not_include_target_variable() {
    let x = Variable {
        name: "x".to_string(),
        var_type: VariableType::Float,
        value: None,
    };
    let a = Variable {
        name: "a".to_string(),
        var_type: VariableType::Float,
        value: None,
    };

    let solution_set = SolutionSet {
        target: x,
        branches: vec![SolutionBranch {
            constraint: LogicalExpression::relation(
                &NumericExpression::variable(a.clone()),
                &Symbol::Relation(Relation::GreaterThan),
                &NumericExpression::constant(Number::integer(0)),
            ),
            result: BranchResult::Finite(vec![NumericExpression::constant(Number::integer(1))]),
        }],
    };

    let (names, _) = get_func_arguments(&[CompileValue::SolutionSet(solution_set)]);
    let names = names.into_iter().map(|ident| ident.to_string()).collect::<Vec<_>>();
    assert_eq!(names, vec!["a".to_string()]);
}
