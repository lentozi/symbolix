use std::collections::BTreeSet;

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

    let parameter_names = collect_numeric_variables(&numeric)?
        .into_iter()
        .collect::<Vec<_>>();
    let parameters = parameter_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| ParameterInfo { name, index })
        .collect::<Vec<_>>();

    Ok((numeric, parameters))
}

pub(crate) fn lower_logical_semantic(
    semantic: SemanticExpression,
) -> Result<(LogicalExpression, Vec<ParameterInfo>), JitError> {
    let logical = match semantic {
        SemanticExpression::Logical(logical) => logical,
        SemanticExpression::Numeric(_) => {
            return Err(JitError::UnsupportedExpression(
                "JIT logical compilation requires a logical expression".to_string(),
            ))
        }
    };

    let parameter_names = collect_logical_numeric_variables(&logical)?
        .into_iter()
        .collect::<Vec<_>>();
    let parameters = parameter_names
        .into_iter()
        .enumerate()
        .map(|(index, name)| ParameterInfo { name, index })
        .collect::<Vec<_>>();

    Ok((logical, parameters))
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
        NumericExpression::Power { base, exponent } => {
            collect_numeric_variables_into(base, names)?;
            collect_numeric_variables_into(exponent, names)?;
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            for (condition, branch) in cases {
                collect_logical_variables(condition, names)?;
                collect_numeric_variables_into(branch, names)?;
            }
            if let Some(otherwise) = otherwise {
                collect_numeric_variables_into(otherwise, names)?;
            }
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
            return Err(JitError::UnsupportedLogicalVariable(variable.name.clone()));
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

fn collect_logical_numeric_variables(
    expr: &LogicalExpression,
) -> Result<BTreeSet<String>, JitError> {
    let mut names = BTreeSet::new();
    collect_logical_variables(expr, &mut names)?;
    Ok(names)
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
