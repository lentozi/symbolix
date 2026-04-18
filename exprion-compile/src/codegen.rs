use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use exprion_core::equation::{BranchResult, SolutionSet};
use exprion_core::lexer::symbol::{Relation, Symbol};
use exprion_core::semantic::semantic_ir::{
    logic::LogicalExpression, numeric::NumericExpression, SemanticExpression,
};
use exprion_core::semantic::variable::{Variable, VariableType};
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
                    "exprion! cannot lower solution sets with identity branches to runtime values"
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

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use exprion_core::lexer::constant::Number;

    fn numeric_var(name: &str, ty: VariableType) -> Variable {
        Variable {
            name_id: 0,
            name: name.to_string(),
            var_type: ty,
            value: None,
        }
    }

    #[test]
    fn get_func_arguments_collects_variables_from_all_compile_values() {
        let x = numeric_var("x", VariableType::Float);
        let y = numeric_var("y", VariableType::Integer);
        let flag = numeric_var("flag", VariableType::Boolean);
        let values = vec![
            CompileValue::Variable(x.clone()),
            CompileValue::Semantic(SemanticExpression::numeric(
                NumericExpression::variable(y.clone())
                    + NumericExpression::constant(Number::integer(1)),
            )),
            CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::variable(
                flag.clone(),
            ))),
        ];

        let (names, types) = get_func_arguments(&values);
        let names = names.into_iter().map(|id| id.to_string()).collect::<Vec<_>>();
        let types = types.into_iter().map(|ts| ts.to_string()).collect::<Vec<_>>();
        assert_eq!(names, vec!["flag", "x", "y"]);
        assert_eq!(types, vec!["bool", "f64", "i32"]);
    }

    #[test]
    fn solution_set_arguments_do_not_include_target_variable() {
        let x = numeric_var("x", VariableType::Float);
        let a = numeric_var("a", VariableType::Float);
        let solution_set = SolutionSet {
            target: x,
            branches: vec![exprion_core::equation::SolutionBranch {
                constraint: LogicalExpression::relation(
                    &NumericExpression::variable(a.clone()),
                    &Symbol::Relation(Relation::GreaterThan),
                    &NumericExpression::constant(Number::integer(0)),
                ),
                result: BranchResult::Finite(vec![NumericExpression::constant(Number::integer(1))]),
            }],
        };

        let (names, _) = get_func_arguments(&[CompileValue::SolutionSet(solution_set)]);
        assert_eq!(
            names.into_iter().map(|id| id.to_string()).collect::<Vec<_>>(),
            vec!["a"]
        );
    }

    #[test]
    fn return_types_and_codegen_cover_scalar_vector_and_identity_solution_sets() {
        let x = numeric_var("x", VariableType::Float);
        let scalar = SolutionSet {
            target: x.clone(),
            branches: vec![exprion_core::equation::SolutionBranch::finite(
                LogicalExpression::constant(true),
                vec![NumericExpression::constant(Number::integer(1))],
            )],
        };
        assert_eq!(
            get_func_return_type(&CompileValue::SolutionSet(scalar.clone())).to_string(),
            "f64"
        );
        assert!(codegen_value(&CompileValue::SolutionSet(scalar))
            .to_string()
            .contains("if"));

        let vector = SolutionSet {
            target: x.clone(),
            branches: vec![exprion_core::equation::SolutionBranch::finite(
                LogicalExpression::constant(true),
                vec![
                    NumericExpression::constant(Number::integer(1)),
                    NumericExpression::constant(Number::integer(2)),
                ],
            )],
        };
        assert_eq!(
            get_func_return_type(&CompileValue::SolutionSet(vector.clone())).to_string(),
            "Vec < f64 >"
        );
        assert!(codegen_value(&CompileValue::SolutionSet(vector))
            .to_string()
            .contains("vec !"));

        let identity = SolutionSet {
            target: x,
            branches: vec![exprion_core::equation::SolutionBranch::identity(
                LogicalExpression::constant(true),
            )],
        };
        let panic = std::panic::catch_unwind(|| {
            let _ = get_func_return_type(&CompileValue::SolutionSet(identity));
        });
        assert!(panic.is_err());
    }

    #[test]
    fn codegen_numeric_and_logical_cover_empty_and_piecewise_shapes() {
        let numeric_empty_add = NumericExpression::Addition(exprion_core::numeric_bucket![]);
        let numeric_empty_mul = NumericExpression::Multiplication(exprion_core::numeric_bucket![]);
        assert_eq!(codegen_numeric(&numeric_empty_add).to_string(), "0.0");
        assert!(codegen_numeric(&numeric_empty_mul).to_string().contains("1.0"));

        let piecewise = NumericExpression::Piecewise {
            cases: vec![(
                LogicalExpression::constant(true),
                NumericExpression::constant(Number::integer(4)),
            )],
            otherwise: None,
        };
        assert!(codegen_numeric(&piecewise).to_string().contains("NAN"));

        let logical_empty_and = LogicalExpression::And(exprion_core::logical_bucket![]);
        let logical_empty_or = LogicalExpression::Or(exprion_core::logical_bucket![]);
        assert_eq!(codegen_logical(&logical_empty_and).to_string(), "true");
        assert_eq!(codegen_logical(&logical_empty_or).to_string(), "false");
    }

    #[test]
    fn generate_struct_and_multi_codegen_values_emit_expected_shape() {
        let names = vec![format_ident!("x"), format_ident!("y")];
        let types = vec![quote! { f64 }, quote! { bool }];
        let generated = generate_struct(
            names.clone(),
            types,
            quote! { (f64, bool) },
            quote! { (x + 1.0, y) },
        );
        let rendered = generated.to_string();
        assert!(rendered.contains("CompiledFormula"));
        assert!(rendered.contains("calculate"));
        assert!(rendered.contains("to_closure"));

        let tuple_code = multi_codegen_values(
            &vec![
                CompileValue::Semantic(SemanticExpression::numeric(NumericExpression::constant(
                    Number::integer(1),
                ))),
                CompileValue::Semantic(SemanticExpression::logical(LogicalExpression::constant(
                    true,
                ))),
            ],
            &names,
        );
        assert!(tuple_code.to_string().contains("let x"));
    }
}


