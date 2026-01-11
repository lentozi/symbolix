use crate::lexer::constant::Number;
use crate::lexer::symbol::{Relation, Symbol};
use crate::semantic::variable::Variable;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticExpression {
    Numeric(NumericExpression),
    Logical(LogicalExpression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NumericExpression {
    Constant(Number),
    Variable(Variable),
    Negation(Box<NumericExpression>),
    // TODO 基于桶排序构造数据结构替代 Vec
    Addition(Vec<NumericExpression>),
    Multiplication(Vec<NumericExpression>), // a/b = a * b^(-1)
    Power {
        base: Box<NumericExpression>,
        exponent: Box<NumericExpression>, // 是否允许任意表达式？允许：超越函数；不允许：仅允许常数指数
    },
    Piecewise {
        cases: Vec<(LogicalExpression, NumericExpression)>,
        otherwise: Option<Box<NumericExpression>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalExpression {
    Constant(bool),
    Variable(Variable),
    Not(Box<LogicalExpression>),
    And(Vec<LogicalExpression>),
    Or(Vec<LogicalExpression>),
    Relation {
        left: Box<NumericExpression>,
        operator: Symbol,
        right: Box<NumericExpression>,
    },
}

impl NumericExpression {
    pub fn constant(number: Number) -> NumericExpression {
        NumericExpression::Constant(number)
    }

    pub fn variable(variable: Variable) -> NumericExpression {
        NumericExpression::Variable(variable)
    }

    pub fn negation(expr: NumericExpression) -> NumericExpression {
        match expr {
            NumericExpression::Constant(_) |
            NumericExpression::Variable(_) => NumericExpression::Negation(Box::new(expr)),
            NumericExpression::Negation(inner) => *inner,
            NumericExpression::Addition(v) => {
                let negated_terms: Vec<NumericExpression> = v.into_iter()
                    .map(|term| NumericExpression::negation(term))
                    .collect();
                NumericExpression::Addition(negated_terms)
            }
            NumericExpression::Multiplication(v) => {
                NumericExpression::multiplication(NumericExpression::Multiplication(v), NumericExpression::Constant(Number::integer(-1)))
            }
            NumericExpression::Power { .. } => NumericExpression::Negation(Box::new(expr)),
            NumericExpression::Piecewise { cases, otherwise } => {
                let new_cases: Vec<(LogicalExpression, NumericExpression)> = cases
                    .into_iter()
                    .map(|(cond, num)| (cond, NumericExpression::negation(num)))
                    .collect();
                let new_otherwise = otherwise.map(|boxed| Box::new(NumericExpression::negation(*boxed)));
                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
        }
    }

    pub fn addition(term1: NumericExpression, term2: NumericExpression) -> NumericExpression {
        match (term1, term2) {
            (NumericExpression::Piecewise { cases: cases1, otherwise: otherwise1 },
                NumericExpression::Piecewise { cases: cases2, otherwise: otherwise2 }) => {
                let mut new_cases = Vec::new();

                // 先算 otherwise × otherwise
                let new_otherwise = match (&otherwise1, &otherwise2) {
                    (Some(o1), Some(o2)) => {
                        Some(Box::new(NumericExpression::addition((**o1).clone(), (**o2).clone())))
                    }
                    _ => None,
                };

                // cases1 × cases2
                for (cond1, num1) in cases1 {
                    for (cond2, num2) in &cases2 {
                        new_cases.push((
                            LogicalExpression::And(vec![cond1.clone(), cond2.clone()]),
                            NumericExpression::addition(num1.clone(), num2.clone()),
                        ));
                    }

                    // cases1 × otherwise2
                    if let Some(ref o2) = otherwise2 {
                        new_cases.push((
                            cond1,
                            NumericExpression::addition(num1, (**o2).clone()),
                        ));
                    }
                }

                // otherwise1 × cases2
                if let Some(o1) = otherwise1 {
                    for (cond2, num2) in cases2 {
                        new_cases.push((
                            cond2,
                            NumericExpression::addition(*o1.clone(), num2),
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
                    .map(|(cond, num)| (cond, NumericExpression::addition(num, r.clone())))
                    .collect();

                let new_otherwise = otherwise
                    .map(|o| Box::new(NumericExpression::addition(*o, r)));

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

                let new_otherwise = otherwise
                    .map(|o| Box::new(NumericExpression::addition(l, *o)));

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
                let mut combined = vec![NumericExpression::Constant(n)];
                combined.extend(v);
                NumericExpression::Addition(combined)
            }
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {     // 常量折叠
                NumericExpression::constant(c1 + c2)
            }
            (l, NumericExpression::Constant(c2)) => {                       // 常量放左侧
                NumericExpression::Addition(vec![NumericExpression::Constant(c2), l])
            }
            (NumericExpression::Addition(mut v), r) => {
                v.push(r);
                NumericExpression::Addition(v)
            }
            (l, NumericExpression::Addition(v)) => {
                let mut combined = Vec::with_capacity(v.len() + 1);
                combined.push(l);
                combined.extend(v);
                NumericExpression::Addition(combined)
            }
            (l, r) => NumericExpression::Addition(vec![l, r]),
        }
    }


    pub fn multiplication(term1: NumericExpression, term2: NumericExpression) -> NumericExpression {
        match (term1, term2) {
            (NumericExpression::Piecewise { cases: cases1, otherwise: otherwise1 },
                NumericExpression::Piecewise { cases: cases2, otherwise: otherwise2 }) => {
                let mut new_cases = Vec::new();

                // 先计算 otherwise × otherwise（避免 moved value）
                let new_otherwise = match (&otherwise1, &otherwise2) {
                    (Some(o1), Some(o2)) => {
                        Some(Box::new(NumericExpression::multiplication(
                            (**o1).clone(),
                            (**o2).clone(),
                        )))
                    }
                    _ => None,
                };

                // cases1 × cases2
                for (cond1, num1) in cases1 {
                    for (cond2, num2) in &cases2 {
                        new_cases.push((
                            LogicalExpression::And(vec![cond1.clone(), cond2.clone()]),
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
                    .map(|(cond, num)| {
                        (cond, NumericExpression::multiplication(num, r.clone()))
                    })
                    .collect();

                let new_otherwise = otherwise
                    .map(|o| Box::new(NumericExpression::multiplication(*o, r)));

                NumericExpression::Piecewise {
                    cases: new_cases,
                    otherwise: new_otherwise,
                }
            }
            (l, NumericExpression::Piecewise { cases, otherwise }) => {
                let new_cases = cases
                    .into_iter()
                    .map(|(cond, num)| {
                        (cond, NumericExpression::multiplication(l.clone(), num))
                    })
                    .collect();

                let new_otherwise = otherwise
                    .map(|o| Box::new(NumericExpression::multiplication(l, *o)));

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
                let mut combined = vec![NumericExpression::Constant(n)];
                combined.extend(v);
                NumericExpression::Multiplication(combined)
            }
            (NumericExpression::Multiplication(mut v), r) => {
                v.push(r);
                NumericExpression::Multiplication(v)
            }
            (NumericExpression::Constant(c1), NumericExpression::Constant(c2)) => {         // 常量折叠
                NumericExpression::Constant(c1 * c2)
            }
            (l, NumericExpression::Constant(c2)) => {                           // 常量放左侧
                NumericExpression::Multiplication(vec![NumericExpression::Constant(c2), l])
            }
            (l, NumericExpression::Multiplication(v)) => {
                let mut combined = Vec::with_capacity(v.len() + 1);
                combined.push(l);
                combined.extend(v);
                NumericExpression::Multiplication(combined)
            }
            (l, r) => NumericExpression::Multiplication(vec![l, r]),
        }
    }

    pub fn power(base: NumericExpression, exponent: NumericExpression) -> NumericExpression {
        match base {
            NumericExpression::Power { base: b, exponent: e } => {
                let new_exponent = NumericExpression::multiplication(*e, exponent);
                NumericExpression::Power {
                    base: b,
                    exponent: Box::new(new_exponent),
                }
            }
            NumericExpression::Multiplication(v) => {
                let new_factors: Vec<NumericExpression> = v.into_iter()
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

    pub fn piecewise(cases: Vec<(LogicalExpression, NumericExpression)>, otherwise: Option<NumericExpression>) -> NumericExpression {
        // 处理三元表达式嵌套的情况
        let mut flattened_cases: Vec<(LogicalExpression, NumericExpression)> = Vec::new();

        for (cond, num) in cases {
            match num {
                NumericExpression::Piecewise { cases: inner_cases, otherwise: inner_otherwise } => {
                    for (inner_cond, inner_num) in inner_cases {
                        let combined_cond = LogicalExpression::and(cond.clone(), inner_cond);
                        flattened_cases.push((combined_cond, inner_num));
                    }
                    if let Some(inner_o) = inner_otherwise {
                        let combined_cond = LogicalExpression::and(cond.clone(), LogicalExpression::constant(true));
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
}

impl LogicalExpression {
    pub fn constant(value: bool) -> LogicalExpression {
        LogicalExpression::Constant(value)
    }

    pub fn variable(variable: Variable) -> LogicalExpression {
        LogicalExpression::Variable(variable)
    }

    pub fn not(expr: LogicalExpression) -> LogicalExpression {
        match expr {
            LogicalExpression::Constant(c) => LogicalExpression::Constant(!c),
            LogicalExpression::Variable(_) => LogicalExpression::Not(Box::new(expr)),
            LogicalExpression::Not(inner) => *inner,
            LogicalExpression::And(v) => {
                let negated_terms: Vec<LogicalExpression> = v.into_iter()
                    .map(|term| LogicalExpression::not(term))
                    .collect();
                LogicalExpression::Or(negated_terms)
            }
            LogicalExpression::Or(v) => {
                let negated_terms: Vec<LogicalExpression> = v.into_iter()
                    .map(|term| LogicalExpression::not(term))
                    .collect();
                LogicalExpression::And(negated_terms)
            }
            LogicalExpression::Relation { left, operator: relation, right } => match relation {
                Symbol::Relation(Relation::Equal) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::NotEqual), right
                },
                Symbol::Relation(Relation::NotEqual) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::Equal), right
                },
                Symbol::Relation(Relation::LessThan) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::GreaterEqual), right
                },
                Symbol::Relation(Relation::GreaterThan) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::LessEqual), right
                },
                Symbol::Relation(Relation::LessEqual) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::GreaterThan), right
                },
                Symbol::Relation(Relation::GreaterEqual) => LogicalExpression::Relation {
                    left, operator: Symbol::Relation(Relation::LessThan), right
                },
                _ => panic!("unsupported relation operator: {}", relation),
            }
        }
    }

    pub fn and(expr1: LogicalExpression, expr2: LogicalExpression) -> LogicalExpression {
        match (expr1, expr2) {
            // 常量折叠
            (LogicalExpression::Constant(c1), LogicalExpression::Constant(c2)) => {
                LogicalExpression::constant(c1 && c2)
            }
            (_, LogicalExpression::Constant(false)) => LogicalExpression::Constant(false),
            (l, LogicalExpression::Constant(true)) => l,
            // And + And
            (LogicalExpression::And(mut v1), LogicalExpression::And(v2)) => {
                v1.extend(v2);
                LogicalExpression::And(v1)
            }
            // And + r
            (LogicalExpression::And(mut v), r) => {
                v.push(r);
                LogicalExpression::And(v)
            }
            // l + And
            (l, LogicalExpression::And(v)) => {
                let mut combined = Vec::with_capacity(v.len() + 1);
                combined.push(l);
                combined.extend(v);
                LogicalExpression::And(combined)
            }
            // fallback
            (l, r) => LogicalExpression::And(vec![l, r]),
        }
    }

    pub fn or(expr1: LogicalExpression, expr2: LogicalExpression) -> LogicalExpression {
        match (expr1, expr2) {
            // 常量折叠
            (LogicalExpression::Constant(c1), LogicalExpression::Constant(c2)) => {
                LogicalExpression::constant(c1 || c2)
            }
            (_, LogicalExpression::Constant(true)) => LogicalExpression::Constant(true),
            (l, LogicalExpression::Constant(false)) => l,
            // Or + Or
            (LogicalExpression::Or(mut v1), LogicalExpression::Or(v2)) => {
                v1.extend(v2);
                LogicalExpression::Or(v1)
            }
            // Or + r
            (LogicalExpression::Or(mut v), r) => {
                v.push(r);
                LogicalExpression::Or(v)
            }
            // l + Or
            (l, LogicalExpression::Or(v)) => {
                let mut combined = Vec::with_capacity(v.len() + 1);
                combined.push(l);
                combined.extend(v);
                LogicalExpression::Or(combined)
            }
            (l, r) => LogicalExpression::Or(vec![l, r]),
        }
    }

    pub fn relation(left: NumericExpression, operator: Symbol, right: NumericExpression) -> LogicalExpression {
        LogicalExpression::Relation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }
}