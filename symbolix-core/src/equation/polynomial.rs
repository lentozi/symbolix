use crate::{
    equation::{
        contains_target, is_zero, optimize_numeric, piecewise::split_branch_equation,
        Equation, SolutionBranch, SolutionSet, SolveError, Solver,
    },
    lexer::constant::Number,
    semantic::{semantic_ir::numeric::NumericExpression, variable::Variable},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolynomialForm {
    pub target: Variable,
    pub coefficients: Vec<NumericExpression>,
}

pub struct PolynomialSolver;

impl Solver for PolynomialSolver {
    fn can_solve(eq: &Equation) -> bool {
        let (_, expr) = split_branch_equation(eq);
        let Some(form) = PolynomialForm::extract(&expr, &eq.solve_for, 2) else {
            return false;
        };
        form.degree() == Some(2)
    }

    fn solve(eq: &Equation) -> Result<SolutionSet, SolveError> {
        let (constraint, expr) = split_branch_equation(eq);
        let form = PolynomialForm::extract(&expr, &eq.solve_for, 2)
            .ok_or(SolveError::NonPolynomialExpression)?;

        if form.degree() != Some(2) {
            return Err(SolveError::NonPolynomialExpression);
        }

        let a = form.coefficient(2);
        let b = form.coefficient(1);
        let c = form.coefficient(0);

        if is_zero(&a) {
            return Err(SolveError::NonPolynomialExpression);
        }

        let discriminant = optimize_numeric(
            b.clone() * b.clone()
                - NumericExpression::constant(Number::integer(4)) * a.clone() * c.clone(),
        );

        let two_a = optimize_numeric(NumericExpression::constant(Number::integer(2)) * a.clone());
        let minus_b = optimize_numeric(-b);

        let branches = if is_zero(&discriminant) {
            vec![SolutionBranch::finite(
                constraint,
                vec![optimize_numeric(minus_b / two_a)],
            )]
        } else {
            let sqrt_discriminant = optimize_numeric(NumericExpression::power(
                &discriminant,
                &NumericExpression::constant(Number::fraction(1, 2)),
            ));

            vec![SolutionBranch::finite(
                constraint,
                vec![
                    optimize_numeric((minus_b.clone() + sqrt_discriminant.clone()) / two_a.clone()),
                    optimize_numeric((minus_b - sqrt_discriminant) / two_a),
                ],
            )]
        };

        Ok(SolutionSet::new(eq.solve_for.clone(), branches))
    }
}

impl PolynomialForm {
    pub fn extract(
        expr: &NumericExpression,
        target: &Variable,
        max_degree: usize,
    ) -> Option<Self> {
        let coefficients = extract_polynomial(expr, target, max_degree)?;
        Some(Self {
            target: target.clone(),
            coefficients,
        })
    }

    pub fn degree(&self) -> Option<usize> {
        self.coefficients
            .iter()
            .enumerate()
            .rev()
            .find(|(_, coef)| !is_zero(coef))
            .map(|(index, _)| index)
    }

    pub fn coefficient(&self, degree: usize) -> NumericExpression {
        self.coefficients
            .get(degree)
            .cloned()
            .unwrap_or_else(|| NumericExpression::constant(Number::integer(0)))
    }
}

fn extract_polynomial(
    expr: &NumericExpression,
    target: &Variable,
    max_degree: usize,
) -> Option<Vec<NumericExpression>> {
    match expr {
        NumericExpression::Constant(_) => Some(vec![expr.clone()]),
        NumericExpression::Variable(variable) => {
            if variable == target {
                let mut result = vec![NumericExpression::constant(Number::integer(0)); 2];
                result[1] = NumericExpression::constant(Number::integer(1));
                Some(result)
            } else {
                Some(vec![expr.clone()])
            }
        }
        NumericExpression::Negation(inner) => Some(
            extract_polynomial(inner, target, max_degree)?
                .into_iter()
                .map(|coef| optimize_numeric(-coef))
                .collect(),
        ),
        NumericExpression::Addition(bucket) => {
            let mut acc = vec![NumericExpression::constant(Number::integer(0)); max_degree + 1];
            for term in bucket.iter() {
                let coeffs = extract_polynomial(&term, target, max_degree)?;
                acc = add_coefficients(acc, coeffs, max_degree);
            }
            Some(trim_coefficients(acc))
        }
        NumericExpression::Multiplication(bucket) => {
            let mut acc = vec![NumericExpression::constant(Number::integer(1))];
            for factor in bucket.iter() {
                let coeffs = extract_polynomial(&factor, target, max_degree)?;
                acc = multiply_coefficients(acc, coeffs, max_degree)?;
            }
            Some(trim_coefficients(acc))
        }
        NumericExpression::Power { base, exponent } => {
            if !contains_target(expr, target) {
                return Some(vec![expr.clone()]);
            }

            let power = match exponent.as_ref() {
                NumericExpression::Constant(number) => number.to_integer()?,
                _ => return None,
            };

            if power < 0 || power as usize > max_degree {
                return None;
            }

            if power == 0 {
                return Some(vec![NumericExpression::constant(Number::integer(1))]);
            }

            let base_coeffs = extract_polynomial(base, target, max_degree)?;
            let mut acc = vec![NumericExpression::constant(Number::integer(1))];
            for _ in 0..power {
                acc = multiply_coefficients(acc, base_coeffs.clone(), max_degree)?;
            }
            Some(trim_coefficients(acc))
        }
        NumericExpression::Piecewise { .. } => None,
    }
}

fn add_coefficients(
    left: Vec<NumericExpression>,
    right: Vec<NumericExpression>,
    max_degree: usize,
) -> Vec<NumericExpression> {
    let mut result = vec![NumericExpression::constant(Number::integer(0)); max_degree + 1];
    for degree in 0..=max_degree {
        let left_coef = left
            .get(degree)
            .cloned()
            .unwrap_or_else(|| NumericExpression::constant(Number::integer(0)));
        let right_coef = right
            .get(degree)
            .cloned()
            .unwrap_or_else(|| NumericExpression::constant(Number::integer(0)));
        result[degree] = optimize_numeric(left_coef + right_coef);
    }
    trim_coefficients(result)
}

fn multiply_coefficients(
    left: Vec<NumericExpression>,
    right: Vec<NumericExpression>,
    max_degree: usize,
) -> Option<Vec<NumericExpression>> {
    let left_degree = highest_degree(&left);
    let right_degree = highest_degree(&right);
    if left_degree + right_degree > max_degree {
        return None;
    }

    let mut result = vec![NumericExpression::constant(Number::integer(0)); max_degree + 1];
    for (left_degree, left_coef) in left.into_iter().enumerate() {
        for (right_degree, right_coef) in right.iter().cloned().enumerate() {
            let degree = left_degree + right_degree;
            if degree > max_degree {
                return None;
            }
            result[degree] = optimize_numeric(result[degree].clone() + left_coef.clone() * right_coef);
        }
    }
    Some(trim_coefficients(result))
}

fn highest_degree(coefficients: &[NumericExpression]) -> usize {
    coefficients
        .iter()
        .enumerate()
        .rev()
        .find(|(_, coef)| !is_zero(coef))
        .map(|(degree, _)| degree)
        .unwrap_or(0)
}

fn trim_coefficients(mut coefficients: Vec<NumericExpression>) -> Vec<NumericExpression> {
    while coefficients.len() > 1 {
        let last_is_zero = coefficients.last().is_some_and(is_zero);
        if last_is_zero {
            coefficients.pop();
        } else {
            break;
        }
    }
    coefficients
}
