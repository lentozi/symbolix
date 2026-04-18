use exprion_core::semantic::{
    semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
    variable::{Variable, VariableType},
};

use crate::{JitError, ParameterInfo};

pub(crate) fn lower_numeric_semantic(
    semantic: SemanticExpression,
) -> Result<(NumericExpression, Vec<ParameterInfo>), JitError> {
    let numeric = match semantic {
        SemanticExpression::Numeric(numeric) => numeric,
        SemanticExpression::Logical(_) => {
            return Err(JitError::UnsupportedExpression(
                "JIT currently only supports numeric expressions".to_string(),
            ))
        }
    };

    let mut parameter_names = Vec::new();
    collect_numeric_variables(&numeric, &mut parameter_names)?;
    parameter_names.sort_unstable();
    parameter_names.dedup();

    let parameters = parameter_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| ParameterInfo {
            name: name.to_string(),
            index,
        })
        .collect::<Vec<_>>();

    Ok((numeric, parameters))
}

fn collect_numeric_variables<'a>(
    expr: &'a NumericExpression,
    names: &mut Vec<&'a str>,
) -> Result<(), JitError> {
    match expr {
        NumericExpression::Constant(_) => {}
        NumericExpression::Variable(variable) => {
            assert_numeric_variable(variable)?;
            names.push(variable.name.as_str());
        }
        NumericExpression::Negation(inner) => collect_numeric_variables(inner, names)?,
        NumericExpression::Addition(bucket) | NumericExpression::Multiplication(bucket) => {
            for variable in &bucket.variables {
                assert_numeric_variable(variable)?;
                names.push(variable.name.as_str());
            }
            for expr in &bucket.expressions {
                collect_numeric_variables(expr, names)?;
            }
        }
        NumericExpression::Power { base, exponent } => {
            collect_numeric_variables(base, names)?;
            collect_numeric_variables(exponent, names)?;
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            for (condition, branch) in cases {
                collect_logical_variables(condition, names)?;
                collect_numeric_variables(branch, names)?;
            }
            if let Some(otherwise) = otherwise {
                collect_numeric_variables(otherwise, names)?;
            }
        }
    }

    Ok(())
}

fn collect_logical_variables<'a>(
    expr: &'a LogicalExpression,
    names: &mut Vec<&'a str>,
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
            return Err(JitError::UnsupportedLogicalVariable(variable.name.clone()));
        }
        LogicalExpression::Not(inner) => collect_logical_variables(inner, names)?,
        LogicalExpression::And(bucket) | LogicalExpression::Or(bucket) => {
            for variable in &bucket.variables {
                if variable.var_type != VariableType::Boolean {
                    return Err(JitError::UnsupportedVariable(format!(
                        "logical variable `{}` must have bool type",
                        variable.name
                    )));
                }
                return Err(JitError::UnsupportedLogicalVariable(variable.name.clone()));
            }
            for expr in &bucket.expressions {
                collect_logical_variables(&expr, names)?;
            }
        }
        LogicalExpression::Relation { left, right, .. } => {
            collect_numeric_variables(left, names)?;
            collect_numeric_variables(right, names)?;
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
