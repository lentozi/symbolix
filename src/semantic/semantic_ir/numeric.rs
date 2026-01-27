use crate::lexer::constant::Number;
use crate::semantic::bucket::NumericBucket;
use crate::semantic::semantic_ir::LogicalExpression;
use crate::semantic::variable::Variable;
use crate::{logical_bucket, numeric_bucket};
use std::fmt;
use std::fmt::Formatter;
use tree_drawer::tree::OwnedTree;

#[derive(Debug, Clone, PartialEq)]
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

    pub fn compatible_constant(number: f64) -> NumericExpression {
        NumericExpression::Constant(Number::float(number))
    }

    pub fn variable(variable: Variable) -> NumericExpression {
        NumericExpression::Variable(variable)
    }

    pub fn negation(expr: NumericExpression) -> NumericExpression {
        match expr {
            NumericExpression::Constant(n) => NumericExpression::Constant(-n),
            NumericExpression::Variable(_) => NumericExpression::Negation(Box::new(expr)),
            NumericExpression::Negation(inner) => *inner,
            NumericExpression::Addition(v) => {
                let negated_terms: NumericBucket = v
                    .into_iter()
                    .map(|term| NumericExpression::negation(term))
                    .collect();
                NumericExpression::Addition(negated_terms)
            }
            NumericExpression::Multiplication(v) => NumericExpression::multiplication(
                NumericExpression::Multiplication(v),
                NumericExpression::Constant(Number::integer(-1)),
            ),
            NumericExpression::Power { .. } => NumericExpression::Negation(Box::new(expr)),
            NumericExpression::Piecewise { cases, otherwise } => {
                let new_cases: Vec<(LogicalExpression, NumericExpression)> = cases
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::negation(num)))
                    .collect();
                let new_otherwise =
                    otherwise.map(|boxed| Box::new(NumericExpression::negation(*boxed)));
                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
        }
    }

    pub fn addition(term1: NumericExpression, term2: NumericExpression) -> NumericExpression {
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
                let new_otherwise = match (&otherwise1, &otherwise2) {
                    (Some(o1), Some(o2)) => Some(Box::new(NumericExpression::addition(
                        (**o1).clone(),
                        (**o2).clone(),
                    ))),
                    _ => None,
                };

                // cases1 × cases2
                for (cond1, num1) in cases1 {
                    for (cond2, num2) in &cases2 {
                        new_cases.push((
                            LogicalExpression::And(logical_bucket![cond1.clone(), cond2.clone()]),
                            NumericExpression::addition(num1.clone(), num2.clone()),
                        ));
                    }

                    // cases1 × otherwise2
                    if let Some(ref o2) = otherwise2 {
                        new_cases.push((cond1, NumericExpression::addition(num1, (**o2).clone())));
                    }
                }

                // otherwise1 × cases2
                if let Some(o1) = otherwise1 {
                    for (cond2, num2) in cases2 {
                        new_cases.push((cond2, NumericExpression::addition(*o1.clone(), num2)));
                    }
                }

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (NumericExpression::Piecewise { cases, otherwise }, r) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::addition(num, r.clone())))
                    .collect();

                let new_otherwise = otherwise.map(|o| Box::new(NumericExpression::addition(*o, r)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (l, NumericExpression::Piecewise { cases, otherwise }) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::addition(l.clone(), num)))
                    .collect();

                let new_otherwise = otherwise.map(|o| Box::new(NumericExpression::addition(l, *o)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (NumericExpression::Addition(mut v1), NumericExpression::Addition(v2)) => {
                v1.extend(v2);
                NumericExpression::Addition(v1)
            }
            (NumericExpression::Addition(v), NumericExpression::Constant(n)) => {
                let mut combined = numeric_bucket![NumericExpression::Constant(n)];
                combined.extend(v);
                NumericExpression::Addition(combined)
            }
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {
                // 常量折叠
                NumericExpression::constant(c1 + c2)
            }
            (l, NumericExpression::Constant(c2)) => {
                // 常量放左侧
                NumericExpression::Addition(numeric_bucket![NumericExpression::Constant(c2), l])
            }
            (NumericExpression::Addition(mut v), r) => {
                v.push(r);
                NumericExpression::Addition(v)
            }
            (l, NumericExpression::Addition(v)) => {
                let mut combined = numeric_bucket![l];
                combined.extend(v);
                NumericExpression::Addition(combined)
            }
            (l, r) => NumericExpression::Addition(numeric_bucket![l, r]),
        }
    }

    pub fn subtraction(
        minuend: NumericExpression,
        subtrahend: NumericExpression,
    ) -> NumericExpression {
        NumericExpression::addition(minuend, NumericExpression::negation(subtrahend))
    }

    pub fn multiplication(term1: NumericExpression, term2: NumericExpression) -> NumericExpression {
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
                let new_otherwise = match (&otherwise1, &otherwise2) {
                    (Some(o1), Some(o2)) => Some(Box::new(NumericExpression::multiplication(
                        (**o1).clone(),
                        (**o2).clone(),
                    ))),
                    _ => None,
                };

                // cases1 × cases2
                for (cond1, num1) in cases1 {
                    for (cond2, num2) in &cases2 {
                        new_cases.push((
                            LogicalExpression::And(logical_bucket![cond1.clone(), cond2.clone()]),
                            NumericExpression::multiplication(num1.clone(), num2.clone()),
                        ));
                    }

                    // cases1 × otherwise2
                    if let Some(ref o2) = otherwise2 {
                        new_cases.push((
                            cond1,
                            NumericExpression::multiplication(num1, (**o2).clone()),
                        ));
                    }
                }

                // otherwise1 × cases2
                if let Some(o1) = otherwise1 {
                    for (cond2, num2) in cases2 {
                        new_cases.push((
                            cond2,
                            NumericExpression::multiplication((*o1).clone(), num2),
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
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::multiplication(num, r.clone())))
                    .collect();

                let new_otherwise =
                    otherwise.map(|o| Box::new(NumericExpression::multiplication(*o, r)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (l, NumericExpression::Piecewise { cases, otherwise }) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::multiplication(l.clone(), num)))
                    .collect();

                let new_otherwise =
                    otherwise.map(|o| Box::new(NumericExpression::multiplication(l, *o)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (NumericExpression::Multiplication(mut v1), NumericExpression::Multiplication(v2)) => {
                v1.extend(v2);
                NumericExpression::Multiplication(v1)
            }
            (NumericExpression::Multiplication(v), NumericExpression::Constant(n)) => {
                let mut combined = numeric_bucket![NumericExpression::Constant(n)];
                combined.extend(v);
                NumericExpression::Multiplication(combined)
            }
            (NumericExpression::Multiplication(mut v), r) => {
                v.push(r);
                NumericExpression::Multiplication(v)
            }
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {
                // 常量折叠
                NumericExpression::Constant(c1 * c2)
            }
            (l, NumericExpression::Constant(c2)) => {
                // 常量放左侧
                NumericExpression::Multiplication(numeric_bucket![
                    NumericExpression::Constant(c2),
                    l
                ])
            }
            (l, NumericExpression::Multiplication(v)) => {
                let mut combined = numeric_bucket![l];
                combined.extend(v);
                NumericExpression::Multiplication(combined)
            }
            (l, r) => NumericExpression::Multiplication(numeric_bucket![l, r]),
        }
    }

    pub fn division(dividend: NumericExpression, divisor: NumericExpression) -> NumericExpression {
        NumericExpression::multiplication(
            dividend,
            NumericExpression::power(divisor, NumericExpression::Constant(Number::integer(-1))),
        )
    }

    pub fn power(base: NumericExpression, exponent: NumericExpression) -> NumericExpression {
        match base {
            NumericExpression::Power {
                base: b,
                exponent: e,
            } => {
                let new_exponent = NumericExpression::multiplication(*e, exponent);
                NumericExpression::Power {
                    base: b,
                    exponent: Box::new(new_exponent),
                }
            }
            NumericExpression::Multiplication(v) => {
                let new_factors: NumericBucket = v
                    .into_iter()
                    .map(|factor| NumericExpression::power(factor, exponent.clone()))
                    .collect();
                NumericExpression::Multiplication(new_factors)
            }
            _ => NumericExpression::Power {
                base: Box::new(base),
                exponent: Box::new(exponent),
            },
        }
    }

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
                        let combined_cond = LogicalExpression::and(cond.clone(), inner_cond);
                        flattened_cases.push((combined_cond, inner_num));
                    }
                    if let Some(inner_o) = inner_otherwise {
                        let combined_cond =
                            LogicalExpression::and(cond.clone(), LogicalExpression::constant(true));
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

    pub fn normalize(&mut self) {
        match self {
            NumericExpression::Addition(bucket) => {
                for expr in &mut bucket.expressions {
                    expr.normalize();
                }
                bucket.execute_constant(true);
                bucket.remove_zero();
            }
            NumericExpression::Multiplication(bucket) => {
                for expr in &mut bucket.expressions {
                    expr.normalize();
                }
                bucket.execute_constant(true);
                bucket.remove_one();

                if bucket.contains_zero() {
                    *self = NumericExpression::Constant(Number::Integer(0));
                }
            }
            NumericExpression::Negation(inner) => {
                inner.normalize();
            }
            NumericExpression::Power { base, exponent } => {
                base.normalize();
                exponent.normalize();
            }
            NumericExpression::Constant(_) => {}
            NumericExpression::Variable(_) => {}
            NumericExpression::Piecewise { cases, otherwise } => {
                for (cond, num) in cases {
                    cond.normalize();
                    num.normalize();
                }
                if let Some(otherwise) = otherwise {
                    otherwise.normalize();
                }
            }
        }
    }
}

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

impl NumericExpression {
    pub fn to_owned_tree(&self) -> OwnedTree {
        match self {
            NumericExpression::Constant(n) => OwnedTree::new(format!("{n}")),

            NumericExpression::Variable(v) => OwnedTree::new(format!("{v}")),

            NumericExpression::Negation(expr) => {
                OwnedTree::new("-".to_string()).with_child(expr.to_owned_tree())
            }

            NumericExpression::Addition(bucket) => {
                let mut node = OwnedTree::new("+".to_string());
                for term in bucket.iter() {
                    node = node.with_child(term.to_owned_tree());
                }
                node
            }

            NumericExpression::Multiplication(bucket) => {
                let mut node = OwnedTree::new("*".to_string());
                for factor in bucket.iter() {
                    node = node.with_child(factor.to_owned_tree());
                }
                node
            }

            NumericExpression::Power { base, exponent } => OwnedTree::new("^".to_string())
                .with_child(base.to_owned_tree())
                .with_child(exponent.to_owned_tree()),

            NumericExpression::Piecewise { cases, otherwise } => {
                let mut node = OwnedTree::new("piecewise".to_string());

                for (cond, expr) in cases {
                    node = node.with_child(
                        OwnedTree::new("case".to_string())
                            .with_child(cond.to_owned_tree())
                            .with_child(expr.to_owned_tree()),
                    );
                }

                if let Some(other) = otherwise {
                    node = node.with_child(
                        OwnedTree::new("otherwise".to_string()).with_child(other.to_owned_tree()),
                    );
                }

                node
            }
        }
    }
}
