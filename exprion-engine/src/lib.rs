mod backend;
mod error;
mod lowering;

use std::collections::HashMap;

pub use error::JitError;

use exprion_core::semantic::semantic_ir::SemanticExpression;

use backend::{compile_numeric, CompiledNumericKernel};
use lowering::lower_numeric_semantic;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterInfo {
    pub name: String,
    pub index: usize,
}

pub fn jit_compile_numeric(semantic: SemanticExpression) -> Result<JitNumericFunction, JitError> {
    let (numeric, parameters) = lower_numeric_semantic(semantic)?;
    let parameter_lookup = build_parameter_lookup(&parameters);
    let kernel = compile_numeric(&numeric, &parameters)?;
    Ok(JitNumericFunction {
        parameters,
        parameter_lookup,
        kernel,
    })
}

pub struct JitNumericFunction {
    parameters: Vec<ParameterInfo>,
    parameter_lookup: HashMap<String, usize>,
    kernel: Box<dyn CompiledNumericKernel>,
}

impl JitNumericFunction {
    pub fn arity(&self) -> usize {
        self.parameters.len()
    }

    pub fn variables(&self) -> Vec<String> {
        self.parameters
            .iter()
            .map(|parameter| parameter.name.clone())
            .collect()
    }

    pub fn parameters(&self) -> &[ParameterInfo] {
        &self.parameters
    }

    pub fn calculate(&self, arguments: &[f64]) -> Result<f64, JitError> {
        if arguments.len() != self.arity() {
            return Err(JitError::ArityMismatch {
                expected: self.arity(),
                actual: arguments.len(),
            });
        }
        Ok(self.calculate_unchecked(arguments))
    }

    pub fn calculate_named(&self, arguments: &[(&str, f64)]) -> Result<f64, JitError> {
        let values = resolve_named_arguments(&self.parameters, &self.parameter_lookup, arguments)?;
        self.calculate(&values)
    }

    pub fn calculate_unchecked(&self, arguments: &[f64]) -> f64 {
        self.kernel.call(arguments)
    }
}

fn build_parameter_lookup(parameters: &[ParameterInfo]) -> HashMap<String, usize> {
    parameters
        .iter()
        .map(|parameter| (parameter.name.clone(), parameter.index))
        .collect()
}

fn resolve_named_arguments(
    parameters: &[ParameterInfo],
    parameter_lookup: &HashMap<String, usize>,
    arguments: &[(&str, f64)],
) -> Result<Vec<f64>, JitError> {
    let mut values = vec![0.0; parameters.len()];
    let mut seen = vec![false; parameters.len()];

    for (name, value) in arguments {
        let index = parameter_lookup
            .get(*name)
            .copied()
            .ok_or_else(|| JitError::UnknownArgument((*name).to_string()))?;
        if seen[index] {
            return Err(JitError::DuplicateArgument((*name).to_string()));
        }
        seen[index] = true;
        values[index] = *value;
    }

    for parameter in parameters {
        if !seen[parameter.index] {
            return Err(JitError::MissingArgument(parameter.name.clone()));
        }
    }

    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::{jit_compile_numeric, JitError, ParameterInfo};
    use exprion_core::{
        lexer::Lexer,
        new_compile_context,
        optimizer::optimize,
        parser::Parser,
        semantic::Analyzer,
    };

    fn parse_semantic(expression: &str) -> exprion_core::semantic::semantic_ir::SemanticExpression {
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
        let compiled = jit_compile_numeric(parse_semantic("z + x * 2 + 1")).unwrap();

        assert_eq!(compiled.variables(), vec!["x".to_string(), "z".to_string()]);
        let result = compiled.calculate(&[3.0, 10.0]).unwrap();
        assert!((result - 17.0).abs() < 1e-9);
    }

    #[test]
    fn jit_supports_parenthesized_division() {
        let compiled = jit_compile_numeric(parse_semantic("(x + 6) / 2")).unwrap();
        let result = compiled.calculate(&[8.0]).unwrap();
        assert!((result - 7.0).abs() < 1e-9);
    }

    #[test]
    fn jit_rejects_logical_expressions() {
        let semantic = parse_semantic("x > 4");
        let compiled = jit_compile_numeric(semantic);
        assert!(compiled.is_err());
    }

    #[test]
    fn jit_compiles_from_semantic_boundary() {
        let compiled = jit_compile_numeric(parse_semantic("x + 4")).unwrap();
        let result = compiled.calculate(&[6.0]).unwrap();

        assert!((result - 10.0).abs() < 1e-9);
    }

    #[test]
    fn jit_exposes_parameter_metadata_and_named_arguments() {
        let compiled = jit_compile_numeric(parse_semantic("z + x * 2")).unwrap();

        assert_eq!(
            compiled.parameters(),
            &[
                ParameterInfo {
                    name: "x".to_string(),
                    index: 0,
                },
                ParameterInfo {
                    name: "z".to_string(),
                    index: 1,
                },
            ]
        );

        let result = compiled.calculate_named(&[("z", 10.0), ("x", 3.0)]).unwrap();
        assert!((result - 16.0).abs() < 1e-9);
    }

    #[test]
    fn jit_named_arguments_validate_missing_unknown_and_duplicate_values() {
        let compiled = jit_compile_numeric(parse_semantic("x + z")).unwrap();

        assert!(matches!(
            compiled.calculate_named(&[("x", 1.0)]),
            Err(JitError::MissingArgument(name)) if name == "z"
        ));
        assert!(matches!(
            compiled.calculate_named(&[("x", 1.0), ("z", 2.0), ("y", 3.0)]),
            Err(JitError::UnknownArgument(name)) if name == "y"
        ));
        assert!(matches!(
            compiled.calculate_named(&[("x", 1.0), ("x", 2.0), ("z", 3.0)]),
            Err(JitError::DuplicateArgument(name)) if name == "x"
        ));
    }

    #[test]
    fn jit_compiles_piecewise_numeric_expressions() {
        let compiled = jit_compile_numeric(parse_semantic("x > 0 ? x * 2 : -x")).unwrap();

        assert!((compiled.calculate(&[3.0]).unwrap() - 6.0).abs() < 1e-9);
        assert!((compiled.calculate(&[-4.0]).unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn jit_compiles_pow_expressions() {
        let compiled = jit_compile_numeric(parse_semantic("(x + 1) ^ 2")).unwrap();

        assert!((compiled.calculate(&[3.0]).unwrap() - 16.0).abs() < 1e-9);
    }
}
