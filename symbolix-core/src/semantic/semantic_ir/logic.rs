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
use tree_drawer::tree::OwnedTree;

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
            LogicalExpression::Variable(_) => LogicalExpression::Not(Box::new(expr.clone())),
            LogicalExpression::Not(inner) => *inner.clone(),
            LogicalExpression::And(v) => {
                let negated_terms: LogicalBucket =
                    v.iter().map(|term| LogicalExpression::not(&term)).collect();
                LogicalExpression::Or(negated_terms)
            }
            LogicalExpression::Or(v) => {
                let negated_terms: LogicalBucket =
                    v.iter().map(|term| LogicalExpression::not(&term)).collect();
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
                let mut combined = v.clone();
                combined.push(r.clone());
                LogicalExpression::And(combined)
            }
            // l + And
            (l, LogicalExpression::And(v)) => {
                let mut combined = logical_bucket![l.clone()];
                combined.extend(v);
                LogicalExpression::And(combined)
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
                let mut combined = v.clone();
                combined.push(r.clone());
                LogicalExpression::Or(combined)
            }
            // l + Or
            (l, LogicalExpression::Or(v)) => {
                let mut combined = logical_bucket![l.clone()];
                combined.extend(v);
                LogicalExpression::Or(combined)
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
                let mut iter = bucket
                    .iter()
                    .map(|expr| expr.substitute(target, replacement));
                match iter.next() {
                    Some(first) => iter.fold(first, |acc, expr| acc & expr),
                    None => LogicalExpression::constant(true),
                }
            }
            LogicalExpression::Or(bucket) => {
                let mut iter = bucket
                    .iter()
                    .map(|expr| expr.substitute(target, replacement));
                match iter.next() {
                    Some(first) => iter.fold(first, |acc, expr| acc | expr),
                    None => LogicalExpression::constant(false),
                }
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
                let terms: Vec<String> = bucket.iter().map(|term| format!("{}", term)).collect();
                write!(f, "({})", terms.join(" AND "))
            }
            LogicalExpression::Or(bucket) => {
                let terms: Vec<String> = bucket.iter().map(|term| format!("{}", term)).collect();
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

impl LogicalExpression {
    pub fn to_owned_tree(&self) -> OwnedTree {
        match self {
            LogicalExpression::Constant(b) => OwnedTree::new(format!("{b}")),

            LogicalExpression::Variable(v) => OwnedTree::new(format!("{v}")),

            LogicalExpression::Not(expr) => {
                OwnedTree::new("NOT".to_string()).with_child(expr.to_owned_tree())
            }

            LogicalExpression::And(bucket) => {
                let mut node = OwnedTree::new("AND".to_string());
                for term in bucket.iter() {
                    node = node.with_child(term.to_owned_tree());
                }
                node
            }

            LogicalExpression::Or(bucket) => {
                let mut node = OwnedTree::new("OR".to_string());
                for term in bucket.iter() {
                    node = node.with_child(term.to_owned_tree());
                }
                node
            }

            LogicalExpression::Relation {
                left,
                operator,
                right,
            } => OwnedTree::new(format!("{operator}"))
                .with_child(left.to_owned_tree())
                .with_child(right.to_owned_tree()),
        }
    }
}
