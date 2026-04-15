//! 分段方程求解模块

use crate::{
    equation::{Equation, SolveError},
    optimizer::normalize_numeric,
    semantic::semantic_ir::{logic::LogicalExpression, numeric::NumericExpression},
};

pub struct PiecewiseSolver;

impl PiecewiseSolver {
    /// 将分段表达式展开为多个方程
    pub fn expand(equation: &Equation) -> Result<Vec<Equation>, SolveError> {
        let branches = expand_numeric(&equation.expr);
        Ok(branches
            .into_iter()
            .map(|(constraint, expr)| Equation {
                expr: NumericExpression::Piecewise {
                    cases: vec![({
                        let mut expr = expr;
                        normalize_numeric(&mut expr);
                        (constraint, expr)
                    })],
                    otherwise: None,
                },
                solve_for: equation.solve_for.clone(),
            })
            .collect())
    }
}

pub fn split_branch_equation(equation: &Equation) -> (LogicalExpression, NumericExpression) {
    match &equation.expr {
        NumericExpression::Piecewise {
            cases,
            otherwise: None,
        } if cases.len() == 1 => cases[0].clone(),
        expr => (LogicalExpression::constant(true), expr.clone()),
    }
}

/// 消除 NumericExpression 中的分段表达式，将其展开为多个表达式
fn expand_numeric(expr: &NumericExpression) -> Vec<(LogicalExpression, NumericExpression)> {
    match expr {
        NumericExpression::Constant(_) | NumericExpression::Variable(_) => {
            vec![(LogicalExpression::constant(true), expr.clone())]
        }
        NumericExpression::Negation(inner) => expand_numeric(inner)
            .into_iter()
            .map(|(constraint, expr)| {
                let mut expr = -expr;
                normalize_numeric(&mut expr);
                (constraint, expr)
            })
            .collect(),
        NumericExpression::Addition(bucket) => combine_branches(
            bucket.iter().map(|expr| expand_numeric(&expr)).collect(),
            |terms| {
                let expr = terms.into_iter().fold(
                    NumericExpression::constant(crate::lexer::constant::Number::integer(0)),
                    |acc, expr| acc + expr,
                );
                let mut expr = expr;
                normalize_numeric(&mut expr);
                expr
            },
        ),
        NumericExpression::Multiplication(bucket) => combine_branches(
            bucket.iter().map(|expr| expand_numeric(&expr)).collect(),
            |factors| {
                let expr = factors.into_iter().fold(
                    NumericExpression::constant(crate::lexer::constant::Number::integer(1)),
                    |acc, expr| acc * expr,
                );
                let mut expr = expr;
                normalize_numeric(&mut expr);
                expr
            },
        ),
        NumericExpression::Power { base, exponent } => {
            let mut branches = Vec::new();
            for (base_constraint, base_expr) in expand_numeric(base) {
                for (exp_constraint, exp_expr) in expand_numeric(exponent) {
                    branches.push((
                        LogicalExpression::and(&base_constraint, &exp_constraint),
                        {
                            let mut expr = NumericExpression::power(&base_expr, &exp_expr);
                            normalize_numeric(&mut expr);
                            expr
                        },
                    ));
                }
            }
            branches
        }
        NumericExpression::Piecewise { cases, otherwise } => {
            let mut branches = Vec::new();
            let mut covered = LogicalExpression::constant(false);
            for (condition, expr) in cases {
                for (inner_constraint, inner_expr) in expand_numeric(expr) {
                    branches.push((
                        LogicalExpression::and(
                            condition,
                            &LogicalExpression::and(&!covered.clone(), &inner_constraint),
                        ),
                        inner_expr,
                    ));
                }
                covered = LogicalExpression::or(&covered, condition);
            }
            if let Some(otherwise) = otherwise {
                for (inner_constraint, inner_expr) in expand_numeric(otherwise) {
                    branches.push((
                        LogicalExpression::and(&!covered.clone(), &inner_constraint),
                        inner_expr,
                    ));
                }
            }
            branches
        }
    }
}

/// 将多个分支列表合并为一个，使用 `build_expr` 构建表达式
fn combine_branches<F>(
    branch_lists: Vec<Vec<(LogicalExpression, NumericExpression)>>,
    build_expr: F,
) -> Vec<(LogicalExpression, NumericExpression)>
where
    F: Fn(Vec<NumericExpression>) -> NumericExpression,
{
    let mut acc = vec![(LogicalExpression::constant(true), Vec::new())];
    for branch_list in branch_lists {
        let mut next = Vec::new();
        for (acc_constraint, acc_exprs) in &acc {
            for (constraint, expr) in &branch_list {
                let mut exprs = acc_exprs.clone();
                exprs.push(expr.clone());
                next.push((LogicalExpression::and(acc_constraint, constraint), exprs));
            }
        }
        acc = next;
    }

    acc.into_iter()
        .map(|(constraint, exprs)| (constraint, build_expr(exprs)))
        .collect()
}
