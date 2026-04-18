use crate::lexer::symbol::{Relation, Symbol};
use crate::semantic::bucket::LogicalBucket;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::variable::Variable;
use crate::{
    impl_logic_ir_binary_operation, impl_logic_ir_logic_operation, impl_logic_ir_unary_operation,
    logical_bucket,
};
use std::fmt;
use std::fmt::Formatter;
use std::ops::{BitAnd, BitOr, Not};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum LogicalExpression {
    Constant(bool),
    Variable(Variable),
    Not(Box<LogicalExpression>),
    And(LogicalBucket),
    Or(LogicalBucket),
    Relation {
        left: Box<NumericExpression>,
        operator: Symbol,
        right: Box<NumericExpression>,
    },
}

impl LogicalExpression {
    pub fn constant(value: bool) -> LogicalExpression {
        LogicalExpression::Constant(value)
    }

    pub fn variable(variable: Variable) -> LogicalExpression {
        LogicalExpression::Variable(variable)
    }

    pub fn not(expr: &LogicalExpression) -> LogicalExpression {
        match expr {
            LogicalExpression::Constant(c) => LogicalExpression::Constant(!c),
            LogicalExpression::Variable(variable) => {
                LogicalExpression::Not(Box::new(LogicalExpression::Variable(variable.clone())))
            }
            LogicalExpression::Not(inner) => *inner.clone(),
            LogicalExpression::And(v) => {
                let mut negated_terms = LogicalBucket::new();
                negated_terms
                    .constants
                    .extend(v.constants.iter().map(|constant| !constant));
                negated_terms.expressions.extend(
                    v.variables
                        .iter()
                        .map(|variable| LogicalExpression::not(&LogicalExpression::Variable(variable.clone()))),
                );
                negated_terms
                    .expressions
                    .extend(v.expressions.iter().map(LogicalExpression::not));
                LogicalExpression::Or(negated_terms)
            }
            LogicalExpression::Or(v) => {
                let mut negated_terms = LogicalBucket::new();
                negated_terms
                    .constants
                    .extend(v.constants.iter().map(|constant| !constant));
                negated_terms.expressions.extend(
                    v.variables
                        .iter()
                        .map(|variable| LogicalExpression::not(&LogicalExpression::Variable(variable.clone()))),
                );
                negated_terms
                    .expressions
                    .extend(v.expressions.iter().map(LogicalExpression::not));
                LogicalExpression::And(negated_terms)
            }
            LogicalExpression::Relation {
                left,
                operator: relation,
                right,
            } => match relation {
                Symbol::Relation(Relation::Equal) => LogicalExpression::Relation {
                    left: left.clone(),
                    operator: Symbol::Relation(Relation::NotEqual),
                    right: right.clone(),
                },
                Symbol::Relation(Relation::NotEqual) => LogicalExpression::Relation {
                    left: left.clone(),
                    operator: Symbol::Relation(Relation::Equal),
                    right: right.clone(),
                },
                Symbol::Relation(Relation::LessThan) => LogicalExpression::Relation {
                    left: left.clone(),
                    operator: Symbol::Relation(Relation::GreaterEqual),
                    right: right.clone(),
                },
                Symbol::Relation(Relation::GreaterThan) => LogicalExpression::Relation {
                    left: left.clone(),
                    operator: Symbol::Relation(Relation::LessEqual),
                    right: right.clone(),
                },
                Symbol::Relation(Relation::LessEqual) => LogicalExpression::Relation {
                    left: left.clone(),
                    operator: Symbol::Relation(Relation::GreaterThan),
                    right: right.clone(),
                },
                Symbol::Relation(Relation::GreaterEqual) => LogicalExpression::Relation {
                    left: left.clone(),
                    operator: Symbol::Relation(Relation::LessThan),
                    right: right.clone(),
                },
                _ => panic!("unsupported relation operator: {}", relation),
            },
        }
    }

    pub fn and(expr1: &LogicalExpression, expr2: &LogicalExpression) -> LogicalExpression {
        match (expr1, expr2) {
            // 常量折叠
            (LogicalExpression::Constant(c1), LogicalExpression::Constant(c2)) => {
                LogicalExpression::constant(*c1 && *c2)
            }
            (_, LogicalExpression::Constant(false)) => LogicalExpression::Constant(false),
            (l, LogicalExpression::Constant(true)) => l.clone(),
            // And + And
            (LogicalExpression::And(v1), LogicalExpression::And(v2)) => {
                let mut combined = v1.clone();
                combined.extend(v2);
                LogicalExpression::And(combined)
            }
            // And + r
            (LogicalExpression::And(v), r) => {
                LogicalExpression::And(v.clone().with_appended(r.clone()))
            }
            // l + And
            (l, LogicalExpression::And(v)) => {
                LogicalExpression::And(v.clone().with_appended(l.clone()))
            }
            // fallback
            (l, r) => LogicalExpression::And(logical_bucket![l.clone(), r.clone()]),
        }
    }

    pub fn or(expr1: &LogicalExpression, expr2: &LogicalExpression) -> LogicalExpression {
        match (expr1, expr2) {
            // 常量折叠
            (LogicalExpression::Constant(c1), LogicalExpression::Constant(c2)) => {
                LogicalExpression::constant(*c1 || *c2)
            }
            (_, LogicalExpression::Constant(true)) => LogicalExpression::Constant(true),
            (l, LogicalExpression::Constant(false)) => l.clone(),
            // Or + Or
            (LogicalExpression::Or(v1), LogicalExpression::Or(v2)) => {
                let mut combined = v1.clone();
                combined.extend(v2);
                LogicalExpression::Or(combined)
            }
            // Or + r
            (LogicalExpression::Or(v), r) => {
                LogicalExpression::Or(v.clone().with_appended(r.clone()))
            }
            // l + Or
            (l, LogicalExpression::Or(v)) => {
                LogicalExpression::Or(v.clone().with_appended(l.clone()))
            }
            (l, r) => LogicalExpression::Or(logical_bucket![l.clone(), r.clone()]),
        }
    }

    pub fn relation(
        left: &NumericExpression,
        operator: &Symbol,
        right: &NumericExpression,
    ) -> LogicalExpression {
        if let (NumericExpression::Constant(left_num), NumericExpression::Constant(right_num)) =
            (left, right)
        {
            let left = left_num.to_float();
            let right = right_num.to_float();

            let result = match operator {
                Symbol::Relation(Relation::Equal) => (left - right).abs() < 1e-9,
                Symbol::Relation(Relation::NotEqual) => (left - right).abs() >= 1e-9,
                Symbol::Relation(Relation::LessThan) => left < right,
                Symbol::Relation(Relation::GreaterThan) => left > right,
                Symbol::Relation(Relation::LessEqual) => left <= right,
                Symbol::Relation(Relation::GreaterEqual) => left >= right,
                _ => false,
            };

            return LogicalExpression::Constant(result);
        }

        LogicalExpression::Relation {
            left: Box::new(left.clone()),
            operator: operator.clone(),
            right: Box::new(right.clone()),
        }
    }

    pub fn substitute(
        &self,
        target: &Variable,
        replacement: Option<&crate::semantic::semantic_ir::SemanticExpression>,
    ) -> LogicalExpression {
        match self {
            LogicalExpression::Constant(_) => self.clone(),
            LogicalExpression::Variable(variable) => {
                if variable == target {
                    match replacement {
                        Some(crate::semantic::semantic_ir::SemanticExpression::Logical(
                            logical,
                        )) => logical.clone(),
                        Some(crate::semantic::semantic_ir::SemanticExpression::Numeric(_)) => {
                            panic!("cannot substitute a logical variable with a numeric expression")
                        }
                        None => self.clone(),
                    }
                } else {
                    self.clone()
                }
            }
            LogicalExpression::Not(inner) => {
                LogicalExpression::not(&inner.substitute(target, replacement))
            }
            LogicalExpression::And(bucket) => {
                substitute_logical_bucket(bucket, target, replacement, true)
            }
            LogicalExpression::Or(bucket) => {
                substitute_logical_bucket(bucket, target, replacement, false)
            }
            LogicalExpression::Relation {
                left,
                operator,
                right,
            } => {
                let numeric_replacement = match replacement {
                    Some(crate::semantic::semantic_ir::SemanticExpression::Numeric(numeric)) => {
                        Some(numeric)
                    }
                    Some(crate::semantic::semantic_ir::SemanticExpression::Logical(_))
                        if target.var_type != crate::semantic::variable::VariableType::Boolean =>
                    {
                        panic!("cannot substitute a numeric variable with a logical expression")
                    }
                    _ => None,
                };

                LogicalExpression::relation(
                    &left.substitute(target, numeric_replacement),
                    operator,
                    &right.substitute(target, numeric_replacement),
                )
            }
        }
    }
}

impl_logic_ir_unary_operation!(Not, not, not);
impl_logic_ir_binary_operation!(BitAnd, bitand, and);
impl_logic_ir_binary_operation!(BitOr, bitor, or);
impl_logic_ir_logic_operation!(BitAnd, bitand, and, bool);
impl_logic_ir_logic_operation!(BitOr, bitor, or, bool);

impl fmt::Display for LogicalExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LogicalExpression::Constant(c) => {
                write!(f, "{}", c)
            }
            LogicalExpression::Variable(v) => {
                write!(f, "{}", v)
            }
            LogicalExpression::Not(n) => {
                write!(f, "NOT ({})", n)
            }
            LogicalExpression::And(bucket) => {
                let terms = render_logical_bucket(bucket);
                write!(f, "({})", terms.join(" AND "))
            }
            LogicalExpression::Or(bucket) => {
                let terms = render_logical_bucket(bucket);
                write!(f, "({})", terms.join(" OR "))
            }
            LogicalExpression::Relation {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", left, operator, right)
            }
        }
    }
}

fn substitute_logical_bucket(
    bucket: &LogicalBucket,
    target: &Variable,
    replacement: Option<&crate::semantic::semantic_ir::SemanticExpression>,
    and_mode: bool,
) -> LogicalExpression {
    let mut folded: Option<LogicalExpression> = None;

    for constant in &bucket.constants {
        let expr = LogicalExpression::Constant(*constant);
        folded = Some(match folded {
            Some(acc) if and_mode => acc & expr,
            Some(acc) => acc | expr,
            None => expr,
        });
    }

    for variable in &bucket.variables {
        let expr = substitute_logical_variable(variable, target, replacement);
        folded = Some(match folded {
            Some(acc) if and_mode => acc & expr,
            Some(acc) => acc | expr,
            None => expr,
        });
    }

    for expr in &bucket.expressions {
        let expr = expr.substitute(target, replacement);
        folded = Some(match folded {
            Some(acc) if and_mode => acc & expr,
            Some(acc) => acc | expr,
            None => expr,
        });
    }

    folded.unwrap_or_else(|| {
        if and_mode {
            LogicalExpression::constant(true)
        } else {
            LogicalExpression::constant(false)
        }
    })
}

fn render_logical_bucket(bucket: &LogicalBucket) -> Vec<String> {
    let mut rendered =
        Vec::with_capacity(bucket.constants.len() + bucket.variables.len() + bucket.expressions.len());
    rendered.extend(bucket.constants.iter().map(|constant| constant.to_string()));
    rendered.extend(bucket.variables.iter().map(|variable| variable.to_string()));
    rendered.extend(bucket.expressions.iter().map(|expr| expr.to_string()));
    rendered
}

fn substitute_logical_variable(
    variable: &Variable,
    target: &Variable,
    replacement: Option<&crate::semantic::semantic_ir::SemanticExpression>,
) -> LogicalExpression {
    if variable == target {
        match replacement {
            Some(crate::semantic::semantic_ir::SemanticExpression::Logical(logical)) => {
                logical.clone()
            }
            Some(crate::semantic::semantic_ir::SemanticExpression::Numeric(_)) => {
                panic!("cannot substitute a logical variable with a numeric expression")
            }
            None => LogicalExpression::Variable(variable.clone()),
        }
    } else {
        LogicalExpression::Variable(variable.clone())
    }
}

