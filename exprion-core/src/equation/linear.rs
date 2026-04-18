use crate::{
    equation::{
        contains_target, is_zero, piecewise::split_branch_equation, Equation,
        SolutionBranch, SolutionSet, SolveError, Solver,
    },
    lexer::constant::Number,
    optimizer::normalize_numeric,
    semantic::{semantic_ir::numeric::NumericExpression, variable::Variable},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinearForm {
    pub target: Variable,
    pub coef: NumericExpression,
    pub constant: NumericExpression,
}

pub struct LinearSolver;

impl LinearForm {
    pub fn extract(expr: &NumericExpression, target: &Variable) -> Option<Self> {
        let (coef, constant) = extract_linear_parts(expr, target)?;
        let mut coef = coef;
        let mut constant = constant;
        normalize_numeric(&mut coef);
        normalize_numeric(&mut constant);
        Some(Self {
            target: target.clone(),
            coef,
            constant,
        })
    }
}

impl Solver for LinearSolver {
    /// 判断是否可求解
    fn can_solve(eq: &Equation) -> bool {
        let (_, expr) = split_branch_equation(eq);
        LinearForm::extract(&expr, &eq.solve_for).is_some()
    }

    fn solve(eq: &Equation) -> Result<SolutionSet, SolveError> {
        let (constraint, expr) = split_branch_equation(eq);
        let linear =
            LinearForm::extract(&expr, &eq.solve_for).ok_or(SolveError::NonLinearExpression)?;

        let branches = if is_zero(&linear.coef) {
            if is_zero(&linear.constant) {
                vec![SolutionBranch::identity(constraint)]
            } else {
                Vec::new()
            }
        } else {
            let mut solution = -linear.constant.clone() / linear.coef.clone();
            normalize_numeric(&mut solution);
            vec![SolutionBranch::finite(constraint, vec![solution])]
        };

        Ok(SolutionSet::new(eq.solve_for.clone(), branches))
    }
}

fn extract_linear_parts(
    expr: &NumericExpression,
    target: &Variable,
) -> Option<(NumericExpression, NumericExpression)> {
    match expr {
        NumericExpression::Constant(_) => Some((zero(), expr.clone())),
        NumericExpression::Variable(variable) => {
            if variable.same_identity(target) {
                Some((one(), zero()))
            } else {
                Some((zero(), expr.clone()))
            }
        }
        NumericExpression::Negation(inner) => {
            let (coef, constant) = extract_linear_parts(inner, target)?;
            let mut coef = -coef;
            let mut constant = -constant;
            normalize_numeric(&mut coef);
            normalize_numeric(&mut constant);
            Some((coef, constant))
        }
        NumericExpression::Addition(bucket) => {
            let mut coef = zero();
            let mut constant = zero();
            for expr in bucket.iter() {
                let (term_coef, term_constant) = extract_linear_parts(&expr, target)?;
                coef = coef + term_coef;
                constant = constant + term_constant;
                normalize_numeric(&mut coef);
                normalize_numeric(&mut constant);
            }
            Some((coef, constant))
        }
        NumericExpression::Multiplication(bucket) => {
            let forms = bucket
                .iter()
                .map(|expr| extract_linear_parts(&expr, target))
                .collect::<Option<Vec<_>>>()?;

            let variable_forms = forms
                .iter()
                .filter(|(coef, _)| !is_zero(coef))
                .collect::<Vec<_>>();

            match variable_forms.len() {
                0 => {
                    let constant = bucket
                        .iter()
                        .fold(one(), |acc, expr| {
                            let mut value = acc * expr;
                            normalize_numeric(&mut value);
                            value
                        });
                    Some((zero(), constant))
                }
                1 => {
                    let mut constants = Vec::new();
                    for (coef, constant) in &forms {
                        if is_zero(coef) {
                            constants.push(constant.clone());
                        }
                    }

                    let const_product = constants
                        .into_iter()
                        .fold(one(), |acc, expr| {
                            let mut value = acc * expr;
                            normalize_numeric(&mut value);
                            value
                        });

                    let (coef, constant) = variable_forms[0];
                    let mut new_coef = coef.clone() * const_product.clone();
                    let mut new_constant = constant.clone() * const_product;
                    normalize_numeric(&mut new_coef);
                    normalize_numeric(&mut new_constant);
                    Some((
                        new_coef,
                        new_constant,
                    ))
                }
                _ => None,
            }
        }
        NumericExpression::Power { base, exponent } => {
            if !contains_target(expr, target) {
                return Some((zero(), expr.clone()));
            }

            match exponent.as_ref() {
                NumericExpression::Constant(number) if number == &Number::integer(1) => {
                    extract_linear_parts(base, target)
                }
                _ => None,
            }
        }
        NumericExpression::Piecewise { .. } => None,
    }
}

fn zero() -> NumericExpression {
    NumericExpression::constant(Number::integer(0))
}

fn one() -> NumericExpression {
    NumericExpression::constant(Number::integer(1))
}
