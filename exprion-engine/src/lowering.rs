use std::collections::HashMap;

use exprion_core::lexer::{
    constant::Number,
    symbol::{Relation, Symbol},
};
use exprion_core::semantic::{
    semantic_ir::{logic::LogicalExpression, numeric::NumericExpression, SemanticExpression},
    variable::{Variable, VariableType},
};

use crate::{JitError, ParameterInfo};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LoweredNumericExpr {
    Constant(Number),
    Parameter(usize),
    Negation(Box<LoweredNumericExpr>),
    Addition(Vec<LoweredNumericExpr>),
    Multiplication(Vec<LoweredNumericExpr>),
    Power {
        base: Box<LoweredNumericExpr>,
        exponent: Box<LoweredNumericExpr>,
    },
    Piecewise {
        cases: Vec<(LoweredLogicalExpr, LoweredNumericExpr)>,
        otherwise: Option<Box<LoweredNumericExpr>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum LoweredLogicalExpr {
    Constant(bool),
    Not(Box<LoweredLogicalExpr>),
    And(Vec<LoweredLogicalExpr>),
    Or(Vec<LoweredLogicalExpr>),
    Relation {
        left: Box<LoweredNumericExpr>,
        operator: Relation,
        right: Box<LoweredNumericExpr>,
    },
}

pub(crate) fn lower_numeric_semantic(
    semantic: SemanticExpression,
) -> Result<(LoweredNumericExpr, Vec<ParameterInfo>), JitError> {
    let numeric = match semantic {
        SemanticExpression::Numeric(numeric) => numeric,
        SemanticExpression::Logical(_) => {
            return Err(JitError::UnsupportedExpression(
                "JIT currently only supports numeric expressions".to_string(),
            ))
        }
    };

    let mut collected = Vec::new();
    collect_numeric_variables(&numeric, &mut collected)?;
    collected.sort_unstable_by(|left, right| left.name.cmp(right.name));
    collected.dedup_by(|left, right| left.name == right.name);

    let parameters = collected
        .into_iter()
        .enumerate()
        .map(|(index, variable)| ParameterInfo {
            name: variable.name.to_string(),
            index,
            name_id: variable.name_id,
        })
        .collect::<Vec<_>>();

    let mut slots = HashMap::with_capacity(parameters.len());
    for parameter in &parameters {
        if parameter.name_id != 0 {
            slots.insert(parameter.name_id, parameter.index);
        }
    }
    for parameter in &parameters {
        if parameter.name_id == 0 {
            continue;
        }
    }

    let lowered = lower_numeric_expr(&numeric, &parameters, &slots)?;
    Ok((lowered, parameters))
}

#[derive(Clone, Copy)]
struct CollectedVariable<'a> {
    name: &'a str,
    name_id: u32,
}

fn collect_numeric_variables<'a>(
    expr: &'a NumericExpression,
    names: &mut Vec<CollectedVariable<'a>>,
) -> Result<(), JitError> {
    match expr {
        NumericExpression::Constant(_) => {}
        NumericExpression::Variable(variable) => {
            assert_numeric_variable(variable)?;
            names.push(CollectedVariable {
                name: variable.name.as_str(),
                name_id: variable.name_id,
            });
        }
        NumericExpression::Negation(inner) => collect_numeric_variables(inner, names)?,
        NumericExpression::Addition(bucket) | NumericExpression::Multiplication(bucket) => {
            for variable in &bucket.variables {
                assert_numeric_variable(variable)?;
                names.push(CollectedVariable {
                    name: variable.name.as_str(),
                    name_id: variable.name_id,
                });
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
    names: &mut Vec<CollectedVariable<'a>>,
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
                collect_logical_variables(expr, names)?;
            }
        }
        LogicalExpression::Relation { left, right, .. } => {
            collect_numeric_variables(left, names)?;
            collect_numeric_variables(right, names)?;
        }
    }

    Ok(())
}

fn lower_numeric_expr(
    expr: &NumericExpression,
    parameters: &[ParameterInfo],
    slots: &HashMap<u32, usize>,
) -> Result<LoweredNumericExpr, JitError> {
    Ok(match expr {
        NumericExpression::Constant(number) => LoweredNumericExpr::Constant(number.clone()),
        NumericExpression::Variable(variable) => {
            LoweredNumericExpr::Parameter(resolve_slot(variable, parameters, slots)?)
        }
        NumericExpression::Negation(inner) => {
            LoweredNumericExpr::Negation(Box::new(lower_numeric_expr(inner, parameters, slots)?))
        }
        NumericExpression::Addition(bucket) => {
            let mut terms =
                Vec::with_capacity(bucket.constants.len() + bucket.variables.len() + bucket.expressions.len());
            terms.extend(bucket.constants.iter().cloned().map(LoweredNumericExpr::Constant));
            for variable in &bucket.variables {
                terms.push(LoweredNumericExpr::Parameter(resolve_slot(variable, parameters, slots)?));
            }
            for expr in &bucket.expressions {
                terms.push(lower_numeric_expr(expr, parameters, slots)?);
            }
            LoweredNumericExpr::Addition(terms)
        }
        NumericExpression::Multiplication(bucket) => {
            let mut terms =
                Vec::with_capacity(bucket.constants.len() + bucket.variables.len() + bucket.expressions.len());
            terms.extend(bucket.constants.iter().cloned().map(LoweredNumericExpr::Constant));
            for variable in &bucket.variables {
                terms.push(LoweredNumericExpr::Parameter(resolve_slot(variable, parameters, slots)?));
            }
            for expr in &bucket.expressions {
                terms.push(lower_numeric_expr(expr, parameters, slots)?);
            }
            LoweredNumericExpr::Multiplication(terms)
        }
        NumericExpression::Power { base, exponent } => LoweredNumericExpr::Power {
            base: Box::new(lower_numeric_expr(base, parameters, slots)?),
            exponent: Box::new(lower_numeric_expr(exponent, parameters, slots)?),
        },
        NumericExpression::Piecewise { cases, otherwise } => LoweredNumericExpr::Piecewise {
            cases: cases
                .iter()
                .map(|(condition, value)| {
                    Ok((
                        lower_logical_expr(condition, parameters, slots)?,
                        lower_numeric_expr(value, parameters, slots)?,
                    ))
                })
                .collect::<Result<_, JitError>>()?,
            otherwise: otherwise
                .as_ref()
                .map(|expr| lower_numeric_expr(expr, parameters, slots).map(Box::new))
                .transpose()?,
        },
    })
}

fn lower_logical_expr(
    expr: &LogicalExpression,
    parameters: &[ParameterInfo],
    slots: &HashMap<u32, usize>,
) -> Result<LoweredLogicalExpr, JitError> {
    Ok(match expr {
        LogicalExpression::Constant(value) => LoweredLogicalExpr::Constant(*value),
        LogicalExpression::Variable(variable) => {
            return Err(JitError::UnsupportedLogicalVariable(variable.name.clone()))
        }
        LogicalExpression::Not(inner) => {
            LoweredLogicalExpr::Not(Box::new(lower_logical_expr(inner, parameters, slots)?))
        }
        LogicalExpression::And(bucket) => {
            let mut terms = Vec::with_capacity(bucket.constants.len() + bucket.expressions.len());
            terms.extend(bucket.constants.iter().copied().map(LoweredLogicalExpr::Constant));
            for expr in &bucket.expressions {
                terms.push(lower_logical_expr(expr, parameters, slots)?);
            }
            LoweredLogicalExpr::And(terms)
        }
        LogicalExpression::Or(bucket) => {
            let mut terms = Vec::with_capacity(bucket.constants.len() + bucket.expressions.len());
            terms.extend(bucket.constants.iter().copied().map(LoweredLogicalExpr::Constant));
            for expr in &bucket.expressions {
                terms.push(lower_logical_expr(expr, parameters, slots)?);
            }
            LoweredLogicalExpr::Or(terms)
        }
        LogicalExpression::Relation {
            left,
            operator: Symbol::Relation(relation),
            right,
        } => LoweredLogicalExpr::Relation {
            left: Box::new(lower_numeric_expr(left, parameters, slots)?),
            operator: *relation,
            right: Box::new(lower_numeric_expr(right, parameters, slots)?),
        },
        LogicalExpression::Relation { operator, .. } => {
            return Err(JitError::UnsupportedExpression(format!(
                "unsupported relation operator `{operator}`"
            )))
        }
    })
}

fn resolve_slot(
    variable: &Variable,
    parameters: &[ParameterInfo],
    slots: &HashMap<u32, usize>,
) -> Result<usize, JitError> {
    if variable.name_id != 0 {
        if let Some(index) = slots.get(&variable.name_id).copied() {
            return Ok(index);
        }
    }

    parameters
        .iter()
        .find(|parameter| parameter.name == variable.name)
        .map(|parameter| parameter.index)
        .ok_or_else(|| JitError::Codegen(format!("missing variable slot for `{}`", variable.name)))
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
