use crate::optimizer::optimize;
use crate::{
    equation::EquationTrait,
    lexer::constant::Number,
    semantic::semantic_ir::{numeric::NumericExpression, SemanticExpression},
};

pub struct LinearEquation {
    _raw: NumericExpression,
    coef: Number,
    rhs: Number,
}

impl LinearEquation {
    pub fn new(raw: NumericExpression) -> Self {
        if let NumericExpression::Addition(bucket) = raw.clone() {
            let expressions = bucket.expressions;
            let constants = bucket.constants;
            let variables = bucket.variables;
            if expressions.len() == 1 && constants.len() == 1 && variables.is_empty() {
                let rhs = constants[0].clone();
                if let NumericExpression::Multiplication(mul_bucket) = expressions[0].clone() {
                    let expressions = mul_bucket.expressions;
                    let constants = mul_bucket.constants;
                    let variables = mul_bucket.variables;
                    if variables.len() == 1 && constants.len() == 1 && expressions.is_empty() {
                        let coef = constants[0].clone();
                        return Self {
                            _raw: raw,
                            coef,
                            rhs,
                        };
                    }
                }
            }
        }
        panic!("unexpected expression in linear equation");
    }

    pub fn addition(linear1: &LinearEquation, linear2: &LinearEquation) -> Self {
        LinearEquation {
            // TODO 这里 multiply 进行了两次
            _raw: NumericExpression::multiplication(linear1._raw, linear2._raw),
            coef: linear1.coef + linear2.coef,
            rhs: linear1.rhs + linear2.rhs,
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
