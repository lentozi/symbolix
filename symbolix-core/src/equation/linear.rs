use crate::optimizer::optimize;
use crate::{
    equation::EquationTrait,
    lexer::constant::Number,
    semantic::semantic_ir::{numeric::NumericExpression, SemanticExpression},
};

pub struct LinearEquation {
    coef: Number,
    rhs: Number,
}

impl LinearEquation {
    pub fn new(coef: Number, rhs: Number) -> Self {
        Self { coef, rhs }
    }
    // 默认 raw 为等号左侧，等号右侧为 0，目前只支持单变量方程（TODO）
    pub fn from(raw: NumericExpression) -> Option<Self> {
        match raw {
            NumericExpression::Constant(number) => Some(LinearEquation {
                coef: Number::integer(0),
                rhs: number,
            }),
            NumericExpression::Variable(_) => Some(LinearEquation {
                coef: Number::integer(1),
                rhs: Number::integer(0),
            }),
            NumericExpression::Negation(numeric_expression) => {
                LinearEquation::from(-(*numeric_expression))
            }
            NumericExpression::Addition(numeric_bucket) => {
                let linear_list = numeric_bucket
                    .iter()
                    .map(|e| LinearEquation::from(e))
                    .collect::<Vec<_>>();

                // 如果有 None 直接返回 None
                if linear_list.iter().any(|l| l.is_none()) {
                    return None;
                }

                let linear_list = linear_list
                    .into_iter()
                    .map(|l| l.unwrap())
                    .collect::<Vec<_>>();

                Some(linear_list.into_iter().fold(
                    LinearEquation::from(NumericExpression::Constant(Number::integer(0))).unwrap(),
                    |acc, linear| LinearEquation::addition(&acc, &linear),
                ))
            }
            NumericExpression::Multiplication(numeric_bucket) => {
                // 统计 bucket 中所有的变量转换成的方程
                let linear_list = numeric_bucket
                    .iter()
                    .map(|e| LinearEquation::from(e))
                    .collect::<Vec<_>>();

                // 如果有 None 直接返回 None
                if linear_list.iter().any(|l| l.is_none()) {
                    return None;
                }

                let linear_list = linear_list
                    .into_iter()
                    .map(|l| l.unwrap())
                    .collect::<Vec<_>>();

                // 统计 linear_list 中包含变量的项
                let variable_linear_list = linear_list
                    .iter()
                    .filter(|linear| linear.coef != Number::integer(0))
                    .collect::<Vec<_>>();

                match variable_linear_list.len() {
                    0 => {
                        let product = linear_list
                            .iter()
                            .fold(Number::integer(1), |acc, l| acc * l.rhs.clone());

                        Some(LinearEquation {
                            coef: Number::integer(0),
                            rhs: product,
                        })
                    }
                    1 => {
                        let var_term = variable_linear_list[0];
                        // 计算所有常数项的积
                        let const_product = linear_list
                            .iter()
                            .filter(|l| l.coef.is_zero())
                            .fold(Number::integer(1), |acc, l| acc * l.rhs.clone());

                        // 结果：(a * const_product)x + (b * const_product)
                        Some(LinearEquation {
                            coef: &var_term.coef * &const_product,
                            rhs: &var_term.rhs * &const_product,
                        })
                    }
                    // Non-linear expression: multiplication of multiple variables is not supported
                    _ => None,
                }
            }
            // Non-linear expression: power is not supported
            NumericExpression::Power { .. } => None,
            // 先不考虑分段函数
            NumericExpression::Piecewise { cases, otherwise } => todo!(),
        }
    }

    pub fn addition(linear1: &LinearEquation, linear2: &LinearEquation) -> Self {
        LinearEquation {
            coef: &linear1.coef + &linear2.coef,
            rhs: &linear1.rhs + &linear2.rhs,
        }
    }
}

impl EquationTrait for LinearEquation {
    fn solve(&self) -> Option<SemanticExpression> {
        let mut result =
            -SemanticExpression::numeric(NumericExpression::constant(self.rhs.clone()))
                / SemanticExpression::numeric(NumericExpression::constant(self.coef.clone()));

        optimize(&mut result);
        Some(result)
    }
}
