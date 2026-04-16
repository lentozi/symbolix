mod error;
mod llvm_backend;

use std::collections::BTreeSet;

pub use error::JitError;

use exprion_core::semantic::{
    semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
    variable::{Variable, VariableType},
};

use llvm_backend::LlvmNumericKernel;

pub fn jit_compile_numeric(semantic: SemanticExpression) -> Result<JitNumericFunction, JitError> {
    let numeric = match semantic {
        SemanticExpression::Numeric(numeric) => numeric,
        SemanticExpression::Logical(_) => {
            return Err(JitError::UnsupportedExpression(
                "JIT currently only supports numeric expressions".to_string(),
            ))
        }
    };

    let variable_names = collect_numeric_variables(&numeric)?
        .into_iter()
        .collect::<Vec<_>>();

    let kernel = LlvmNumericKernel::compile(&numeric, &variable_names)?;
    Ok(JitNumericFunction {
        variable_names,
        kernel,
    })
}

pub struct JitNumericFunction {
    variable_names: Vec<String>,
    kernel: LlvmNumericKernel,
}

impl JitNumericFunction {
    pub fn arity(&self) -> usize {
        self.variable_names.len()
    }

    pub fn variables(&self) -> &[String] {
        &self.variable_names
    }

    pub fn calculate(&self, arguments: &[f64]) -> Result<f64, JitError> {
        if arguments.len() != self.arity() {
            return Err(JitError::ArityMismatch {
                expected: self.arity(),
                actual: arguments.len(),
            });
        }
        Ok(self.kernel.call(arguments))
    }
}

fn collect_numeric_variables(expr: &NumericExpression) -> Result<BTreeSet<String>, JitError> {
    let mut names = BTreeSet::new();
    collect_numeric_variables_into(expr, &mut names)?;
    Ok(names)
}

fn collect_numeric_variables_into(
    expr: &NumericExpression,
    names: &mut BTreeSet<String>,
) -> Result<(), JitError> {
    match expr {
        NumericExpression::Constant(_) => {}
        NumericExpression::Variable(variable) => {
            assert_numeric_variable(variable)?;
            names.insert(variable.name.clone());
        }
        NumericExpression::Negation(inner) => collect_numeric_variables_into(inner, names)?,
        NumericExpression::Addition(bucket) | NumericExpression::Multiplication(bucket) => {
            for expr in bucket.iter() {
                collect_numeric_variables_into(&expr, names)?;
            }
        }
        NumericExpression::Power { .. } => {
            return Err(JitError::UnsupportedExpression(
                "JIT does not yet lower power expressions".to_string(),
            ))
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            for (condition, branch) in cases {
                collect_logical_variables(condition, names)?;
                collect_numeric_variables_into(branch, names)?;
            }
            if let Some(otherwise) = otherwise {
                collect_numeric_variables_into(otherwise, names)?;
            }
            return Err(JitError::UnsupportedExpression(
                "JIT does not yet lower piecewise expressions".to_string(),
            ));
        }
    }

    Ok(())
}

fn collect_logical_variables(
    expr: &LogicalExpression,
    names: &mut BTreeSet<String>,
) -> Result<(), JitError> {
    match expr {
        LogicalExpression::Constant(_) => {}
        LogicalExpression::Variable(variable) => {
            if variable.var_type != VariableType::Boolean {
                return Err(JitError::UnsupportedVariable(format!(
                    "logical variable `{}` must have bool type",
                    variable.name
                )));
            }
            names.insert(variable.name.clone());
        }
        LogicalExpression::Not(inner) => collect_logical_variables(inner, names)?,
        LogicalExpression::And(bucket) | LogicalExpression::Or(bucket) => {
            for expr in bucket.iter() {
                collect_logical_variables(&expr, names)?;
            }
        }
        LogicalExpression::Relation { left, right, .. } => {
            collect_numeric_variables_into(left, names)?;
            collect_numeric_variables_into(right, names)?;
        }
    }

    Ok(())
}

fn assert_numeric_variable(variable: &Variable) -> Result<(), JitError> {
    match variable.var_type {
        VariableType::Integer | VariableType::Float | VariableType::Fraction => Ok(()),
        VariableType::Boolean => Err(JitError::UnsupportedVariable(format!(
            "numeric JIT cannot accept boolean variable `{}`",
            variable.name
        ))),
        VariableType::Unknown => Err(JitError::UnsupportedVariable(format!(
            "numeric JIT cannot accept variable `{}` with unknown type",
            variable.name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::jit_compile_numeric;
    use exprion_core::{
        lexer::Lexer,
        new_compile_context,
        optimizer::optimize,
        parser::Parser,
        semantic::Analyzer,
    };

    fn parse_numeric(expression: &str) -> exprion_core::semantic::semantic_ir::SemanticExpression {
        new_compile_context! {
            let mut lexer = Lexer::new(expression);
            let parsed = Parser::pratt(&mut lexer);
            let mut analyzer = Analyzer::new();
            let mut semantic = analyzer.analyze_with_ctx(&parsed);
            optimize(&mut semantic);
            semantic
        }
    }

    #[test]
    fn jit_compiles_basic_numeric_expression() {
        let compiled = jit_compile_numeric(parse_numeric("z + x * 2 + 1")).unwrap();

        assert_eq!(compiled.variables(), &["x".to_string(), "z".to_string()]);
        let result = compiled.calculate(&[3.0, 10.0]).unwrap();
        assert!((result - 17.0).abs() < 1e-9);
    }

    #[test]
    fn jit_supports_parenthesized_division() {
        let compiled = jit_compile_numeric(parse_numeric("(x + 6) / 2")).unwrap();
        let result = compiled.calculate(&[8.0]).unwrap();
        assert!((result - 7.0).abs() < 1e-9);
    }

    #[test]
    fn jit_rejects_logical_expressions() {
        let semantic = parse_numeric("x > 4");
        let compiled = jit_compile_numeric(semantic);
        assert!(compiled.is_err());
    }

    #[test]
    fn jit_compiles_from_semantic_boundary() {
        let compiled = jit_compile_numeric(parse_numeric("x + 4")).unwrap();
        let result = compiled.calculate(&[6.0]).unwrap();

        assert!((result - 10.0).abs() < 1e-9);
    }
}
