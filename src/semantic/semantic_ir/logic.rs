use std::fmt;
use std::fmt::Formatter;
use tree_drawer::tree::OwnedTree;
use crate::lexer::symbol::{Relation, Symbol};
use crate::logical_bucket;
use crate::semantic::bucket::LogicalBucket;
use crate::semantic::semantic_ir::numeric::NumericExpression;
use crate::semantic::semantic_ir::SemanticExpression;
use crate::semantic::variable::Variable;


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

    pub fn not(expr: LogicalExpression) -> LogicalExpression {
        match expr {
            LogicalExpression::Constant(c) => LogicalExpression::Constant(!c),
            LogicalExpression::Variable(_) => LogicalExpression::Not(Box::new(expr)),
            LogicalExpression::Not(inner) => *inner,
            LogicalExpression::And(v) => {
                let negated_terms: LogicalBucket = v.into_iter()
                    .map(|term| LogicalExpression::not(term))
                    .collect();
                LogicalExpression::Or(negated_terms)
            }
            LogicalExpression::Or(v) => {
                let negated_terms: LogicalBucket = v.into_iter()
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
                let mut combined = logical_bucket![l];
                combined.extend(v);
                LogicalExpression::And(combined)
            }
            // fallback
            (l, r) => LogicalExpression::And(logical_bucket![l, r]),
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
                let mut combined = logical_bucket![l];
                combined.extend(v);
                LogicalExpression::Or(combined)
            }
            (l, r) => LogicalExpression::Or(logical_bucket![l, r]),
        }
    }

    pub fn relation(left: NumericExpression, operator: Symbol, right: NumericExpression) -> LogicalExpression {
        LogicalExpression::Relation {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn normalize(&mut self) {
        // 对表达式进行层序遍历
        let mut stack: Vec<LogicalExpression> = vec![self.clone()];
        while !stack.is_empty() {
            match stack.pop().unwrap() {
                LogicalExpression::And(mut bucket) => {
                    bucket.execute_constant(true);
                    for item in bucket {
                        stack.push(item);
                    }
                }
                LogicalExpression::Or(mut bucket) => {
                    bucket.execute_constant(false);
                    for item in bucket {
                        stack.push(item);
                    }
                }
                LogicalExpression::Not(inner) => {
                    stack.push(*inner);
                }
                LogicalExpression::Relation { mut left, operator: _, mut right } => {
                    left.normalize();
                    right.normalize();
                }
                _ => {}
            }
        }
    }
}

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
            LogicalExpression::Relation { left, operator, right } => {
                write!(f, "({} {} {})", left, operator, right)
            }
        }
    }
}

impl SemanticExpression {
    pub fn to_owned_tree(&self) -> OwnedTree {
        match self {
            SemanticExpression::Numeric(expr) => {
                expr.to_owned_tree()
            }
            SemanticExpression::Logical(expr) => {
                expr.to_owned_tree()
            }
        }
    }
}


impl LogicalExpression {
    pub fn to_owned_tree(&self) -> OwnedTree {
        match self {
            LogicalExpression::Constant(b) => {
                OwnedTree::new(format!("{b}"))
            }

            LogicalExpression::Variable(v) => {
                OwnedTree::new(format!("{v}"))
            }

            LogicalExpression::Not(expr) => {
                OwnedTree::new("NOT".to_string())
                    .with_child(expr.to_owned_tree())
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

            LogicalExpression::Relation { left, operator, right } => {
                OwnedTree::new(format!("{operator}"))
                    .with_child(left.to_owned_tree())
                    .with_child(right.to_owned_tree())
            }
        }
    }
}
