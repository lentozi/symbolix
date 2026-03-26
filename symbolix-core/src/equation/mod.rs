use crate::{
    equation::linear::LinearEquation,
    lexer::symbol::{Relation, Symbol},
    optimizer::optimize,
    semantic::semantic_ir::{
        logic::LogicalExpression, numeric::NumericExpression, SemanticExpression,
    },
};

pub mod linear;

pub trait EquationTrait {
    fn solve(&self) -> Option<SemanticExpression>;
}

pub enum Equation {
    SingleVariableEquation(SingleVariableEquation),
    MultiVariableEquation(MultiVariableEquation),
}

pub enum SingleVariableEquation {
    LinearEquation(LinearEquation),
    PolynomialEquation(PolynomialEquation),
}

pub enum MultiVariableEquation {}

pub struct PolynomialEquation {}

impl Equation {
    pub fn new(raw: SemanticExpression) -> Self {
        if let SemanticExpression::Logical(LogicalExpression::Relation {
            left,
            operator: Symbol::Relation(Relation::Equal),
            right,
        }) = raw
        {
            let mut lhs =
                SemanticExpression::numeric(NumericExpression::subtraction(*left, *right));
            optimize(&mut lhs);
            // 判断方程类型
            if let SemanticExpression::Numeric(numeric) = lhs {
                let linear_eq = LinearEquation::new(numeric);
                return Self::SingleVariableEquation(SingleVariableEquation::LinearEquation(
                    linear_eq,
                ));
            }
        }
        panic!("invalid equation format");
    }

    pub fn solve(&self) -> Option<SemanticExpression> {
        match self {
            Equation::SingleVariableEquation(single) => single.solve(),
            Equation::MultiVariableEquation(multi) => multi.solve(),
        }
    }
}

impl EquationTrait for SingleVariableEquation {
    fn solve(&self) -> Option<SemanticExpression> {
        match self {
            SingleVariableEquation::LinearEquation(linear) => linear.solve(),
            SingleVariableEquation::PolynomialEquation(_) => None,
        }
    }
}

impl EquationTrait for MultiVariableEquation {
    fn solve(&self) -> Option<SemanticExpression> {
        None
    }
}
