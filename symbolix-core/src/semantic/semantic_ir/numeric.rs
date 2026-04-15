use crate::lexer::constant::Number;
use crate::lexer::NumberTrait;
use crate::optimizer::flatten_numeric;
use crate::semantic::bucket::NumericBucket;
use crate::semantic::semantic_ir::LogicalExpression;
use crate::semantic::variable::Variable;
use crate::{
    impl_numeric_ir_binary_operation, impl_numeric_ir_numeric_operation,
    impl_numeric_ir_unary_operation, logical_bucket, numeric_bucket,
};
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum NumericExpression {
    Constant(Number),
    Variable(Variable),
    Negation(Box<NumericExpression>),
    Addition(NumericBucket),
    Multiplication(NumericBucket), // a/b = a * b^(-1)
    Power {
        base: Box<NumericExpression>,
        exponent: Box<NumericExpression>, // 是否允许任意表达式？允许：超越函数；不允许：仅允许常数指数
    },
    Piecewise {
        cases: Vec<(LogicalExpression, NumericExpression)>,
        otherwise: Option<Box<NumericExpression>>,
    },
}

impl NumericExpression {
    pub fn constant(number: Number) -> NumericExpression {
        NumericExpression::Constant(number)
    }

    // 为类型建立统一 trait
    pub fn compatible_constant(number: impl NumberTrait) -> NumericExpression {
        NumericExpression::Constant(Number::common_build(number))
    }

    pub fn variable(variable: Variable) -> NumericExpression {
        NumericExpression::Variable(variable)
    }

    pub fn negation(expr: &NumericExpression) -> NumericExpression {
        match expr {
            NumericExpression::Constant(n) => NumericExpression::Constant(-n),
            NumericExpression::Variable(_) => NumericExpression::Negation(Box::new(expr.clone())),
            NumericExpression::Negation(inner) => *inner.clone(),
            NumericExpression::Addition(v) => {
                let negated_terms: NumericBucket = v
                    .iter()
                    .map(|term| NumericExpression::negation(&term))
                    .collect();
                NumericExpression::Addition(negated_terms)
            }
            NumericExpression::Multiplication(v) => NumericExpression::multiplication(
                &NumericExpression::Multiplication(v.clone()),
                &NumericExpression::Constant(Number::integer(-1)),
            ),
            NumericExpression::Power { .. } => NumericExpression::Negation(Box::new(expr.clone())),
            NumericExpression::Piecewise { cases, otherwise } => {
                let new_cases: Vec<(LogicalExpression, NumericExpression)> = cases
                    .into_iter()
                    .map(|(cond, num)| (cond.clone(), NumericExpression::negation(num)))
                    .collect();
                let new_otherwise = otherwise
                    .as_ref()
                    .map(|boxed| Box::new(NumericExpression::negation(boxed.as_ref())));
                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
        }
    }

    pub fn addition(term1: &NumericExpression, term2: &NumericExpression) -> NumericExpression {
        match (term1, term2) {
            (
                NumericExpression::Piecewise {
                    cases: cases1,
                    otherwise: otherwise1,
                },
                NumericExpression::Piecewise {
                    cases: cases2,
                    otherwise: otherwise2,
                },
            ) => {
                let mut new_cases = Vec::new();

                // 先算 otherwise × otherwise
                let new_otherwise = match (otherwise1, otherwise2) {
                    (Some(o1), Some(o2)) => Some(Box::new(NumericExpression::addition(
                        o1.as_ref(),
                        o2.as_ref(),
                    ))),
                    _ => None,
                };

                // cases1 × cases2
                for (cond1, num1) in cases1.iter() {
                    for (cond2, num2) in cases2.iter() {
                        new_cases.push((
                            LogicalExpression::And(logical_bucket![cond1.clone(), cond2.clone()]),
                            NumericExpression::addition(num1, num2),
                        ));
                    }

                    // cases1 × otherwise2
                    if let Some(o2) = otherwise2.as_ref() {
                        new_cases.push((
                            cond1.clone(),
                            NumericExpression::addition(num1, o2.as_ref()),
                        ));
                    }
                }

                // otherwise1 × cases2
                if let Some(o1) = otherwise1.as_ref() {
                    for (cond2, num2) in cases2.iter() {
                        new_cases.push((cond2.clone(), NumericExpression::addition(o1, num2)));
                    }
                }

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (NumericExpression::Piecewise { cases, otherwise }, r) => {
                let new_cases = cases
                    .iter()
                    .map(|(cond, num)| (cond.clone(), NumericExpression::addition(num, r)))
                    .collect();

                let new_otherwise = otherwise
                    .as_ref()
                    .map(|o| Box::new(NumericExpression::addition(o.as_ref(), r)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (l, NumericExpression::Piecewise { cases, otherwise }) => {
                let new_cases = cases
                    .iter()
                    .map(|(cond, num)| (cond.clone(), NumericExpression::addition(l, num)))
                    .collect();

                let new_otherwise = otherwise
                    .as_ref()
                    .map(|o| Box::new(NumericExpression::addition(l, o)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (NumericExpression::Addition(v1), NumericExpression::Addition(v2)) => {
                let mut new_bucket = v1.clone();
                new_bucket.extend(v2);
                NumericExpression::Addition(new_bucket)
            }
            (NumericExpression::Addition(v), NumericExpression::Constant(n)) => {
                let mut combined = numeric_bucket![NumericExpression::Constant(n.clone())];
                combined.extend(v);
                NumericExpression::Addition(combined)
            }
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {
                // 常量折叠
                NumericExpression::constant(c1 + c2)
            }
            (l, NumericExpression::Constant(c2)) => {
                // 常量放左侧
                NumericExpression::Addition(numeric_bucket![
                    NumericExpression::Constant(c2.clone()),
                    l.clone()
                ])
            }
            (NumericExpression::Addition(v), r) => {
                let mut new_bucket = numeric_bucket![r.clone()];
                new_bucket.extend(v);
                NumericExpression::Addition(new_bucket)
            }
            (l, NumericExpression::Addition(v)) => {
                let mut combined = numeric_bucket![l.clone()];
                combined.extend(v);
                NumericExpression::Addition(combined)
            }
            (l, r) => NumericExpression::Addition(numeric_bucket![l.clone(), r.clone()]),
        }
    }

    pub fn subtraction(
        minuend: &NumericExpression,
        subtrahend: &NumericExpression,
    ) -> NumericExpression {
        NumericExpression::addition(minuend, &NumericExpression::negation(subtrahend))
    }

    // TODO 优化，最小化克隆，考虑优化方向 `Rc<NumericExpression>`
    pub fn multiplication(
        term1: &NumericExpression,
        term2: &NumericExpression,
    ) -> NumericExpression {
        match (term1, term2) {
            (
                NumericExpression::Piecewise {
                    cases: cases1,
                    otherwise: otherwise1,
                },
                NumericExpression::Piecewise {
                    cases: cases2,
                    otherwise: otherwise2,
                },
            ) => {
                let mut new_cases = Vec::new();

                // 先计算 otherwise × otherwise（避免 moved value）
                let new_otherwise = match (otherwise1.as_ref(), otherwise2.as_ref()) {
                    (Some(o1), Some(o2)) => Some(Box::new(NumericExpression::multiplication(
                        o1.as_ref(),
                        o2.as_ref(),
                    ))),
                    _ => None,
                };

                // cases1 × cases2
                for (cond1, num1) in cases1.iter() {
                    for (cond2, num2) in cases2 {
                        new_cases.push((
                            LogicalExpression::And(logical_bucket![cond1.clone(), cond2.clone()]),
                            NumericExpression::multiplication(num1, num2),
                        ));
                    }

                    // cases1 × otherwise2
                    if let Some(o2) = otherwise2.as_ref() {
                        new_cases.push((
                            cond1.clone(),
                            NumericExpression::multiplication(num1, o2.as_ref()),
                        ));
                    }
                }

                // otherwise1 × cases2
                if let Some(o1) = otherwise1.as_ref() {
                    for (cond2, num2) in cases2 {
                        new_cases.push((
                            cond2.clone(),
                            NumericExpression::multiplication(o1.as_ref(), num2),
                        ));
                    }
                }

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (NumericExpression::Piecewise { cases, otherwise }, r) => {
                let new_cases = cases
                    .iter()
                    .map(|(cond, num)| (cond.clone(), NumericExpression::multiplication(num, r)))
                    .collect();

                let new_otherwise = otherwise
                    .as_ref()
                    .map(|o| Box::new(NumericExpression::multiplication(o.as_ref(), r)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (l, NumericExpression::Piecewise { cases, otherwise }) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| (cond.clone(), NumericExpression::multiplication(l, num)))
                    .collect();

                let new_otherwise = otherwise
                    .as_ref()
                    .map(|o| Box::new(NumericExpression::multiplication(l, o.as_ref())));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (NumericExpression::Multiplication(v1), NumericExpression::Multiplication(v2)) => {
                let mut new_bucket = v1.clone();
                new_bucket.extend(v2);
                NumericExpression::Multiplication(new_bucket)
            }
            (NumericExpression::Multiplication(v), NumericExpression::Constant(n)) => {
                let mut combined = numeric_bucket![NumericExpression::Constant(n.clone())];
                combined.extend(&v);
                NumericExpression::Multiplication(combined)
            }
            (NumericExpression::Multiplication(v), r) => {
                let mut new_bucket = numeric_bucket![r.clone()];
                new_bucket.extend(v);
                NumericExpression::Multiplication(new_bucket)
            }
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {
                // 常量折叠
                NumericExpression::Constant(c1 * c2)
            }
            (l, NumericExpression::Constant(c2)) => {
                NumericExpression::Multiplication(numeric_bucket![
                    NumericExpression::Constant(c2.clone()),
                    l.clone()
                ])
            }
            (l, NumericExpression::Multiplication(v)) => {
                let mut combined = numeric_bucket![l.clone()];
                combined.extend(v);
                NumericExpression::Multiplication(combined)
            }
            (l, r) => NumericExpression::Multiplication(numeric_bucket![l.clone(), r.clone()]),
        }
    }

    pub fn division(
        dividend: &NumericExpression,
        divisor: &NumericExpression,
    ) -> NumericExpression {
        NumericExpression::multiplication(
            dividend,
            &NumericExpression::power(divisor, &NumericExpression::Constant(Number::integer(-1))),
        )
    }

    pub fn power(base: &NumericExpression, exponent: &NumericExpression) -> NumericExpression {
        match (base, exponent) {
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {
                NumericExpression::Constant(Number::float(c1.to_float().powf(c2.to_float())))
            }
            (
                NumericExpression::Power {
                    base: b,
                    exponent: e,
                },
                exponent,
            ) => {
                let new_exponent = NumericExpression::multiplication(e.as_ref(), exponent);
                NumericExpression::Power {
                    base: b.clone(),
                    exponent: Box::new(new_exponent),
                }
            }
            (NumericExpression::Multiplication(v), exponent) => {
                let new_factors: NumericBucket = v
                    .iter()
                    .map(|factor| NumericExpression::power(&factor, exponent))
                    .collect();
                NumericExpression::Multiplication(new_factors)
            }
            (base, exponent) => NumericExpression::Power {
                base: Box::new(base.clone()),
                exponent: Box::new(exponent.clone()),
            },
        }
    }

    // TODO 还需要处理 otherwise 的嵌套
    pub fn piecewise(
        cases: Vec<(LogicalExpression, NumericExpression)>,
        otherwise: Option<NumericExpression>,
    ) -> NumericExpression {
        // 处理三元表达式嵌套的情况
        let mut flattened_cases: Vec<(LogicalExpression, NumericExpression)> = Vec::new();

        for (cond, num) in cases {
            match num {
                NumericExpression::Piecewise {
                    cases: inner_cases,
                    otherwise: inner_otherwise,
                } => {
                    for (inner_cond, inner_num) in inner_cases {
                        let combined_cond = &cond & &inner_cond;
                        flattened_cases.push((combined_cond, inner_num));
                    }
                    if let Some(inner_o) = inner_otherwise {
                        let combined_cond = &cond & true;
                        flattened_cases.push((combined_cond, *inner_o));
                    }
                }
                _ => {
                    flattened_cases.push((cond, num));
                }
            }
        }

        NumericExpression::Piecewise {
            cases: flattened_cases,
            otherwise: otherwise.map(Box::new),
        }
    }

    /// 展开表达式，对于数值表达式来说将表达式中的括号去掉，将嵌套的表达式展开。
    pub fn flatten(self) -> Self {
        flatten_numeric(self)
    }

    /// 规约表达式
    pub fn factor(&mut self) {}

    pub fn substitute(
        &self,
        target: &crate::semantic::variable::Variable,
        replacement: Option<&NumericExpression>,
    ) -> NumericExpression {
        match self {
            NumericExpression::Constant(_) => self.clone(),
            NumericExpression::Variable(variable) => {
                if variable == target {
                    replacement.cloned().unwrap_or_else(|| self.clone())
                } else {
                    self.clone()
                }
            }
            NumericExpression::Negation(inner) => {
                NumericExpression::negation(&inner.substitute(target, replacement))
            }
            NumericExpression::Addition(bucket) => {
                let mut iter = bucket.iter().map(|expr| expr.substitute(target, replacement));
                match iter.next() {
                    Some(first) => iter.fold(first, |acc, expr| acc + expr),
                    None => NumericExpression::constant(Number::integer(0)),
                }
            }
            NumericExpression::Multiplication(bucket) => {
                let mut iter = bucket.iter().map(|expr| expr.substitute(target, replacement));
                match iter.next() {
                    Some(first) => iter.fold(first, |acc, expr| acc * expr),
                    None => NumericExpression::constant(Number::integer(1)),
                }
            }
            NumericExpression::Power { base, exponent } => NumericExpression::power(
                &base.substitute(target, replacement),
                &exponent.substitute(target, replacement),
            ),
            NumericExpression::Piecewise { cases, otherwise } => NumericExpression::piecewise(
                cases
                    .iter()
                    .map(|(condition, expr)| {
                        (
                            condition.substitute(
                                target,
                                replacement.map(|expr| {
                                    crate::semantic::semantic_ir::SemanticExpression::numeric(
                                        expr.clone(),
                                    )
                                }).as_ref(),
                            ),
                            expr.substitute(target, replacement),
                        )
                    })
                    .collect(),
                otherwise
                    .as_ref()
                    .map(|expr| expr.substitute(target, replacement)),
            ),
        }
    }
}

impl_numeric_ir_unary_operation!(Neg, neg, negation);
impl_numeric_ir_binary_operation!(Add, add, addition);
impl_numeric_ir_binary_operation!(Sub, sub, subtraction);
impl_numeric_ir_binary_operation!(Mul, mul, multiplication);
impl_numeric_ir_binary_operation!(Div, div, division);
impl_numeric_ir_numeric_operation!(Add, add, addition, i32, i64, f32, f64, u32, u64);
impl_numeric_ir_numeric_operation!(Sub, sub, subtraction, i32, i64, f32, f64, u32, u64);
impl_numeric_ir_numeric_operation!(Mul, mul, multiplication, i32, i64, f32, f64, u32, u64);
impl_numeric_ir_numeric_operation!(Div, div, division, i32, i64, f32, f64, u32, u64);

impl fmt::Display for NumericExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NumericExpression::Constant(c) => {
                write!(f, "{}", c)
            }
            NumericExpression::Variable(v) => {
                write!(f, "{}", v)
            }
            NumericExpression::Negation(n) => {
                write!(f, "-({})", n)
            }
            NumericExpression::Addition(bucket) => {
                let terms: Vec<String> = bucket.iter().map(|term| format!("{}", term)).collect();
                write!(f, "({})", terms.join(" + "))
            }
            NumericExpression::Multiplication(bucket) => {
                let factors: Vec<String> =
                    bucket.iter().map(|factor| format!("{}", factor)).collect();
                write!(f, "({})", factors.join(" * "))
            }
            NumericExpression::Power { base, exponent } => {
                write!(f, "({})^({})", base, exponent)
            }
            NumericExpression::Piecewise { cases, otherwise } => {
                for case in cases {
                    write!(f, "{}, {};\n", case.1, case.0)?;
                }
                match otherwise.as_ref() {
                    Some(expr) => write!(f, "{}, other;", expr),
                    None => write!(f, ""),
                }
            }
        }
    }
}

